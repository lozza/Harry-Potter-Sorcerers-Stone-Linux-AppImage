use crate::{sha256_file, BuildEvent, BuildRequest, BuildStage, CoreError, DisplayProfile, EditionProfile, EventSink};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const INNOEXTRACT_SHA256: &str = "008efe5011476ccc4aae17c3e22038b5a1bc5c7aad2b9d4d869537bf3874d21f";
const DXGI_SHA256: &str = "82a9d8a2cfc5589c91fc0144579e8793d2064071a6ec5edcba926b6b19c93dfd";
const D3D11_SHA256: &str = "b9280d6183ed00873a68e2fd190688f6ced92cfc9e9478be87b22cb4be7c8946";
const APPIMAGE_RUNTIME_SHA256: &str = "4448aff037fa32788d2fb8ac9a10bd9688cd95ffcebbda05d1962278d0fa8c47";
const RUNTIME32_LOADER_SHA256: &str = "59e07d873c208c2e11518e5b20dc867adad0c544730c4cf193ee03160873369e";
const RUNTIME32_LIBC_SHA256: &str = "15530c2512962b355a9fe72df566615db585acc6fcbe2a7597fc066dab6c52bc";
// Wine's ELF interpreter is the host /lib/ld-linux.so.2. Bundling a newer
// glibc beside it mixes incompatible private loader/libc internals on Deck.
// Leave non-glibc runtime libraries in place and use the host's matched pair.
const HOST_GLIBC_FILES: &[&str] = &[
    "ld-linux.so.2", "libc.so", "libc.so.6", "libm.so.6", "libmvec.so.1",
    "libpthread.so.0", "libdl.so.2", "librt.so.1", "libutil.so.1", "libanl.so.1",
    "libresolv.so.2", "libnss_compat.so.2", "libnss_db.so.2", "libnss_dns.so.2",
    "libnss_files.so.2", "libnss_hesiod.so.2", "libnss_resolve.so.2",
    "libBrokenLocale.so.1", "libthread_db.so.1", "libmemusage.so", "libpcprofile.so",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitState { Ready, Initialize, RecoverInterrupted, Busy }

pub fn init_state(game_ready: bool, prefix_ready: bool, lock_owner_alive: bool) -> InitState {
    if game_ready && prefix_ready { InitState::Ready }
    else if lock_owner_alive { InitState::Busy }
    else if game_ready || prefix_ready { InitState::RecoverInterrupted }
    else { InitState::Initialize }
}

/// Returns the active Flatpak GL32 runtime directory for a vendor/version
/// pair. It deliberately returns a host path; callers must never package it.
pub fn driver_candidate_from_flatpak_root(root: &Path, nvidia_version: Option<&str>) -> Option<PathBuf> {
    let id = match nvidia_version {
        Some(version) => format!("org.freedesktop.Platform.GL32.nvidia-{}", version.replace('.', "-")),
        None => "org.freedesktop.Platform.GL32.default".into(),
    };
    let candidate = root.join(id);
    if candidate.is_dir() { Some(candidate) } else { None }
}

struct Toolchain { innoextract: PathBuf, reference_appdir: PathBuf, appimagetool: PathBuf, appimage_runtime: PathBuf, packager_path: Option<std::ffi::OsString> }

pub fn build_private_appimage(request: &BuildRequest, profile: &EditionProfile, sink: &mut dyn EventSink) -> Result<(), CoreError> {
    let tools = private_toolchain()?;
    let output_dir = &request.output;
    fs::create_dir_all(output_dir)?;
    if !fs::metadata(output_dir)?.is_dir() { return Err(CoreError::InvalidInput("Output must be a directory.".into())); }
    let artifact = output_dir.join("Harry-Potter-Sorcerers-Stone-private.AppImage");
    if artifact.exists() { return Err(CoreError::InvalidInput(format!("Refusing to overwrite existing output {}.", artifact.display()))); }
    let nonce = format!("{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| CoreError::Io(e.to_string()))?.as_millis());
    let work = output_dir.join(format!(".hp1-build-{nonce}"));
    fs::create_dir(&work)?;
    let result = build_into(&work, &artifact, request, profile, &tools, sink);
    if result.is_err() || !request.keep_workdir { let _ = fs::remove_dir_all(&work); }
    result
}

fn private_toolchain() -> Result<Toolchain, CoreError> {
    let path = |variable: &str| -> Result<PathBuf, CoreError> {
        let value = env::var_os(variable).ok_or_else(|| CoreError::Unsupported(format!(
            "Private integration requires {variable}; no system Wine/Bottles fallback is permitted."
        )))?;
        let path = PathBuf::from(value);
        let meta = fs::metadata(&path).map_err(|_| CoreError::InvalidInput(format!("{variable} does not name a readable path.")))?;
        if !meta.is_file() { return Err(CoreError::InvalidInput(format!("{variable} must name a regular file."))); }
        Ok(path)
    };
    let innoextract = path("HP1_INNOEXTRACT")?;
    if sha256_file(&innoextract)? != INNOEXTRACT_SHA256 && !innoextract.to_string_lossy().ends_with("/innoextract") {
        return Err(CoreError::Unsupported("HP1_INNOEXTRACT does not match the audited innoextract 1.9 archive or launcher layout.".into()));
    }
    let reference_appdir = PathBuf::from(env::var_os("HP1_PRIVATE_REFERENCE_APPDIR").ok_or_else(|| CoreError::Unsupported(
        "Private integration requires HP1_PRIVATE_REFERENCE_APPDIR containing the audited runner/runtime; it is never a public release input.".into()
    ))?);
    for relative in ["runner/bin/wine", "runner/bin/wineserver", "runtime32/ld-linux.so.2", "runtime32/libc.so.6", "prefix-template/drive_c/windows/system32/dxgi.dll", "prefix-template/drive_c/windows/system32/d3d11.dll"] {
        if !reference_appdir.join(relative).exists() { return Err(CoreError::InvalidInput(format!("Private reference AppDir is missing {relative}."))); }
    }
    for (relative, expected) in [("runtime32/ld-linux.so.2", RUNTIME32_LOADER_SHA256), ("runtime32/libc.so.6", RUNTIME32_LIBC_SHA256)] {
        if sha256_file(&reference_appdir.join(relative))? != expected {
            return Err(CoreError::Unsupported(format!("Private reference {relative} is not the audited matching 32-bit glibc component.")));
        }
    }
    let appimage_runtime = path("HP1_APPIMAGE_RUNTIME")?;
    if sha256_file(&appimage_runtime)? != APPIMAGE_RUNTIME_SHA256 {
        return Err(CoreError::Unsupported("HP1_APPIMAGE_RUNTIME does not match the audited private x86_64 AppImage runtime.".into()));
    }
    Ok(Toolchain { innoextract, reference_appdir, appimagetool: path("HP1_APPIMAGETOOL")?, appimage_runtime, packager_path: env::var_os("HP1_PACKAGER_PATH") })
}

fn build_into(work: &Path, artifact: &Path, request: &BuildRequest, profile: &EditionProfile, tools: &Toolchain, sink: &mut dyn EventSink) -> Result<(), CoreError> {
    let installer = work.join("installer"); fs::create_dir(&installer)?;
    sink.emit(BuildEvent::started(BuildStage::ExtractInstaller));
    let setup = installer.join("setup.exe");
    extract_zip_member(&request.source_zip, &profile.installer_path, &setup, &profile.installer_sha256, None)?;
    let data = installer.join("setup-1.bin");
    extract_zip_member(&request.source_zip, &profile.data_path, &data, &profile.data_sha256, Some(profile.data_size))?;
    let extracted = work.join("inno");
    run(Command::new(&tools.innoextract).current_dir(&installer).args(["--extract", "--output-dir"]).arg(&extracted).args(["--include", "app", "setup.exe"]), "innoextract direct app extraction")?;
    let payload = extracted.join("app");
    if !payload.join("System/HP.exe").is_file() { return Err(CoreError::Unsupported("innoextract did not produce the expected app/System/HP.exe payload.".into())); }
    sink.emit(BuildEvent::completed(BuildStage::ExtractInstaller));

    sink.emit(BuildEvent::started(BuildStage::PreparePayload));
    let appdir = work.join("AppDir"); fs::create_dir(&appdir)?;
    copy_tree(&payload, &appdir.join("game"))?;
    remove_installer_artifacts(&appdir.join("game"))?;
    copy_tree(&tools.reference_appdir.join("runner"), &appdir.join("runner"))?;
    copy_tree(&tools.reference_appdir.join("runtime32"), &appdir.join("runtime32"))?;
    remove_bundled_glibc(&appdir.join("runtime32"))?;
    let dxvk = appdir.join("dxvk"); fs::create_dir(&dxvk)?;
    copy_checked(&tools.reference_appdir.join("prefix-template/drive_c/windows/system32/dxgi.dll"), &dxvk.join("dxgi.dll"), DXGI_SHA256)?;
    copy_checked(&tools.reference_appdir.join("prefix-template/drive_c/windows/system32/d3d11.dll"), &dxvk.join("d3d11.dll"), D3D11_SHA256)?;
    sink.emit(BuildEvent::completed(BuildStage::PreparePayload));

    sink.emit(BuildEvent::started(BuildStage::AssembleAppDir));
    write_appdir_files(&appdir, request.default_profile)?;
    sink.emit(BuildEvent::completed(BuildStage::AssembleAppDir));

    sink.emit(BuildEvent::started(BuildStage::PackageAppImage));
    let temporary = artifact.with_extension("AppImage.partial");
    let mut packager = Command::new(&tools.appimagetool);
    // appimagetool is itself an AppImage.  Its supported extraction mode keeps
    // private builds usable on hosts where FUSE is deliberately unavailable.
    packager.env("ARCH", "x86_64").env("APPIMAGE_EXTRACT_AND_RUN", "1").args(["--no-appstream", "--comp", "zstd", "--runtime-file"]).arg(&tools.appimage_runtime).arg(&appdir).arg(&temporary);
    if let Some(path) = &tools.packager_path { packager.env("PATH", path); }
    run(&mut packager, "appimagetool private packaging")?;
    if !temporary.is_file() || fs::metadata(&temporary)?.len() == 0 { return Err(CoreError::Io("appimagetool did not create a non-empty AppImage.".into())); }
    fs::rename(&temporary, artifact)?;
    sink.emit(BuildEvent::completed(BuildStage::PackageAppImage));
    sink.emit(BuildEvent::started(BuildStage::ValidateOutput));
    sink.emit(BuildEvent::message(BuildStage::ValidateOutput, format!("Created private integration AppImage: {}", artifact.display())));
    sink.emit(BuildEvent::completed(BuildStage::ValidateOutput));
    Ok(())
}

fn extract_zip_member(zip: &Path, member: &str, destination: &Path, expected_hash: &str, expected_size: Option<u64>) -> Result<(), CoreError> {
    let file = File::create(destination)?;
    let unzip = env::var_os("HP1_UNZIP").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/bin/unzip"));
    run(Command::new(unzip).args(["-p"]).arg(zip).arg(member).stdout(Stdio::from(file)), "safe extraction of a profile-approved ZIP member")?;
    if let Some(size) = expected_size { if fs::metadata(destination)?.len() != size { return Err(CoreError::InvalidInput(format!("Extracted {member} has an unexpected size."))); } }
    if !sha256_file(destination)?.eq_ignore_ascii_case(expected_hash) { return Err(CoreError::InvalidInput(format!("Extracted {member} has an unexpected SHA-256."))); }
    Ok(())
}

fn copy_checked(source: &Path, destination: &Path, expected_hash: &str) -> Result<(), CoreError> {
    if sha256_file(source)? != expected_hash { return Err(CoreError::Unsupported(format!("Private reference component {} failed its audited hash check.", source.display()))); }
    fs::copy(source, destination)?;
    Ok(())
}

fn run(command: &mut Command, description: &str) -> Result<(), CoreError> {
    let status = command.status().map_err(|e| CoreError::Io(format!("Could not start {description}: {e}")))?;
    if status.success() { Ok(()) } else { Err(CoreError::Unsupported(format!("{description} failed with {status}."))) }
}

fn remove_installer_artifacts(game: &Path) -> Result<(), CoreError> {
    for name in ["unins000.exe", "unins000.dat"] { let path = game.join(name); if path.exists() { fs::remove_file(path)?; } }
    Ok(())
}

fn remove_bundled_glibc(runtime32: &Path) -> Result<(), CoreError> {
    for name in HOST_GLIBC_FILES {
        let path = runtime32.join(name);
        if fs::symlink_metadata(&path).is_ok() { fs::remove_file(path)?; }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), CoreError> {
    let root = fs::canonicalize(source)?;
    copy_tree_inner(source, destination, &root)
}

fn copy_tree_inner(source: &Path, destination: &Path, root: &Path) -> Result<(), CoreError> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        let resolved = fs::canonicalize(source).map_err(|_| CoreError::InvalidInput(format!("Broken reference symlink: {}", source.display())))?;
        if !resolved.starts_with(root) { return Err(CoreError::InvalidInput(format!("Reference symlink escapes its tree: {}", source.display()))); }
        #[cfg(unix)] std::os::unix::fs::symlink(fs::read_link(source)?, destination)?;
        #[cfg(not(unix))] return Err(CoreError::Unsupported("Private AppDir assembly requires Unix symlink support.".into()));
    } else if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        for child in fs::read_dir(source)? { let child = child?; copy_tree_inner(&child.path(), &destination.join(child.file_name()), root)?; }
    } else if metadata.is_file() { fs::copy(source, destination)?; }
    else { return Err(CoreError::InvalidInput(format!("Unsupported reference file type: {}", source.display()))); }
    Ok(())
}

