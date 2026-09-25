# HP1 Builder

HP1 Builder is an in-progress, local-only Linux tool intended to build a
private AppImage from either of two verified ZIP profiles for
**Harry Potter and the Sorcerer’s Stone**. **Harry Potter and the
Philosopher’s Stone** is represented in
the title and profile architecture, but it is not separately supported.

This repository contains no commercial game data, input ZIP, patch,
compatibility file, downloader, or link for finding one. You must supply the
verified ZIP locally. The builder never downloads game inputs and performs its
work locally.

The official project and update location is
[lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage).
A link to it is also shown inside the builder. This identifies the project; it
does not prevent others from redistributing or misrepresenting copies.

## Current status

The headless private-integration path supports two exact ZIP layouts: one
with a parent directory and one with the installer files at the archive root.
Both contain the same verified installer payload. It validates the ZIP by
size, SHA-256, member hashes, and safe paths; extracts its Inno payload
without running the installer; creates an
XDG-writable Wine prefix; and builds a private game AppImage. It does not use
ISO media, a separately supplied compatibility/no-CD file, Bottles, or system
Wine.

This is a **beta candidate, not yet redistribution-ready**. The private build requires audited local
copies of the runner/runtime, innoextract, AppImage packager, and AppImage
runtime through explicit environment variables. Their provenance and licences
have not yet been cleared for a public artifact. GPU drivers are discovered on
the host at launch and are never bundled.

See [third-party notices](docs/THIRD_PARTY_NOTICES.md) for the remaining
public-binary checks. Machine-specific audit logs and milestone notes stay
outside the published source tree.

## Private workflow

With either verified v1 ZIP and the private integration tools already available:

1. Launch the builder AppImage.
2. Use **Browse** to choose the verified game ZIP and an empty output folder.
   The path fields remain editable if the host file picker is unavailable.
3. Choose the game's starting resolution. 1280×720 fullscreen is the
   recommended default; other supported modes are listed in the selector.
4. Click **Build AppImage**. A green **BUILD COMPLETE** message shows the
   output path; errors appear in red.
5. Launch the generated game AppImage separately; the builder never starts it.

The **Back up saves & settings** button is independent of building: close the
game, choose a backup destination in the output-folder field, and click it.
No ZIP is needed. It creates a new numbered/timestamped folder containing save
slots, game INI files, the selected launch profile, dgVoodoo configuration,
and the Wine user/system registry files when present. A SHA-256 manifest records
every copied file. The button never changes live saves, never overwrites an
existing backup, and does not restore files automatically. Backups are local
personal game data; do not upload them to this repository or a bug report.

The GUI saves `hp1-builder.log` in the output folder. Its Build button is
disabled during a build. Cancel is not offered until external extraction and
packaging can be stopped safely. The selected resolution becomes the
generated game's default; later launch-profile choices remain changeable.

The builder verifies the ZIP by exact metadata, hashes, and layout rather than
its filename. Unsupported archives are rejected before extraction.

## Command line

The current CLI is useful for exercising the safe plumbing:

```sh
hp1-builder inspect --zip "/path/to/game.zip"
hp1-builder build --zip "/path/to/game.zip" \
  --output "/path/to/output-folder" --profile 720p --dry-run
hp1-builder check
```

For the private integration route, `build` also needs these local inputs:
`HP1_INNOEXTRACT`, `HP1_PRIVATE_REFERENCE_APPDIR`, `HP1_APPIMAGETOOL`, and
`HP1_APPIMAGE_RUNTIME`; `HP1_PACKAGER_PATH` is optional when the packager
needs an isolated utility path. The builder rejects absent or mismatched
private runtime inputs rather than falling back to a system installation.

## Implemented private behaviour

The private AppImage keeps game data, settings, saves, logs, shader cache and
Wine prefix under XDG data, outside its read-only mount. Its launcher supports
720p, 1080p, 1280×800 Steam Deck, ultrawide, windowed, custom resolutions, and
the supplied resolution selector. Audio and hardware Direct3D have passed
private tests on Bazzite and Steam Deck. A user confirmed gameplay, audio,
and working controls in both Steam Deck Desktop and Gaming Modes. The new
backup button has automated tests but still needs hands-on Deck validation.

The future AppImage should support `--appimage-extract-and-run` for systems
without FUSE. No statement is made yet about distribution, GPU support,
Steam Deck, or any other Linux distribution.

## Known beta limitations

- At 1280×800, thin lines can appear on title/loading screens on both Bazzite
  and Steam Deck. Dumbledore’s glasses can show a thick shadow at that profile;
  Harry’s glasses look normal. The user reports these as non-game-breaking.
  The 720p and ultrawide profiles have not shown the same glasses defect in
  their reported tests. The cause is not yet isolated.
- Steam’s standard WASD-and-mouse layout sends WASD from the Deck’s left stick,
  but this game currently assigns movement to the keyboard arrow keys instead.
  For the game’s non-Steam shortcut, map the left stick directions to the four
  arrow keys in its per-game Steam Input layout. The user confirmed that the
  arrow keys work and subsequently confirmed controls work in Gaming Mode;
  a native gamepad layout without this mapping is not verified.
- Some builder text appears uneven or jagged at the tested desktop scale. This
  is a GUI presentation issue; its exact renderer/font cause is not proven.
- Testing covers Bazzite and Steam Deck only. Other Linux distributions,
  GPU/driver combinations and older library versions remain unverified.

## Disk space and diagnostics

The initial implementation uses a private build workspace alongside the chosen
output folder and atomically promotes only a complete AppImage. The launcher's
first-run game/prefix setup uses a lock, staging directories, and recovery of
interrupted initialization. Disk-space preflight and redacted support reports
remain future work.

## Dependency and licence status

The ZIP-builder source is licensed under [GNU GPLv3](LICENSE)
(`GPL-3.0-only`). This does not grant redistribution rights for commercial
game files or third-party binaries.

The source uses Rust and a pinned Slint GUI dependency. Private artifacts copy
the locally audited runner/runtime and two DXVK DLLs, and use local
innoextract/AppImage tooling. Before any release, this project must record the
version, source URL, licence, attribution/source obligations, and redistribution
suitability of every bundled component and include required notices.
