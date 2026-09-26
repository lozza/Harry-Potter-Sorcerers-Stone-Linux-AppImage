slint::include_modules!();
use hp1_core::{backup_saves_and_settings, build, BuildEvent, BuildRequest, BuildStage, DisplayProfile, EventSink};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROJECT_URL: &str = "https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage";

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let browse_zip_weak = ui.as_weak();
    ui.on_browse_zip(move || browse(browse_zip_weak.clone(), BrowseKind::Zip));
    let browse_output_weak = ui.as_weak();
    ui.on_browse_output(move || browse(browse_output_weak.clone(), BrowseKind::Output));
    let project_weak = ui.as_weak();
    ui.on_open_project(move || {
        if let Err(error) = Command::new("xdg-open").arg(PROJECT_URL).spawn() {
            if let Some(window) = project_weak.upgrade() {
                window.set_status_text(format!("Could not open the project page: {error}").into());
            }
        }
    });
    let backup_weak = ui.as_weak();
    ui.on_request_backup(move |output| backup(backup_weak.clone(), output.to_string()));
    let weak = ui.as_weak();
    ui.on_request_build(move |zip, output, profile_index| {
        let zip = zip.trim().to_string();
        let output = output.trim().to_string();
        let Some(window) = weak.upgrade() else { return };
        if window.get_building() || window.get_choosing() || window.get_backing_up() { return; }
        let default_profile = match profile_index {
            0 => DisplayProfile::Hd720, 1 => DisplayProfile::FullHd1080,
            2 => DisplayProfile::SteamDeck, 3 => DisplayProfile::Ultrawide,
            4 => DisplayProfile::Windowed,
            _ => { window.set_status_text("Choose a game resolution.".into()); return; }
        };
        if zip.is_empty() || output.is_empty() {
            window.set_build_failed(true);
            window.set_status_text("Choose both a game ZIP and an output folder.".into());
            return;
        }
        let output_path = PathBuf::from(&output);
        if !output_path.is_absolute() {
            window.set_build_failed(true);
            window.set_status_text("Use a full, absolute output-folder path.".into());
            return;
        }
        window.set_backup_finished(false);
        window.set_backup_failed(false);
        window.set_building(true);
        window.set_build_finished(false);
        window.set_build_failed(false);
        window.set_progress_value(0.0);
        window.set_status_text("Starting private build…".into());
        window.set_diagnostic_text("Starting private build…".into());
        let thread_weak = weak.clone();
        std::thread::spawn(move || {
            let log = fs::create_dir_all(&output_path)
                .and_then(|_| File::create(output_path.join("hp1-builder.log")));
            let mut sink = GuiSink { ui: thread_weak.clone(), log: log.ok(), lines: String::new() };
            if sink.log.is_none() {
                sink.note("Could not create hp1-builder.log in the output folder.");
            }
            sink.note(&format!("Selected game display profile: {}", default_profile.id()));
            if default_profile == DisplayProfile::SteamDeck {
                sink.note("Warning: 1280x800 is experimental on Steam Deck; a recurring Direct3D picker can block Gaming Mode. The verified workaround is 1280x720.");
            }
            let profiles_dir = std::env::var_os("HP1_PROFILES_DIR")
                .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("profiles"));
            let artifact_path = output_path.join("Harry-Potter-Sorcerers-Stone-private.AppImage");
            let request = BuildRequest {
                source_zip: PathBuf::from(zip), output: output_path,
                profiles_dir, dry_run: false, keep_workdir: false, default_profile,
            };
            let result = build(&request, &mut sink);
            let (status, progress, failed) = match result {
                Ok(()) => (format!("Your game AppImage is ready: {}", artifact_path.display()), 1.0, false),
                Err(error) => {
                    sink.note(&format!("ERROR: {error}"));
                    (format!("Build failed: {error}"), 0.0, true)
                }
            };
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(window) = thread_weak.upgrade() {
                    window.set_status_text(status.into());
                    window.set_progress_value(progress);
                    window.set_build_finished(!failed);
                    window.set_build_failed(failed);
                    window.set_building(false);
                }
            });
        });
    });
    ui.run()
}

