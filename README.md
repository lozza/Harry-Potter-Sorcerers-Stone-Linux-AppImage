# Harry Potter and the Sorcerer's Stone (Philosopher's Stone in the UK) — Linux AppImage Build

This project provides a launcher and build script for making a self-contained Linux AppImage from a legally owned copy of **Harry Potter and the Sorcerer's Stone**, known as **Harry Potter and the Philosopher's Stone** in the UK and some other regions.

This is an unofficial community project. It is not affiliated with Warner Bros., Electronic Arts, or the original developers.

## Important: what is and is not included

This GitHub repository contains only the build recipe, launcher, desktop entry, and icon.

It does **not** include the commercial game, game installers, copyrighted game assets, a Wine prefix, or a prebuilt AppImage.

You must provide your own legally owned copy of the game. The build script copies that game, a working Wine prefix, and the required runtime into your private AppImage.

The finished AppImage is self-contained. Someone who receives the finished AppImage does not need Bottles or Wine installed separately.

## What you need to build it

You need these things on the Linux computer where you build the AppImage:

1. Your own copy of the game
2. Bottles, installed as a Flatpak
3. A working 32-bit Bottles/Wine bottle for the game
4. A Wine runner selected by that bottle
5. The 32-bit compatibility runtime used by Bottles
6. `appimagetool`

The words “32-bit bottle” and “compatibility runtime” can sound complicated, so the next sections explain each one.

## Step 1: install Bottles

Bottles is a graphical program that manages Wine environments. The Flatpak version is the most widely supported installation:

```sh
flatpak install flathub com.usebottles.bottles
```

You can also install **Bottles** from your distribution's software store. Start it from the application menu, or run:

```sh
flatpak run com.usebottles.bottles
```

## Step 2: create a 32-bit game bottle

In Bottles:

1. Click the **+** button to create a new bottle.
2. Give it a name, for example `Harry-Potter`.
3. Choose the **Gaming** environment.
4. Choose the **32-bit** architecture, usually shown as `win32` or `x86`.
5. Finish creating the bottle and wait for Bottles to download its components.

