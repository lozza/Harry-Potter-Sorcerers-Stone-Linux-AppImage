//! Safe, reusable domain logic for the private-input HP1 builder.
//! No game content or compatibility files are embedded here.

mod backup;
mod config;
mod events;
mod hash;
mod paths;
mod pipeline;
mod profile;
mod zip;

pub use backup::backup_saves_and_settings;
pub use config::{DisplayProfile, GeneratedConfiguration};
pub use events::{BuildEvent, BuildStage, EventSink};
pub use hash::sha256_file;
pub use paths::{safe_relative_path, SafePathError};
pub use pipeline::{driver_candidate_from_flatpak_root, init_state, InitState};
pub use profile::{load_profiles, EditionProfile, ProfileError};
pub use zip::{inspect_zip, ZipEntry, ZipInspection};

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub source_zip: PathBuf,
    pub output: PathBuf,
    pub profiles_dir: PathBuf,
    pub dry_run: bool,
    pub keep_workdir: bool,
    pub default_profile: DisplayProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    Io(String),
    InvalidInput(String),
    Unsupported(String),
    Profile(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::InvalidInput(message) | Self::Unsupported(message) | Self::Profile(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self { Self::Io(error.to_string()) }
}

/// Validates the single private ZIP input and emits machine-readable stage events.
///
pub fn validate_build(request: &BuildRequest, sink: &mut dyn EventSink) -> Result<EditionProfile, CoreError> {
    sink.emit(BuildEvent::started(BuildStage::ValidateArguments));
    validate_regular_file(&request.source_zip, "Game ZIP")?;
    if request.output.as_os_str().is_empty() {
        return Err(CoreError::InvalidInput("An output directory is required.".into()));
    }
    sink.emit(BuildEvent::completed(BuildStage::ValidateArguments));

    sink.emit(BuildEvent::started(BuildStage::IdentifyEdition));
    let inspection = inspect_zip(&request.source_zip)?;
    let profiles = load_profiles(&request.profiles_dir).map_err(|error| CoreError::Profile(error.to_string()))?;
    let profile = profiles.into_iter().find(|candidate| candidate.matches(&inspection))
        .ok_or_else(|| CoreError::Unsupported(format!(
            "Unsupported or ambiguous ZIP revision (SHA-256 {}, size {}). A filename is never used for identification.",
            inspection.sha256, inspection.size
        )))?;
    sink.emit(BuildEvent::message(BuildStage::IdentifyEdition, format!("Detected {} ({})", profile.edition_name, profile.region)));
    sink.emit(BuildEvent::completed(BuildStage::IdentifyEdition));

    Ok(profile)
}

pub fn build(request: &BuildRequest, sink: &mut dyn EventSink) -> Result<(), CoreError> {
    let profile = validate_build(request, sink)?;
    if request.dry_run {
        sink.emit(BuildEvent::message(BuildStage::Plan, "Dry run passed ZIP validation; no game data was extracted."));
        sink.emit(BuildEvent::completed(BuildStage::Plan));
        return Ok(());
    }
    if !profile.enabled { return Err(CoreError::Unsupported("The matched ZIP profile is disabled.".into())); }
    pipeline::build_private_appimage(request, &profile, sink)
}

fn validate_regular_file(path: &Path, label: &str) -> Result<(), CoreError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| CoreError::InvalidInput(format!("{label} was not found or cannot be read.")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CoreError::InvalidInput(format!("{label} must be a regular file, not a link or device.")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_non_regular_inputs() {
        let mut events = Vec::new();
        let request = BuildRequest { source_zip: PathBuf::from("."), output: PathBuf::from("out"), profiles_dir: PathBuf::from("profiles"), dry_run: true, keep_workdir: false, default_profile: DisplayProfile::Hd720 };
        assert!(validate_build(&request, &mut events).is_err());
    }
}
