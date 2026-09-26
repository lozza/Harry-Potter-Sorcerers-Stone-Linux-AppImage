use crate::{sha256_file, BuildEvent, BuildStage, CoreError, EventSink};
use std::{env, fs::{self, File}, path::{Path, PathBuf}, process::{Command, Stdio}};
const SODA_URL: &str = "https://github.com/bottlesdevs/wine/releases/download/soda-9.0-1/soda-9.0-1-x86_64.tar.xz";
const SODA_SHA: &str = "c38fe0ad3c12a49b61ec1fcaea5c5d8da4a3d1afc5991befe2af6b125f014c28";
const DXVK_URL: &str = "https://github.com/bottlesdevs/components/releases/download/dxvk-2.7.1-6-fc848a4/dxvk-2.7.1-6-fc848a4.tar.gz";
const DXVK_SHA: &str = "96a78de1cbe2275c9325d8a69212d9f742d5e48b3507c6f3ad684d1a186c389f";
const PACKAGER_URL: &str = "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage";
const PACKAGER_SHA: &str = "a6d71e2b6cd66f8e8d16c37ad164658985e0cf5fcaa950c90a482890cb9d13e0";
const FLATPAK_REF: &str = "org.freedesktop.Platform.Compat.i386//25.08";
const FLATPAK_COMMIT: &str = "10c43710cbba7c67183615a06816d8b5a6ef4a079478c9484e9550a1d438241f";
pub struct OnlineComponents { pub runner: PathBuf, pub runtime32: PathBuf, pub dxgi: PathBuf, pub d3d11: PathBuf, pub appimagetool: PathBuf }
fn run(command: &mut Command, what: &str) -> Result<(), CoreError> {
    let status = command.status().map_err(|e| CoreError::Io(format!("Cannot start {what}: {e}")))?;
    if status.success() { Ok(()) } else { Err(CoreError::Unsupported(format!("{what} failed: {status}"))) }
}
fn cache_root() -> Result<PathBuf, CoreError> {
    let base = env::var_os("HP1_COMPONENT_CACHE").map(PathBuf::from)
        .or_else(|| env::var_os("XDG_CACHE_HOME").map(|p| PathBuf::from(p).join("hp1-builder/components")))
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".cache/hp1-builder/components")))
        .ok_or_else(|| CoreError::InvalidInput("HOME or XDG_CACHE_HOME is required for caching.".into()))?;
    fs::create_dir_all(&base)?; Ok(base)
}
fn fetch(cache: &Path, name: &str, url: &str, size: u64, hash: &str, sink: &mut dyn EventSink) -> Result<PathBuf, CoreError> {
    let output = cache.join(name);
    if output.is_file() && fs::metadata(&output)?.len() == size && sha256_file(&output)? == hash {
        sink.emit(BuildEvent::message(BuildStage::DownloadComponents, format!("Using verified cached {name}."))); return Ok(output);
    }
    if output.exists() { fs::remove_file(&output)?; }
    sink.emit(BuildEvent::message(BuildStage::DownloadComponents, format!("Downloading {name} ({:.1} MB) from {url}", size as f64 / 1_000_000.0)));
    let partial = cache.join(format!("{name}.{}.partial", std::process::id()));
    let result = (|| {
        run(Command::new("curl").args(["--fail", "--location", "--silent", "--show-error", "--retry", "2", "--proto", "=https", "--tlsv1.2", "--output"]).arg(&partial).arg(url), "component download")?;
        if fs::metadata(&partial)?.len() != size || sha256_file(&partial)? != hash {
            return Err(CoreError::Unsupported(format!("{name} failed the pinned size/SHA-256 check.")));
        }
        fs::rename(&partial, &output)?; Ok(output)
    })();
    if result.is_err() { let _ = fs::remove_file(partial); }
    result
}
fn extract_dll(cache: &Path, archive: &Path, name: &str, hash: &str) -> Result<PathBuf, CoreError> {
    let output = cache.join(name);
    if output.is_file() && sha256_file(&output)? == hash { return Ok(output); }
    let partial = cache.join(format!("{name}.{}.partial", std::process::id()));
    let result = (|| {
        let file = File::create(&partial)?;
        run(Command::new("tar").args(["-xzOf"]).arg(archive).arg(format!("dxvk-2.7.1-6-fc848a4/x32/{name}")).stdout(Stdio::from(file)), "DXVK extraction")?;
        if sha256_file(&partial)? != hash { return Err(CoreError::Unsupported(format!("{name} failed the pinned DLL SHA-256 check."))); }
        fs::rename(&partial, &output)?; Ok(output)
    })();
    if result.is_err() { let _ = fs::remove_file(partial); }
    result
}
fn runtime(scope: &str) -> Option<PathBuf> {
    let commit = Command::new("flatpak").args(["info", scope, "--show-commit", FLATPAK_REF]).output().ok()?;
    if !commit.status.success() || String::from_utf8_lossy(&commit.stdout).trim() != FLATPAK_COMMIT { return None; }
    let location = Command::new("flatpak").args(["info", scope, "--show-location", FLATPAK_REF]).output().ok()?;
    if !location.status.success() { return None; }
    let path = PathBuf::from(String::from_utf8_lossy(&location.stdout).trim()).join("files");
    if sha256_file(&path.join("ld-linux.so.2")).ok()?.as_str() != "59e07d873c208c2e11518e5b20dc867adad0c544730c4cf193ee03160873369e" { return None; }
    if sha256_file(&path.join("libc.so.6")).ok()?.as_str() != "15530c2512962b355a9fe72df566615db585acc6fcbe2a7597fc066dab6c52bc" { return None; }
    Some(path)
}
pub fn acquire(sink: &mut dyn EventSink) -> Result<OnlineComponents, CoreError> {
    sink.emit(BuildEvent::started(BuildStage::DownloadComponents));
    sink.emit(BuildEvent::message(BuildStage::DownloadComponents, "Your game ZIP stays local. First build downloads verified Soda Wine (~64.6 MB), DXVK (~15.4 MB), AppImage packaging tool (~15.1 MB), and if missing a Flatpak 32-bit runtime (~130 MB). The components are cached."));
    let cache = cache_root()?;
    let soda = fetch(&cache, "soda-9.0-1-x86_64.tar.xz", SODA_URL, 64_564_696, SODA_SHA, sink)?;
    let dxvk = fetch(&cache, "dxvk-2.7.1-6-fc848a4.tar.gz", DXVK_URL, 15_371_492, DXVK_SHA, sink)?;
    let appimagetool = fetch(&cache, "appimagetool-x86_64.AppImage", PACKAGER_URL, 15_092_216, PACKAGER_SHA, sink)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; let mut permissions = fs::metadata(&appimagetool)?.permissions(); permissions.set_mode(0o755); fs::set_permissions(&appimagetool, permissions)?; }
    let runner = cache.join("soda-9.0-1-x86_64");
    if !runner.join("bin/wine").is_file() || !runner.join("LICENSE").is_file() {
        let stage = cache.join(format!(".runner-{}", std::process::id()));
        fs::create_dir(&stage)?;
        let result = run(Command::new("tar").arg("-xJf").arg(&soda).arg("-C").arg(&stage), "Soda extraction")
            .and_then(|_| { if runner.exists() { fs::remove_dir_all(&runner)?; } fs::rename(stage.join("soda-9.0-1-x86_64"), &runner)?; Ok(()) });
        let _ = fs::remove_dir_all(&stage);
        result?;
    }
    let dxgi = extract_dll(&cache, &dxvk, "dxgi.dll", "82a9d8a2cfc5589c91fc0144579e8793d2064071a6ec5edcba926b6b19c93dfd")?;
    let d3d11 = extract_dll(&cache, &dxvk, "d3d11.dll", "b9280d6183ed00873a68e2fd190688f6ced92cfc9e9478be87b22cb4be7c8946")?;
    let runtime32 = if let Some(path) = runtime("--user").or_else(|| runtime("--system")) { path } else {
        sink.emit(BuildEvent::message(BuildStage::DownloadComponents, "Installing pinned Flatpak 32-bit runtime (~130 MB)."));
        run(Command::new("flatpak").args(["install", "--user", "--noninteractive", "--no-related", "--no-deps", "flathub", FLATPAK_REF]), "Flatpak runtime install")?;
        run(Command::new("flatpak").args(["update", "--user", "--noninteractive", "--no-related", "--no-deps", "--commit", FLATPAK_COMMIT, FLATPAK_REF]), "Flatpak runtime pin")?;
        runtime("--user").ok_or_else(|| CoreError::Unsupported("Exact pinned 32-bit Flatpak runtime is unavailable.".into()))?
    };
    sink.emit(BuildEvent::completed(BuildStage::DownloadComponents));
    Ok(OnlineComponents { runner, runtime32, dxgi, d3d11, appimagetool })
}