fn backup(weak: slint::Weak<MainWindow>, output: String) {
    let Some(window) = weak.upgrade() else { return };
    if window.get_building() || window.get_choosing() || window.get_backing_up() { return; }
    let output = output.trim();
    if output.is_empty() {
        window.set_backup_failed(true);
        window.set_backup_finished(false);
        window.set_status_text("Choose an output folder for the backup. No ZIP is needed.".into());
        return;
    }
    let destination = PathBuf::from(output);
    if !destination.is_absolute() {
        window.set_backup_failed(true);
        window.set_backup_finished(false);
        window.set_status_text("Use a full, absolute backup-folder path.".into());
        return;
    }
    window.set_backup_failed(false);
    window.set_backup_finished(false);
    window.set_build_finished(false);
    window.set_build_failed(false);
    window.set_backing_up(true);
    window.set_status_text("Copying and verifying saves and settings…".into());
    let thread_weak = weak.clone();
    std::thread::spawn(move || {
        let result = backup_saves_and_settings(&destination);
        let _ = slint::invoke_from_event_loop(move || {
            let Some(window) = thread_weak.upgrade() else { return };
            window.set_backing_up(false);
            match result {
                Ok(path) => {
                    let message = format!("Backup complete: {}", path.display());
                    window.set_status_text(message.clone().into());
                    window.set_diagnostic_text(format!("{message}\nSaves and settings were copied and verified. Live game data was not changed.\nThe backup includes a SHA-256 manifest.").into());
                    window.set_backup_finished(true);
                }
                Err(error) => {
                    let message = format!("Backup failed: {error}");
                    window.set_status_text(message.clone().into());
                    window.set_diagnostic_text(message.into());
                    window.set_backup_failed(true);
                }
            }
        });
    });
}

#[derive(Clone, Copy)]
enum BrowseKind { Zip, Output }

fn browse(weak: slint::Weak<MainWindow>, kind: BrowseKind) {
    let Some(window) = weak.upgrade() else { return };
    if window.get_building() || window.get_choosing() || window.get_backing_up() { return; }
    window.set_choosing(true);
    window.set_status_text("Opening file picker…".into());
    std::thread::spawn(move || {
        let result = pick_path(kind);
        let _ = slint::invoke_from_event_loop(move || {
            let Some(window) = weak.upgrade() else { return };
            window.set_choosing(false);
            match result {
                Ok(Some(path)) => {
                    let display = path.to_string_lossy().to_string();
                    match kind {
                        BrowseKind::Zip => window.set_zip_path(display.into()),
                        BrowseKind::Output => window.set_output_path(display.into()),
                    }
                    window.set_status_text("Selection ready.".into());
                }
                Ok(None) => window.set_status_text("File selection cancelled.".into()),
                Err(error) => window.set_status_text(
                    format!("File picker unavailable: {error}. You can paste a full path.").into()
                ),
            }
        });
    });
}

fn pick_path(kind: BrowseKind) -> Result<Option<PathBuf>, String> {
    let host_spawn = Path::new("/run/host/usr/bin/flatpak-spawn");
    let mut command = if host_spawn.is_file() {
        let mut command = Command::new(host_spawn);
        command.args(["--host", "kdialog"]);
        command
    } else {
        Command::new("kdialog")
    };
    let start = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    match kind {
        BrowseKind::Zip => {
            command.args(["--getopenfilename", &start, "*.zip|ZIP archives"]);
        }
        BrowseKind::Output => {
            command.args(["--getexistingdirectory", &start]);
        }
    }
    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.code() == Some(1) { return Ok(None); }
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let path = String::from_utf8(output.stdout).map_err(|_| "Selected path is not UTF-8".to_string())?;
    let path = path.trim();
    if path.is_empty() { Ok(None) } else { Ok(Some(PathBuf::from(path))) }
}

struct GuiSink {
    ui: slint::Weak<MainWindow>,
    log: Option<File>,
    lines: String,
}

impl GuiSink {
    fn note(&mut self, line: &str) {
        if let Some(log) = &mut self.log { let _ = writeln!(log, "{line}"); }
        self.lines.push_str(line);
        self.lines.push('\n');
        let lines = self.lines.clone();
        let status = line.to_string();
        let weak = self.ui.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = weak.upgrade() {
                window.set_diagnostic_text(lines.into());
                window.set_status_text(status.into());
            }
        });
    }
}

impl EventSink for GuiSink {
    fn emit(&mut self, event: BuildEvent) {
        let line = if event.message.is_empty() {
            format!("[{}] {}", event.stage.as_str(), event.kind)
        } else {
            format!("[{}] {}: {}", event.stage.as_str(), event.kind, event.message)
        };
        self.note(&line);
        let progress = match (event.stage, event.kind) {
            (BuildStage::ValidateArguments, "completed") => 0.05,
            (BuildStage::IdentifyEdition, "completed") => 0.15,
            (BuildStage::DownloadComponents, "started") => 0.17,
            (BuildStage::DownloadComponents, "completed") => 0.28,
            (BuildStage::ExtractInstaller, "completed") => 0.42,
            (BuildStage::PreparePayload, "completed") => 0.70,
            (BuildStage::AssembleAppDir, "completed") => 0.78,
            (BuildStage::PackageAppImage, "completed") => 0.97,
            (BuildStage::ValidateOutput, "completed") => 1.0,
            _ => return,
        };
        let weak = self.ui.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = weak.upgrade() { window.set_progress_value(progress); }
        });
    }
}
