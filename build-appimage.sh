#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
GAME_DIR=${1:-}
OUTPUT=${2:-Harry-Potter-Sorcerers-Stone.AppImage}
BOTTLE_DIR=${BOTTLE_DIR:-"$HOME/.var/app/com.usebottles.bottles/data/bottles/bottles/Harry-Potter"}
WINE_RUNNER=${WINE_RUNNER:-"$HOME/.var/app/com.usebottles.bottles/data/bottles/runners/soda-9.0-1"}
APPIMAGETOOL=${APPIMAGETOOL:-appimagetool}

if [ -z "$GAME_DIR" ] || [ ! -f "$GAME_DIR/System/HP.exe" ]; then
    printf 'Usage: %s /path/to/game [output.AppImage]\n' "$0" >&2
    exit 2
fi
[ -f "$BOTTLE_DIR/system.reg" ] || { printf 'Missing Wine prefix: %s\n' "$BOTTLE_DIR" >&2; exit 1; }
[ -x "$WINE_RUNNER/bin/wine" ] || { printf 'Missing Wine runner: %s\n' "$WINE_RUNNER" >&2; exit 1; }
command -v "$APPIMAGETOOL" >/dev/null 2>&1 || { printf 'appimagetool was not found\n' >&2; exit 1; }

COMPAT_RUNTIME=${COMPAT_RUNTIME:-}
if [ -z "$COMPAT_RUNTIME" ]; then
    COMPAT_RUNTIME=$(find "$HOME/.local/share/flatpak/runtime/org.freedesktop.Platform.Compat.i386" \
        -type d -path '*/files' -print -quit 2>/dev/null || true)
fi
[ -d "$COMPAT_RUNTIME" ] || { printf 'Set COMPAT_RUNTIME to the i386 runtime files directory\n' >&2; exit 1; }

BUILD_DIR=$(mktemp -d "${TMPDIR:-/tmp}/hp1-appimage-build.XXXXXX")
trap 'rm -rf "$BUILD_DIR"' EXIT HUP INT TERM
APPDIR="$BUILD_DIR/AppDir"
mkdir -p "$APPDIR/game" "$APPDIR/prefix-template" "$APPDIR/runner" "$APPDIR/runtime32"

cp -a "$GAME_DIR/." "$APPDIR/game/"
cp -a "$BOTTLE_DIR/." "$APPDIR/prefix-template/"
rm -rf "$APPDIR/prefix-template/cache" "$APPDIR/prefix-template/standalone" "$APPDIR/prefix-template/bottle.yml"
cp -a "$WINE_RUNNER/." "$APPDIR/runner/"
cp -a "$COMPAT_RUNTIME/." "$APPDIR/runtime32/"
cp "$SCRIPT_DIR/AppRun.template" "$APPDIR/AppRun"
cp "$SCRIPT_DIR/harry-potter.desktop" "$APPDIR/harry-potter.desktop"
cp "$SCRIPT_DIR/hp.svg" "$APPDIR/hp.svg"
chmod +x "$APPDIR/AppRun"

"$APPIMAGETOOL" "$APPDIR" "$OUTPUT"
printf 'Created %s\n' "$OUTPUT"