The Gaming environment supplies common game settings and dependencies. Bottles calls these managed Windows environments “bottles”; a bottle is the Wine prefix required by this build script. See the [Bottles environment documentation](https://docs.usebottles.com/getting-started/environments).

## Step 3: install or run the game in Bottles

If you have an installer, open your new bottle and run the original installer inside Bottles.

If you already have an installed game folder, open the bottle and use **Run executable** to launch:

```text
System/HP.exe
```

The game folder must contain at least:

```text
Your game folder/
├── System/HP.exe
└── ResolutionTool/HPSettings.exe
```

Run the game once before building. When the video device selection appears, choose **Direct3D Support**, not **Software Rendering**. This creates the settings files inside the bottle and confirms that the game works.

## Step 4: install or select a Wine runner

A **Wine runner** is the version of Wine that Bottles uses to run Windows programs. In Bottles, open **Preferences → Runners** and download a Wine runner if one is not already installed. Then select that runner in the bottle's preferences.

Runner names and versions change over time. The build script defaults to the runner used during development, `soda-9.0-1`, but you can point it at another runner with `WINE_RUNNER` later. Bottles explains runners in its [runner documentation](https://docs.usebottles.com/components/runners).

## Step 5: make sure the 32-bit compatibility runtime exists

The **32-bit compatibility runtime** supplies supporting Linux libraries needed by 32-bit Windows software. Bottles normally downloads the required runtime when it creates or prepares a bottle.

The build script automatically searches the standard Flatpak location. If it cannot find the runtime, check whether it exists with:

```sh
find "$HOME/.local/share/flatpak/runtime/org.freedesktop.Platform.Compat.i386" \
  -type d -path '*/files' -print
```

Use the directory ending in `/files` as `COMPAT_RUNTIME` when you run the build command. If the command prints nothing, open Bottles, allow it to finish downloading its components, and try again.

## Step 6: install `appimagetool`

`appimagetool` turns an AppDir into one portable `.AppImage` file. You can install it through your distribution if available, or download it from the [official AppImage continuous releases](https://github.com/AppImage/appimagetool/releases/continuous).

If you downloaded the AppImage version, make it executable:

```sh
chmod +x appimagetool-x86_64.AppImage
```

## Step 7: download this project

On the GitHub page, click **Code → Download ZIP**, then extract the ZIP file.

Alternatively, if Git is installed:

```sh
git clone https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage.git
cd Harry-Potter-Sorcerers-Stone-Linux-AppImage
```

Open a terminal in the folder containing `build-appimage.sh`, then make the script executable:

```sh
chmod +x build-appimage.sh
```

## Step 8: build the AppImage

Replace the path below with the location of your game folder:

```sh
./build-appimage.sh "/path/to/Harry Potter and the Sorcerer's Stone"
```

The script uses these default Bottles locations:

```text
Bottle:  ~/.var/app/com.usebottles.bottles/data/bottles/bottles/Harry-Potter
Runner:  ~/.var/app/com.usebottles.bottles/data/bottles/runners/soda-9.0-1
```

Your paths may be different. A Wine prefix is the folder containing `system.reg`; a runner is the folder containing `bin/wine`.

To use different paths, provide them before the build command:

```sh
BOTTLE_DIR="/path/to/your/Harry-Potter-bottle" \
WINE_RUNNER="/path/to/your/wine-runner" \
COMPAT_RUNTIME="/path/to/org.freedesktop.Platform.Compat.i386/files" \
APPIMAGETOOL="/path/to/appimagetool" \
./build-appimage.sh "/path/to/your/game"
```

The build can take a while and the finished AppImage is large because it contains the game, Wine, the prefix, and the runtime. The output is created in the current directory as:

```text
Harry-Potter-Sorcerers-Stone.AppImage
```

## Step 9: run the finished AppImage

Make it executable once:

```sh
chmod +x Harry-Potter-Sorcerers-Stone.AppImage
```

Then double-click it in your file manager, or run:

```sh
./Harry-Potter-Sorcerers-Stone.AppImage
```

Launching without an option opens the graphical settings chooser. The AppImage stores settings, saves, and shader caches in your user data directory instead of trying to write back into the read-only AppImage.

## Resolution profiles

You can select a profile in the graphical chooser, or start the AppImage with one of these options:

```text
--choose       Open the graphical settings chooser
--auto         Choose a profile from the detected display when possible
--ultrawide    2560×1080 fullscreen
--1080p        1920×1080 fullscreen
--720p         1280×720 fullscreen
--deck         1280×800 fullscreen
--windowed     1280×720 windowed
```

For example:

```sh
./Harry-Potter-Sorcerers-Stone.AppImage --deck
```

The launcher keeps Direct3D, controller support, audio, writable settings, and saves enabled.

## Troubleshooting the build

### `appimagetool was not found`

Install `appimagetool`, or pass its full path:

```sh
APPIMAGETOOL="/full/path/to/appimagetool" \
./build-appimage.sh "/path/to/your/game"
```

### `Missing Wine prefix`

Set `BOTTLE_DIR` to the bottle folder that contains `system.reg`. Do not point it at `drive_c` itself.

### `Missing Wine runner`

Set `WINE_RUNNER` to the runner folder that contains `bin/wine`.

### `Set COMPAT_RUNTIME to the i386 runtime files directory`

Set `COMPAT_RUNTIME` to the directory ending in `/files` under `org.freedesktop.Platform.Compat.i386`. The build script needs the runtime on the build computer so it can include it in the AppImage.

### The AppImage will not start

Make sure it is executable. If your system does not provide FUSE, try:

```sh
./Harry-Potter-Sorcerers-Stone.AppImage --appimage-extract-and-run
```

The game itself still requires a working graphics driver and a Linux system capable of running AppImages.