fn write_appdir_files(appdir: &Path, default_profile: DisplayProfile) -> Result<(), CoreError> {
    let launcher = APP_RUN.replace("__DEFAULT_PROFILE__", default_profile.id());
    write_executable(&appdir.join("AppRun"), &launcher)?;
    fs::write(appdir.join("harry-potter.desktop"), "[Desktop Entry]\nType=Application\nName=Harry Potter and the Sorcerer's Stone (private)\nExec=harry-potter\nIcon=harry-potter\nCategories=Game;\nTerminal=false\n")?;
    fs::write(appdir.join("harry-potter.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"64\" height=\"64\"><rect width=\"64\" height=\"64\" fill=\"#381b09\"/><text x=\"10\" y=\"43\" font-size=\"28\" fill=\"#f6d681\">HP</text></svg>\n")?;
    fs::write(appdir.join("PRIVATE_PROVENANCE.txt"), "Private integration artifact only. Game data came from the user's verified local ZIP. Runner/runtime and DXVK DLLs were copied from a locally audited reference AppImage solely for this private test. Host GPU drivers are discovered at launch and are never bundled. Redistribution readiness has not been assessed.\n")?;
    Ok(())
}

fn write_executable(path: &Path, contents: &str) -> Result<(), CoreError> {
    let mut file = File::create(path)?; file.write_all(contents.as_bytes())?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; let mut permissions = fs::metadata(path)?.permissions(); permissions.set_mode(0o755); fs::set_permissions(path, permissions)?; }
    Ok(())
}

