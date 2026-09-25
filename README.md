# Harry Potter and the Sorcerer's Stone — Linux AppImage builder

Unofficial community project; not affiliated with Warner Bros., Electronic Arts, or the original developers.

## Current status

The ZIP-only builder is a **private beta**, tested on Bazzite and Steam Deck. It is **not yet available as a public download or GitHub release**. The old `build-appimage.sh` in this repository is a separate, **legacy Bottles-based script**; it is not the new beta builder. Its original instructions are preserved in [Legacy Bottles builder](docs/LEGACY_BOTTLES_BUILDER.md).

**Do not install Bottles for the new ZIP-only builder.** It accepts one supported local game ZIP and an output folder, extracts the installer without executing it, and creates a private game AppImage with its own Wine runner. The builder does not download a game, and this repository must not contain the ZIP, game files, saves, or a generated game AppImage.

Two exact ZIP layouts have been verified: the original parent-folder MagiPack archive and a flat-layout archive containing the same installer files. The builder checks the complete archive profile, not just its filename. No download source for either archive is provided here.

## Verified private beta behavior

- A game built privately has launched on Bazzite and Steam Deck in Desktop and Gaming Modes with hardware Direct3D and audio. The Deck's movement controls need a per-game Steam Input layout that maps the left stick to arrow keys; the default WASD mapping does not move the character.
- The builder offers a starting resolution, including 1280×720, 1280×800 Deck, 1080p, ultrawide, and windowed options. 1280×720 is the best-verified visual baseline.
- Saves and settings are writable under `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`, not inside the AppImage or builder. Rebuilding the AppImage normally sees the same local saves.
- The new private beta includes a **Back up saves & settings** button. With the game closed, choose an output folder and use the button; it creates a new `HP1-backup-*` folder without overwriting live saves. It does not restore files automatically. Do not upload your backup.

## Known beta issues and test scope

- At 1280×800, thin lines can appear on title/loading screens. Dumbledore's glasses can show a thick shadow at that resolution. These were observed on Bazzite and Steam Deck; they have not been reported as game-breaking.
- Some builder text appears uneven at the tested desktop scale.
- Other Linux distributions, GPU/driver combinations, and older library versions have not been verified.

## Release gate

The private beta packages a Wine runner and a large 32-bit runtime. Their exact provenance, redistribution terms, notices, and source obligations have not been fully cleared. A public builder binary will not be posted until that review and the final hands-on checks are complete. A generated **game** AppImage contains commercial game content and will never be published here.

This page will be updated when a public beta is actually available. Until then, do not treat the legacy Bottles script below or any unofficial mirror as the ZIP-only builder.
