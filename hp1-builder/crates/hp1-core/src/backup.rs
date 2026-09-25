use crate::{sha256_file, CoreError};
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates an independently named, verified backup without touching live data.
pub fn backup_saves_and_settings(output: &Path) -> Result<PathBuf, CoreError> {
    let home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
    }).ok_or_else(|| CoreError::InvalidInput("Cannot locate the XDG data folder.".into()))?;
    backup_from(&home.join("hp1-magipack-private"), output)
}

fn backup_from(data: &Path, output: &Path) -> Result<PathBuf, CoreError> {
    if !output.is_absolute() {
        return Err(CoreError::InvalidInput("Choose a full, absolute backup destination.".into()));
    }
    let source_root = fs::canonicalize(data).map_err(|_| CoreError::InvalidInput(
        "No local HP1 game data was found. Launch the game at least once first.".into()
    ))?;
    let documents = data.join("prefix/drive_c/users/steamuser/Documents/Harry Potter");
    let documents_root = fs::canonicalize(&documents).map_err(|_| CoreError::InvalidInput(
        "No HP1 saves or settings were found. Launch the game at least once first.".into()
    ))?;
    if !documents_root.starts_with(&source_root) {
        return Err(CoreError::InvalidInput("The game settings path escapes its data folder.".into()));
    }
    fs::create_dir_all(output)?;
    let output_root = fs::canonicalize(output)?;
    if output_root.starts_with(&source_root) {
        return Err(CoreError::InvalidInput("Choose a destination outside the live game data folder.".into()));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|error| CoreError::Io(error.to_string()))?;
    let staging = output_root.join(format!(
        ".HP1-backup-{}-{}-{}.partial", now.as_secs(), now.subsec_nanos(), std::process::id()
    ));
    fs::create_dir(&staging)?;
    let result = (|| {
        let mut manifest = Vec::new();
        copy_save_tree(&documents.join("Save"), &staging.join("Save"), &source_root, &staging, &mut manifest)?;
        for (source, relative) in [
            (documents.join("HP.ini"), "Settings/HP.ini"),
            (documents.join("User.ini"), "Settings/User.ini"),
            (documents.join("Detected.ini"), "Settings/Detected.ini"),
            (data.join("launch-profile"), "Settings/launch-profile"),
            (data.join("builder-default-profile"), "Settings/builder-default-profile"),
            (data.join("game/System/dgVoodoo.conf"), "Settings/dgVoodoo.conf"),
            (data.join("prefix/user.reg"), "Settings/Wine/user.reg"),
            (data.join("prefix/system.reg"), "Settings/Wine/system.reg"),
        ] {
            copy_optional(&source, &staging.join(relative), &source_root, &staging, &mut manifest)?;
        }
        if manifest.is_empty() {
            return Err(CoreError::InvalidInput("No save or settings files were found to back up.".into()));
        }
        let mut index = File::create(staging.join("MANIFEST.txt"))?;
        writeln!(index, "HP1 Builder local backup; created at Unix time {}", now.as_secs())?;
        writeln!(index, "Close the game before backing up to avoid changing files mid-copy.")?;
        writeln!(index, "Manual restore only; no live files were changed by this backup.")?;
        writeln!(index, "SHA-256  Relative path")?;
        for (hash, relative) in manifest {
            writeln!(index, "{hash}  {}", relative.display())?;
        }
        index.sync_all()?;
        let final_path = (0..1000).map(|suffix| {
            let suffix = if suffix == 0 { String::new() } else { format!("-{suffix}") };
            output_root.join(format!("HP1-backup-{}{}", now.as_secs(), suffix))
        }).find(|path| !path.exists()).ok_or_else(|| CoreError::Io(
            "Could not find an unused backup folder name.".into()
        ))?;
        fs::rename(&staging, &final_path)?;
        Ok(final_path)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&staging); }
    result
}

fn copy_save_tree(
    source: &Path, destination: &Path, root: &Path, staging: &Path,
    manifest: &mut Vec<(String, PathBuf)>
) -> Result<(), CoreError> {
    let metadata = match fs::symlink_metadata(source) {
        Ok(value) => value,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(CoreError::InvalidInput("Save data contains an unsupported link or file type.".into()));
    }
    ensure_within(source, root)?;
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child = entry.path();
        let target = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&child)?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            copy_save_tree(&child, &target, root, staging, manifest)?;
        } else {
            copy_file(&child, &target, root, staging, manifest)?;
        }
    }
    Ok(())
}

fn copy_optional(
    source: &Path, destination: &Path, root: &Path, staging: &Path,
    manifest: &mut Vec<(String, PathBuf)>
) -> Result<(), CoreError> {
    match fs::symlink_metadata(source) {
        Ok(_) => copy_file(source, destination, root, staging, manifest),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn copy_file(
    source: &Path, destination: &Path, root: &Path, staging: &Path,
    manifest: &mut Vec<(String, PathBuf)>
) -> Result<(), CoreError> {
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(CoreError::InvalidInput(format!("Not a regular backup source: {}", source.display())));
    }
    ensure_within(source, root)?;
    if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
    fs::copy(source, destination)?;
    let hash = sha256_file(source)?;
    if hash != sha256_file(destination)? {
        return Err(CoreError::Io(format!("Backup verification failed for {}", source.display())));
    }
    let relative = destination.strip_prefix(staging)
        .map_err(|error| CoreError::Io(error.to_string()))?.to_path_buf();
    manifest.push((hash, relative));
    Ok(())
}

fn ensure_within(path: &Path, root: &Path) -> Result<(), CoreError> {
    if !fs::canonicalize(path)?.starts_with(root) {
        return Err(CoreError::InvalidInput(format!("Backup source escapes game data: {}", path.display())));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("hp1-backup-test-{}-{stamp}", std::process::id()));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn saves_settings_and_repeat_backups_do_not_overwrite() {
        let root = fixture();
        let data = root.join("data");
        let docs = data.join("prefix/drive_c/users/steamuser/Documents/Harry Potter");
        fs::create_dir_all(docs.join("Save")).unwrap();
        fs::write(docs.join("Save/Save0.usa"), b"saved game").unwrap();
        fs::write(docs.join("User.ini"), b"controls").unwrap();
        fs::write(docs.join("HP.ini"), b"resolution").unwrap();
        let destination = root.join("output");
        let first = backup_from(&data, &destination).unwrap();
        let second = backup_from(&data, &destination).unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(first.join("Save/Save0.usa")).unwrap(), b"saved game");
        assert_eq!(fs::read(first.join("Settings/User.ini")).unwrap(), b"controls");
        assert!(first.join("MANIFEST.txt").is_file());
        assert_eq!(fs::read(docs.join("Save/Save0.usa")).unwrap(), b"saved game");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn refuses_destination_inside_live_data() {
        let root = fixture();
        let data = root.join("data");
        fs::create_dir_all(data.join("prefix/drive_c/users/steamuser/Documents/Harry Potter")).unwrap();
        assert!(backup_from(&data, &data.join("backup")).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn rejects_save_symlinks_without_a_completed_backup() {
        let root = fixture();
        let data = root.join("data");
        let docs = data.join("prefix/drive_c/users/steamuser/Documents/Harry Potter");
        fs::create_dir_all(docs.join("Save")).unwrap();
        fs::write(root.join("outside"), b"private").unwrap();
        std::os::unix::fs::symlink(root.join("outside"), docs.join("Save/linked")).unwrap();
        let destination = root.join("output");
        assert!(backup_from(&data, &destination).is_err());
        assert_eq!(fs::read_dir(destination).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}