const APP_RUN: &str = r##"#!/bin/sh
set -eu
APPDIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DATA_HOME=${XDG_DATA_HOME:-"$HOME/.local/share"}
DATA_DIR="$DATA_HOME/hp1-magipack-private"
GAME_DIR="$DATA_DIR/game"
PREFIX_DIR="$DATA_DIR/prefix"
LOCK_DIR="$DATA_DIR/.initializing.lock"
BUILDER_DEFAULT_PROFILE=__DEFAULT_PROFILE__
PROFILE_MARKER="$DATA_DIR/builder-default-profile"

usage() { printf '%s\n' "Usage: $0 [--choose|--auto|--720p|--1080p|--deck|--ultrawide|--windowed|--resolution WIDTHxHEIGHT]"; }
profile=${HP_PROFILE:-}
[ -z "$profile" ] && explicit_profile=0 || explicit_profile=1
choose=0
if [ "$#" -gt 0 ]; then
  explicit_profile=1
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --choose) choose=1; profile=custom ;;
    --auto) profile=1080p ;;
    --720p) profile=720p ;;
    --1080p) profile=1080p ;;
    --deck) profile=deck ;;
    --ultrawide) profile=ultrawide ;;
    --windowed) profile=windowed ;;
    --resolution) [ "$#" -eq 2 ] || { usage >&2; exit 2; }; case "$2" in *x*) w=${2%x*}; h=${2#*x};; *) usage >&2; exit 2;; esac; case "$w:$h" in *[!0-9:]*|:*) usage >&2; exit 2;; esac; profile="custom:$w:$h" ;;
    *) usage >&2; exit 2 ;;
  esac
