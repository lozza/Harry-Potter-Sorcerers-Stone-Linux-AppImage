#!/bin/sh
# Legacy Bottles-based builder. This is not the newer ZIP-only private beta.
# See README.md before installing Bottles or using this script.
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DEFAULT_OUTPUT="$SCRIPT_DIR/Harry-Potter-Sorcerers-Stone.AppImage"
GAME_DIR=${GAME_DIR:-}
OUTPUT=${OUTPUT:-$DEFAULT_OUTPUT}
DIALOG=none
TERMINAL_ONLY=0
CHECK_ONLY=0

usage() {
    cat <<EOF
Build a self-contained Harry Potter Linux AppImage.
LEGACY Bottles-based route only; not the ZIP-only private beta. See README.md.

Easy mode:
  $0
  Choose the game folder when the file chooser opens.

Command-line mode:
  $0 /path/to/game [output.AppImage]

Check mode:
  $0 --check

The builder automatically finds the Bottles prefix, Wine runner,
32-bit compatibility runtime, and appimagetool when they are installed
in their usual locations.

Advanced overrides:
  BOTTLE_DIR=/path/to/bottle
  WINE_RUNNER=/path/to/runner
  COMPAT_RUNTIME=/path/to/runtime/files
  APPIMAGETOOL=/path/to/appimagetool
EOF
}

show_error() {
    message=$1
    printf 'Error: %s\n' "$message" >&2
    case "$DIALOG" in
        zenity) zenity --error --title='AppImage build failed' --text="$message" >/dev/null 2>&1 || true ;;
        kdialog) kdialog --error "$message" --title 'AppImage build failed' >/dev/null 2>&1 || true ;;
    esac
}

show_success() {
    message=$1
    printf '%b\n' "$message"
    case "$DIALOG" in
        zenity) zenity --info --title='AppImage created' --text="$message" >/dev/null 2>&1 || true ;;
        kdialog) kdialog --msgbox "$message" --title 'AppImage created' >/dev/null 2>&1 || true ;;
    esac
}

fail() {
    show_error "$1"
    exit 1
}

choose_game_folder() {
    if [ "$TERMINAL_ONLY" -eq 0 ] && command -v zenity >/dev/null 2>&1; then
        DIALOG=zenity
        GAME_DIR=$(zenity --file-selection --directory --title='Select the game folder' 2>/dev/null || true)
    elif [ "$TERMINAL_ONLY" -eq 0 ] && command -v kdialog >/dev/null 2>&1; then
        DIALOG=kdialog
        GAME_DIR=$(kdialog --getexistingdirectory "${HOME:-/}" --title 'Select the game folder' 2>/dev/null || true)
    else
        DIALOG=terminal
        printf 'Enter the full path to the game folder: '
        IFS= read -r GAME_DIR
    fi
    [ -n "$GAME_DIR" ] || fail 'No game folder was selected.'
}

count_lines() {
    printf '%s\n' "$1" | awk 'NF { count++ } END { print count + 0 }'
}

choose_first_candidate() {
    label=$1
    candidates=$2
    count=$(count_lines "$candidates")
    [ "$count" -gt 0 ] || fail "Could not automatically find $label. See the README for the manual path option."
    if [ "$count" -eq 1 ]; then
        PICKED=$(printf '%s\n' "$candidates" | awk 'NF { print; exit }')
        return
    fi

    # Prefer an exact match when the bottle configuration tells us which
    # runner to use. Otherwise use the first installed candidate and print
    # the selected path so it is easy to override if needed.
    PICKED=$(printf '%s\n' "$candidates" | awk 'NF { print; exit }')
    printf 'Multiple %s candidates found; using:\n  %s\n' "$label" "$PICKED" >&2
}

