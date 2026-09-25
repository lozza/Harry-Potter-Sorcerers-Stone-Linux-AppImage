use hp1_core::{load_profiles, ZipEntry, ZipInspection};
use std::path::Path;

#[test]
fn both_exact_zip_layouts_match_only_their_own_profile() {
    let profile_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles");
    let profiles = load_profiles(&profile_dir).expect("load bundled ZIP profiles");
    assert_eq!(profiles.len(), 2);

    let nested = profiles
        .iter()
        .find(|profile| profile.id == "sorcerers-stone-magipack-repack-v3-en")
        .expect("nested ZIP profile");
    let flat = profiles
        .iter()
        .find(|profile| profile.id == "sorcerers-stone-flat-repack-v3-en")
        .expect("flat ZIP profile");

    assert_eq!(nested.installer_sha256, flat.installer_sha256);
    assert_eq!(nested.data_sha256, flat.data_sha256);
    assert_eq!(nested.data_size, flat.data_size);
    assert_ne!(nested.input_sha256, flat.input_sha256);
    assert_ne!(nested.installer_path, flat.installer_path);

    for (expected, other) in [(nested, flat), (flat, nested)] {
        assert!(expected.enabled);
        assert_eq!(
            expected.required_zip_paths,
            vec![expected.installer_path.clone(), expected.data_path.clone()]
        );
        let archive = ZipInspection {
            sha256: expected.input_sha256.clone(),
            size: expected.input_size,
            entries: vec![
                ZipEntry {
                    path: expected.installer_path.clone(),
                    size: 326_057,
                    encrypted: false,
                },
                ZipEntry {
                    path: expected.data_path.clone(),
                    size: expected.data_size,
                    encrypted: false,
                },
            ],
        };
        assert!(expected.matches(&archive));
        assert!(!other.matches(&archive));

        let mut changed = archive.clone();
        changed.sha256 = "0".repeat(64);
        assert!(!expected.matches(&changed));

        changed = archive;
        changed.entries[0].path = "other/setup.exe".into();
        assert!(!expected.matches(&changed));
    }
}