fi
mkdir -p "$DATA_DIR"
if ! mkdir "$LOCK_DIR" 2>/dev/null; then
  if [ -r "$LOCK_DIR/pid" ] && kill -0 "$(cat "$LOCK_DIR/pid")" 2>/dev/null; then printf '%s\n' "Another first-run initialization is in progress." >&2; exit 75; fi
  rm -rf "$LOCK_DIR"
  mkdir "$LOCK_DIR" || exit 75
fi
for stale in "$DATA_DIR"/.game.new.* "$DATA_DIR"/.prefix.new.*; do [ -e "$stale" ] && rm -rf "$stale"; done
printf '%s\n' "$$" > "$LOCK_DIR/pid"
cleanup() { rm -rf "$LOCK_DIR"; }
trap cleanup EXIT HUP INT TERM

copy_game() { tmp="$DATA_DIR/.game.new.$$"; rm -rf "$tmp"; mkdir "$tmp"; cp -a "$APPDIR/game/." "$tmp/"; chmod -R u+rwX "$tmp"; mv "$tmp" "$GAME_DIR"; }
setup_env() {
  export USER=steamuser LOGNAME=steamuser WINEUSERNAME=steamuser WINEPREFIX="$PREFIX_DIR" WINEARCH=win32
  export WINELOADER="$APPDIR/runner/bin/wine" WINESERVER="$APPDIR/runner/bin/wineserver" PATH="$APPDIR/runner/bin:$PATH"
  export LD_LIBRARY_PATH="$APPDIR/runtime32/pulseaudio:$APPDIR/runtime32:$APPDIR/runner/lib32:$APPDIR/runner/lib:$APPDIR/runner/lib32/wine/i386-unix${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  export WINEDLLPATH="$APPDIR/runner/lib/wine:$APPDIR/runner/lib32/wine" FONTCONFIG_PATH="$APPDIR/runtime32/etc/fonts" WINEDLLOVERRIDES="winemenubuilder=;mscoree="
  export WINE_LARGE_ADDRESS_AWARE=1 STAGING_SHARED_MEMORY=1 DISABLE_LFX=1
  mkdir -p "$PREFIX_DIR/cache/dxvk_shader" "$PREFIX_DIR/cache/gl_shader" "$PREFIX_DIR/cache/mesa_shader" "$PREFIX_DIR/cache/vkd3d_shader"
  export DXVK_SHADER_CACHE_PATH="$PREFIX_DIR/cache/dxvk_shader" __GL_SHADER_DISK_CACHE=1 __GL_SHADER_DISK_CACHE_PATH="$PREFIX_DIR/cache/gl_shader" MESA_SHADER_CACHE_DIR="$PREFIX_DIR/cache/mesa_shader" VKD3D_SHADER_CACHE_PATH="$PREFIX_DIR/cache/vkd3d_shader"
}
init_prefix() {
  tmp="$DATA_DIR/.prefix.new.$$"; rm -rf "$tmp"; mkdir "$tmp"; PREFIX_DIR="$tmp"; setup_env; "$APPDIR/runner/bin/wineboot" -u
  rm -f "$tmp/dosdevices/z:"; ln -s "$GAME_DIR" "$tmp/dosdevices/d:"
  mkdir -p "$tmp/drive_c/users/steamuser/Documents/Harry Potter"; cp "$APPDIR/dxvk/dxgi.dll" "$tmp/drive_c/windows/system32/dxgi.dll"; cp "$APPDIR/dxvk/d3d11.dll" "$tmp/drive_c/windows/system32/d3d11.dll"
  "$APPDIR/runner/bin/wine" reg add 'HKCU\Software\Wine\DllOverrides' /v ddraw /d native,builtin /f
  "$APPDIR/runner/bin/wine" reg add 'HKCU\Software\Wine\DllOverrides' /v dxgi /d native,builtin /f
  "$APPDIR/runner/bin/wine" reg add 'HKCU\Software\Wine\DllOverrides' /v d3d11 /d native,builtin /f
  PREFIX_DIR="$tmp"; write_initial_ini; mv "$tmp" "$DATA_DIR/prefix"; PREFIX_DIR="$DATA_DIR/prefix"
}
write_initial_ini() {
  dir="$PREFIX_DIR/drive_c/users/steamuser/Documents/Harry Potter"; mkdir -p "$dir/Save"
  cp "$GAME_DIR/System/Default.ini" "$dir/HP.ini.tmp"; mv "$dir/HP.ini.tmp" "$dir/HP.ini"
  cp "$GAME_DIR/System/DefUser.ini" "$dir/User.ini.tmp"; mv "$dir/User.ini.tmp" "$dir/User.ini"
  sed -i 's/^Reconfig=.*/Reconfig=0/;s/^ForceSoftware=.*/ForceSoftware=0/;s#^GameRenderDevice=.*#GameRenderDevice=D3DDrv.D3DRenderDevice#;s#^SavePath=.*#SavePath=C:\\users\\steamuser\\Documents\\Harry Potter\\Save#;s#^Paths=../save/\\*.usa#Paths=C:\\users\\steamuser\\Documents\\Harry Potter\\Save\\*.usa#' "$dir/HP.ini"
}
apply_profile() {
  case "$1" in
    720p) w=1280; h=720; fov=106.26; fullscreen=True;; 1080p) w=1920; h=1080; fov=106.26; fullscreen=True;; deck) w=1280; h=800; fov=100.39; fullscreen=True;; ultrawide) w=2560; h=1080; fov=121.28; fullscreen=True;; windowed) w=1280; h=720; fov=90; fullscreen=False;; custom:*) IFS=:; set -- $1; w=$2; h=$3; fov=90; fullscreen=True;; custom) return 0;; *) return 2;; esac
  hp="$PREFIX_DIR/drive_c/users/steamuser/Documents/Harry Potter/HP.ini"; user="$PREFIX_DIR/drive_c/users/steamuser/Documents/Harry Potter/User.ini"
  sed -i "s/^Reconfig=.*/Reconfig=0/;s/^ForceSoftware=.*/ForceSoftware=0/;s#^GameRenderDevice=.*#GameRenderDevice=D3DDrv.D3DRenderDevice#;s/^WindowedViewportX=.*/WindowedViewportX=$w/;s/^WindowedViewportY=.*/WindowedViewportY=$h/;s/^FullscreenViewportX=.*/FullscreenViewportX=$w/;s/^FullscreenViewportY=.*/FullscreenViewportY=$h/;s/^WindowedColorBits=.*/WindowedColorBits=32/;s/^FullscreenColorBits=.*/FullscreenColorBits=32/;s/^StartupFullscreen=.*/StartupFullscreen=$fullscreen/" "$hp"
  sed -i "s/^DesiredFOV=.*/DesiredFOV=$fov/;s/^DefaultFOV=.*/DefaultFOV=$fov/" "$user"
}
if [ ! -f "$GAME_DIR/System/HP.exe" ]; then [ ! -e "$GAME_DIR" ] || mv "$GAME_DIR" "$DATA_DIR/game.interrupted.$$"; copy_game; fi
if [ ! -f "$PREFIX_DIR/system.reg" ]; then [ ! -e "$PREFIX_DIR" ] || mv "$PREFIX_DIR" "$DATA_DIR/prefix.interrupted.$$"; init_prefix; fi
setup_env
# A reused prefix may have cached direct ALSA endpoints from an older build.
# Prefer the host's default PipeWire/PulseAudio sink when available, but keep
# any explicit user-selected Wine audio driver unchanged.
if [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/pulse/native" ] && ! "$APPDIR/runner/bin/wine" reg query 'HKCU\Software\Wine\Drivers' /v Audio >/dev/null 2>&1; then
  "$APPDIR/runner/bin/wine" reg add 'HKCU\Software\Wine\Drivers' /v Audio /t REG_SZ /d pulse /f >/dev/null
fi
if [ "$choose" -eq 1 ]; then cd "$GAME_DIR/ResolutionTool"; "$APPDIR/runner/bin/wine" start /wait /unix "$GAME_DIR/ResolutionTool/HPSettings.exe" || true; printf '%s\n' custom > "$DATA_DIR/launch-profile"; else
  if [ -z "$profile" ] && [ -r "$DATA_DIR/launch-profile" ]; then profile=$(cat "$DATA_DIR/launch-profile"); fi
  recorded_default=$(cat "$PROFILE_MARKER" 2>/dev/null || true)
  if [ "$explicit_profile" -eq 0 ] && [ "$recorded_default" != "$BUILDER_DEFAULT_PROFILE" ]; then profile=$BUILDER_DEFAULT_PROFILE; fi
  profile=${profile:-$BUILDER_DEFAULT_PROFILE}; apply_profile "$profile" || { usage >&2; exit 2; }; printf '%s\n' "$profile" > "$DATA_DIR/launch-profile.tmp"; mv "$DATA_DIR/launch-profile.tmp" "$DATA_DIR/launch-profile"
fi
printf '%s\n' "$BUILDER_DEFAULT_PROFILE" > "$PROFILE_MARKER.tmp"; mv "$PROFILE_MARKER.tmp" "$PROFILE_MARKER"
GPU_VENDOR=
for status in /sys/class/drm/card*-*/status; do
  [ -r "$status" ] && [ "$(cat "$status")" = connected ] || continue
  card=$(basename "$(dirname "$status")" | cut -d- -f1)
  vendor="/sys/class/drm/$card/device/vendor"
  [ -r "$vendor" ] && { GPU_VENDOR=$(cat "$vendor"); break; }
done
if [ -z "$GPU_VENDOR" ]; then
  for vendor in /sys/class/drm/card*/device/vendor; do [ -r "$vendor" ] && { GPU_VENDOR=$(cat "$vendor"); break; }; done
fi
case "$GPU_VENDOR" in
  0x10de) GPU_FAMILY=nvidia;;
  0x1002) GPU_FAMILY=amd;;
  0x8086) GPU_FAMILY=intel;;
  *) printf 'Unsupported or unidentified active GPU vendor: %s\n' "$GPU_VENDOR" >&2; exit 78;;
