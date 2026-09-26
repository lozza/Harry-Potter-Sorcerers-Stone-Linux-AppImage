# HP1 Builder

HP1 Builder is a ZIP-only Linux tool that builds a private game AppImage
from either of two verified ZIP layouts for
**Harry Potter and the Sorcerer’s Stone**. **Harry Potter and the
Philosopher’s Stone** is represented in
the title and profile architecture, but it is not separately supported.

This repository contains no commercial game data, input ZIP, no-CD patch,
or link for finding one. You supply the verified ZIP locally. The builder
never uploads or downloads game files. On the first build it downloads only
free compatibility components from pinned upstream releases, verifies their
sizes and SHA-256 hashes, then caches them for later builds.

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

This is a **beta candidate, not yet redistribution-ready**. Its small builder
AppImage bundles the GUI, ZIP extractor, Inno extractor and AppImage packaging
tools except the AppImage packager; it does not bundle a Wine runner or 32-bit
runtime. A first build fetches Soda Wine (~64.6 MB), DXVK (~15.4 MB), and the
AppImage packager (~15.1 MB). If the exact Flatpak Compat.i386
runtime is not already installed, Flatpak downloads about 130 MB more. Bottles
and system Wine are not required. GPU drivers are discovered on the host at
game launch and are never bundled. See the notice and provenance gate below.

See [third-party notices](docs/THIRD_PARTY_NOTICES.md) for the remaining
public-binary checks. Machine-specific audit logs and milestone notes stay
outside the published source tree.

## Private workflow

With either verified v1 ZIP:

1. Launch the builder AppImage.
2. Use **Browse** to choose the verified game ZIP and an empty output folder.
   The path fields remain editable if the host file picker is unavailable.
3. Choose the game's starting resolution. 1280×720 fullscreen is the
   recommended default; other supported modes are listed in the selector.
   If you have already launched an earlier build, its saved resolution remains
   in your XDG game data and takes precedence over the new build's default.
   Launch the game once with `--720p` or `--deck` to change that saved choice.
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

The packaged builder supplies its non-Wine tools itself. Source-tree CLI
builds need `HP1_INNOEXTRACT` and `HP1_APPIMAGE_RUNTIME` pointed at the
audited packaging tools. The packager is downloaded and hash-verified. The optional
`HP1_PRIVATE_REFERENCE_APPDIR` is a development-only override and is not set
by the online builder AppImage. `HP1_COMPONENT_CACHE` can select a private
component cache for testing. Output paths must be absolute.

## Implemented private behaviour

The private AppImage keeps game data, settings, saves, logs, shader cache and
Wine prefix under XDG data, outside its read-only mount. Its launcher supports
720p, 1080p, 1280×800 Steam Deck, ultrawide, windowed, custom resolutions, and
the supplied resolution selector. Audio and hardware Direct3D have passed
private tests on Bazzite and Steam Deck. A user confirmed gameplay, audio,
and working controls in both Steam Deck Desktop and Gaming Modes. The new
backup button has automated tests but still needs hands-on Deck validation.

The current private game AppImage works on tested x86_64 Bazzite and Steam
Deck systems. Other Linux distributions are untested. At 1280×800, title and
loading screens can show lines, and the Direct3D picker returned on repeated
Steam Deck launches. That picker blocks an unattended Gaming Mode launch at
this resolution. Two Deck launches at 1280×720 skipped it and had sound.
The generated game AppImage is for personal use and must not be uploaded.
Public builder publication is conditional on GUI checks and third-party
licence notices.

## Known beta limitations

- At 1280×800, thin lines can appear on title/loading screens on both Bazzite
  and Steam Deck. The Direct3D picker also recurred at this resolution on Deck;
  1280×720 skipped it twice. Dumbledore’s glasses can show a thick shadow at
  1280×800 and intermittently at 1280×720, while Harry’s glasses look normal.
  The user reports the visual defects as non-game-breaking. The picker is
  different: Gaming Mode cannot proceed past it without changing launch
  resolution or manually interacting in Desktop Mode. The root cause remains
  under investigation.
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