find_bottle() {
    if [ -n "${BOTTLE_DIR:-}" ]; then
        [ -f "$BOTTLE_DIR/system.reg" ] || fail "BOTTLE_DIR does not contain system.reg: $BOTTLE_DIR"
        return
    fi

    candidates=
    preferred=
    for root in \
        "${HOME:-}/.var/app/com.usebottles.bottles/data/bottles/bottles" \
        "${XDG_DATA_HOME:-${HOME:-}/.local/share}/bottles/bottles" \
        "${HOME:-}/.local/share/bottles/bottles"; do
        [ -d "$root" ] || continue
        for candidate in "$root"/*; do
            [ -f "$candidate/system.reg" ] || continue
            candidates=$(printf '%s\n%s' "$candidates" "$candidate")
            case "$candidate" in
                *[Hh]arry*[Pp]otter*|*[Pp]otter*[Hh]arry*)
                    preferred=$(printf '%s\n%s' "$preferred" "$candidate") ;;
            esac
        done
    done

    if [ -n "$preferred" ]; then
        candidates=$preferred
    fi
    choose_first_candidate 'a Bottles/Wine prefix' "$candidates"
    BOTTLE_DIR=$PICKED
}

find_runner() {
    if [ -n "${WINE_RUNNER:-}" ]; then
        [ -x "$WINE_RUNNER/bin/wine" ] || fail "WINE_RUNNER does not contain bin/wine: $WINE_RUNNER"
        return
    fi

    runner_name=
    if [ -f "$BOTTLE_DIR/bottle.yml" ]; then
        runner_name=$(sed -n 's/^[[:space:]]*Runner:[[:space:]]*//p' "$BOTTLE_DIR/bottle.yml" | head -n 1)
    fi

    candidates=
    preferred=
    for root in \
        "${HOME:-}/.var/app/com.usebottles.bottles/data/bottles/runners" \
        "${XDG_DATA_HOME:-${HOME:-}/.local/share}/bottles/runners" \
        "${HOME:-}/.local/share/bottles/runners"; do
        [ -d "$root" ] || continue
        if [ -n "$runner_name" ]; then
            for candidate in "$root/$runner_name" "$root/$runner_name/files"; do
                [ -x "$candidate/bin/wine" ] || continue
                preferred=$(printf '%s\n%s' "$preferred" "$candidate")
            done
        fi
        for wine_binary in "$root"/*/bin/wine "$root"/*/files/bin/wine; do
            [ -x "$wine_binary" ] || continue
            candidate=$(dirname "$(dirname "$wine_binary")")
            candidates=$(printf '%s\n%s' "$candidates" "$candidate")
        done
    done

    if [ -n "$preferred" ]; then
        candidates=$preferred
    fi
    choose_first_candidate 'a Wine runner' "$candidates"
    WINE_RUNNER=$PICKED
}

find_compat_runtime() {
    if [ -n "${COMPAT_RUNTIME:-}" ]; then
        [ -d "$COMPAT_RUNTIME" ] || fail "COMPAT_RUNTIME is not a directory: $COMPAT_RUNTIME"
        return
    fi

    candidates=$(find \
        "${HOME:-}/.local/share/flatpak/runtime/org.freedesktop.Platform.Compat.i386" \
        /var/lib/flatpak/runtime/org.freedesktop.Platform.Compat.i386 \
        -type d -path '*/files' -print 2>/dev/null | sort -V)
    choose_first_candidate 'the 32-bit compatibility runtime' "$candidates"
    COMPAT_RUNTIME=$PICKED
}

find_appimagetool() {
    if [ -n "${APPIMAGETOOL:-}" ]; then
        if [ -x "$APPIMAGETOOL" ]; then return; fi
        command -v "$APPIMAGETOOL" >/dev/null 2>&1 || fail "APPIMAGETOOL is not executable: $APPIMAGETOOL"
        return
    fi

    if command -v appimagetool >/dev/null 2>&1; then
        APPIMAGETOOL=$(command -v appimagetool)
        return
    fi

    candidates=$(find "$SCRIPT_DIR" "${HOME:-}/Applications" "${HOME:-}/Downloads" \
        -maxdepth 1 -type f -iname 'appimagetool*' -perm -111 -print 2>/dev/null | sort)
    choose_first_candidate 'appimagetool' "$candidates"
    APPIMAGETOOL=$PICKED
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --help|-h) usage; exit 0 ;;
        --terminal) TERMINAL_ONLY=1; shift ;;
        --check) CHECK_ONLY=1; shift ;;
        --) shift; break ;;
        -*) usage >&2; exit 2 ;;
        *) break ;;
    esac
done

if [ "$#" -gt 0 ]; then
    GAME_DIR=$1
    shift
fi
if [ "$#" -gt 0 ]; then
    OUTPUT=$1
    shift
fi
[ "$#" -eq 0 ] || { usage >&2; exit 2; }

[ -n "$GAME_DIR" ] || choose_game_folder
[ -f "$GAME_DIR/System/HP.exe" ] || fail "The selected folder does not contain System/HP.exe."
[ -f "$GAME_DIR/ResolutionTool/HPSettings.exe" ] || fail "The selected folder does not contain ResolutionTool/HPSettings.exe."

find_bottle
find_runner
find_compat_runtime
find_appimagetool

printf 'Game folder:     %s\n' "$GAME_DIR"
printf 'Wine prefix:     %s\n' "$BOTTLE_DIR"
printf 'Wine runner:     %s\n' "$WINE_RUNNER"
printf '32-bit runtime:  %s\n' "$COMPAT_RUNTIME"
printf 'AppImage tool:   %s\n' "$APPIMAGETOOL"
printf 'Output:          %s\n' "$OUTPUT"

if [ "$CHECK_ONLY" -eq 1 ]; then
    show_success 'All required build components were found.'
    exit 0
fi

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
show_success "Created:\n$OUTPUT"