esac
matches_icd() {
  name=$(basename "$1")
  case "$GPU_FAMILY:$name" in
    nvidia:*nvidia*|amd:*radeon*|amd:*amd*|intel:*intel*) return 0;;
    *) return 1;;
  esac
}
find_icd() {
  candidate_root=$1
  for json in "$candidate_root"/vulkan/icd.d/*.json; do [ -f "$json" ] && matches_icd "$json" || continue; lib=$(sed -n 's/.*"library_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$json" | head -n 1); [ -n "$lib" ] || continue; case "$lib" in /*) file=$lib;; *) file="$candidate_root/lib/$lib";; esac; file=$(readlink -f "$file" 2>/dev/null || true); [ -f "$file" ] || continue; is_elf32 "$file" || continue; ICD_LIBRARY=$file; return 0; done; return 1
}
is_elf32() { [ "$(od -An -j4 -N1 -t u1 "$1" 2>/dev/null | tr -d ' ')" = 1 ]; }
ICD_LIBRARY=
for jsondir in /usr/share/vulkan/icd.d /etc/vulkan/icd.d; do [ -d "$jsondir" ] || continue; for json in "$jsondir"/*.json; do [ -f "$json" ] && matches_icd "$json" || continue; lib=$(sed -n 's/.*"library_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$json" | head -n 1); [ -n "$lib" ] && [ -f "$lib" ] && is_elf32 "$lib" && { ICD_LIBRARY=$(readlink -f "$lib"); break 2; }; done; done
if [ -z "$ICD_LIBRARY" ]; then
  version=$(sed -n 's/.*Kernel Module[[:space:]][[:space:]]*\([0-9][^ ]*\).*/\1/p;s/.*Module[[:space:]]*version:[[:space:]]*\([^ ]*\).*/\1/p' /proc/driver/nvidia/version 2>/dev/null | head -n 1 || true)
  for flatpak in "$HOME/.local/share/flatpak/runtime" /var/lib/flatpak/runtime; do [ -d "$flatpak" ] || continue; if [ "$GPU_FAMILY" = nvidia ]; then [ -n "$version" ] || continue; id="org.freedesktop.Platform.GL32.nvidia-$(printf '%s' "$version" | tr . -)"; else id=org.freedesktop.Platform.GL32.default; fi; for root in "$flatpak/$id"/*/*/active/files; do [ -d "$root" ] && find_icd "$root" && break 3; done; done
fi
[ -n "$ICD_LIBRARY" ] || { printf 'No compatible host 32-bit Vulkan ICD was found for %s. No GPU driver is bundled by this AppImage.\n' "$GPU_FAMILY" >&2; exit 78; }
ICD_DIR=$(dirname "$ICD_LIBRARY"); mkdir -p "$PREFIX_DIR/cache/vulkan"; ICD_JSON="$PREFIX_DIR/cache/vulkan/host-icd.json"; printf '{"file_format_version":"1.0.0","ICD":{"library_path":"%s","api_version":"1.3.0"}}\n' "$ICD_LIBRARY" > "$ICD_JSON"; export VK_DRIVER_FILES="$ICD_JSON" VK_ICD_FILENAMES="$ICD_JSON" LD_LIBRARY_PATH="$ICD_DIR:$LD_LIBRARY_PATH"
rm -rf "$LOCK_DIR"
trap - EXIT HUP INT TERM
# Keep Wine's Pulse client paired with the bundled Pulse common library.
# Some hosts load /usr/lib32/libpulse.so.0 ahead of LD_LIBRARY_PATH.
if [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/pulse/native" ]; then
  [ -r "$APPDIR/runtime32/libpulse.so.0" ] || { printf 'Bundled 32-bit PulseAudio client is missing.\n' >&2; exit 78; }
  export LD_PRELOAD="$APPDIR/runtime32/libpulse.so.0${LD_PRELOAD:+:$LD_PRELOAD}"
fi
cd "$GAME_DIR/System"; exec "$APPDIR/runner/bin/wine" start /wait /unix "$GAME_DIR/System/HP.exe" -nodetect
"##;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn init_state_recovers_only_incomplete_state() {
        assert_eq!(init_state(true, true, false), InitState::Ready);
        assert_eq!(init_state(true, false, false), InitState::RecoverInterrupted);
        assert_eq!(init_state(false, false, true), InitState::Busy);
    }
    #[test] fn selects_expected_flatpak_driver_id() {
        let root = std::env::temp_dir().join(format!("hp1-driver-{}", std::process::id()));
        fs::create_dir_all(root.join("org.freedesktop.Platform.GL32.nvidia-580-178-04")).unwrap();
        assert!(driver_candidate_from_flatpak_root(&root, Some("580.178.04")).is_some());
        assert!(driver_candidate_from_flatpak_root(&root, None).is_none());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn launcher_seeds_complete_defaults_and_releases_init_lock_before_game() {
        assert!(APP_RUN.contains("cp \"$GAME_DIR/System/Default.ini\" \"$dir/HP.ini.tmp\""));
        assert!(APP_RUN.contains("cp \"$GAME_DIR/System/DefUser.ini\" \"$dir/User.ini.tmp\""));
        let unlock = APP_RUN.find("rm -rf \"$LOCK_DIR\"\ntrap - EXIT HUP INT TERM").unwrap();
        let launch = APP_RUN.find("cd \"$GAME_DIR/System\"; exec \"$APPDIR/runner/bin/wine\"").unwrap();
        assert!(unlock < launch);
    }
    #[test] fn launcher_embeds_selected_profile_and_repairs_software_renderer() {
        let launcher = APP_RUN.replace("__DEFAULT_PROFILE__", DisplayProfile::Hd720.id());
        assert!(launcher.contains("BUILDER_DEFAULT_PROFILE=720p"));
        assert!(launcher.contains("profile=${profile:-$BUILDER_DEFAULT_PROFILE}"));
        assert!(launcher.contains("GameRenderDevice=D3DDrv.D3DRenderDevice"));
        assert!(launcher.contains("ForceSoftware=0"));
        assert!(!launcher.contains("__DEFAULT_PROFILE__"));
        let profile = launcher.split("apply_profile() {").nth(1).unwrap().split("\nif [ ! -f").next().unwrap();
        assert!(profile.contains("s/^Reconfig=.*/Reconfig=0/"));
    }
    #[test] fn launcher_suppresses_optional_mono_throughout_prefix_setup() {
        assert!(APP_RUN.contains("export WINEDLLPATH=\"$APPDIR/runner/lib/wine:$APPDIR/runner/lib32/wine\" FONTCONFIG_PATH=\"$APPDIR/runtime32/etc/fonts\" WINEDLLOVERRIDES=\"winemenubuilder=;mscoree=\""));
        assert!(APP_RUN.contains("setup_env; \"$APPDIR/runner/bin/wineboot\" -u"));
    }

    #[test] fn launcher_repairs_only_unset_wine_audio_driver_when_pulse_is_available() {
        assert!(APP_RUN.contains("pulse/native"));
        assert!(APP_RUN.contains("reg query 'HKCU\\Software\\Wine\\Drivers' /v Audio"));
        assert!(APP_RUN.contains("reg add 'HKCU\\Software\\Wine\\Drivers' /v Audio /t REG_SZ /d pulse /f"));
        assert!(APP_RUN.contains("! \"$APPDIR/runner/bin/wine\" reg query"));
    }
    #[test] fn launcher_skips_renderer_wizard_after_first_run() {
        assert!(APP_RUN.contains("s/^Reconfig=.*/Reconfig=0/"));
        assert!(APP_RUN.contains("\"$GAME_DIR/System/HP.exe\" -nodetect"));
    }
    #[test] fn launcher_pairs_bundled_pulse_client_with_common_at_game_start() {
        assert!(APP_RUN.contains("export LD_PRELOAD=\"$APPDIR/runtime32/libpulse.so.0${LD_PRELOAD:+:$LD_PRELOAD}\""));
        assert!(APP_RUN.contains("cd \"$GAME_DIR/System\"; exec \"$APPDIR/runner/bin/wine\""));
    }

    #[test] fn generated_runtime_uses_host_glibc_but_keeps_other_libraries() {
        let root = std::env::temp_dir().join(format!("hp1-host-glibc-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for name in ["ld-linux.so.2", "libc.so.6", "libm.so.6", "libvulkan.so.1"] {
            fs::write(root.join(name), name).unwrap();
        }
        remove_bundled_glibc(&root).unwrap();
        for name in ["ld-linux.so.2", "libc.so.6", "libm.so.6"] {
            assert!(!root.join(name).exists(), "{name} was retained");
        }
        assert!(root.join("libvulkan.so.1").exists());
        assert!(APP_RUN.contains("WINELOADER=\"$APPDIR/runner/bin/wine\""));
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn launcher_matches_only_the_active_gpu_vendor() {
        assert!(Command::new("sh").args(["-n", "-c", APP_RUN]).status().unwrap().success());
        let start = APP_RUN.find("matches_icd() {").unwrap();
        let end = start + APP_RUN[start..].find("\nfind_icd() {").unwrap();
        let matcher = &APP_RUN[start..end];
        for (family, manifest, expected) in [
            ("nvidia", "nvidia_icd.json", true),
            ("nvidia", "asahi_icd.i686.json", false),
            ("nvidia", "radeon_icd.i686.json", false),
            ("amd", "radeon_icd.i686.json", true),
            ("amd", "nvidia_icd.json", false),
            ("intel", "intel_icd.i686.json", true),
            ("intel", "lvp_icd.i686.json", false),
        ] {
            let script = format!("GPU_FAMILY={family}\n{matcher}\nmatches_icd {manifest}\n");
            let accepted = Command::new("sh").args(["-c", &script]).status().unwrap().success();
            assert_eq!(accepted, expected, "{family} / {manifest}");
        }
    }
}
