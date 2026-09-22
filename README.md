# Harry Potter and the Sorcerer's Stone — Linux AppImage builder

This repository contains the launcher and build recipe for creating a self-contained Linux AppImage from a legally owned copy of the game.

The game files, Wine prefix, and runtime are intentionally not included in this repository.

## Build

Requirements:

- A working local copy of the game
- A configured 32-bit Bottles/Wine prefix
- A Wine runner and 32-bit compatibility runtime
- `appimagetool`

Run:

```sh
./build-appimage.sh "/path/to/Harry Potter and the Sorcerer's Stone"
```

Optional paths can be supplied with environment variables:

```sh
BOTTLE_DIR=/path/to/bottle \
WINE_RUNNER=/path/to/runner \
COMPAT_RUNTIME=/path/to/org.freedesktop.Platform.Compat.i386/files \
APPIMAGETOOL=/path/to/appimagetool \
./build-appimage.sh "/path/to/game" ./Harry-Potter-Sorcerers-Stone.AppImage
```

## Launcher profiles

Launching the AppImage without an argument opens the graphical settings chooser. The launcher also supports:

- `--ultrawide` — 2560×1080 fullscreen
- `--1080p` — 1920×1080 fullscreen
- `--720p` — 1280×720 fullscreen
- `--deck` — 1280×800 fullscreen
- `--windowed` — 1280×720 windowed
- `--auto` — choose a profile from the detected display shape when available

The launcher preserves Direct3D, controller support, audio, writable settings, and saves in the user's data directory.
