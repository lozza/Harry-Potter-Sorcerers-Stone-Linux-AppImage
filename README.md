# Harry Potter and the Sorcerer's Stone on Linux

**HP1 Builder** is a desktop tool that turns a supported game ZIP into a Linux AppImage you can launch directly. Choose the ZIP, an output folder and a resolution; the tool builds the game app for you. No Bottles setup or system Wine installation is needed.

The original Windows game runs inside the AppImage using bundled compatibility software. It behaves like a standalone Linux app, but the game's code has **not** been rewritten as native Linux software. This is a **public beta**.

**[Download the latest v2 beta builder](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v2.0.0-beta.1)** — under **Assets**, choose `hp1-m46-online-builder-beta.AppImage`. A `.sha256` file is provided to check the download.

## The MagiPack ZIP required

This beta accepts **only two verified English ZIPs containing the MagiPack Repack v3 installer**:

- `Harry_Potter_and_the_Philosophers_Stone_MagiPack_Repack_Win_Setup_EN.zip` — the installer sits inside a parent folder.
- `Harry-Potter-and-the-Sorcerer-s-Stone_Win_EN_Repack.zip` — the installer files sit at the ZIP's top level.

The ZIP is checked by its contents, size and SHA-256, not just its filename. A different archive with a similar name will be rejected. **ISOs, installed game folders and other repacks are not supported.** No separate no-CD or compatibility file is needed.

The builder download contains **no game files**. You supply the supported ZIP locally; it is never uploaded or downloaded by the tool. This is an unofficial fan project, not connected to Warner Bros., EA or the original developers.

## Build the game AppImage

1. Download the builder, make its AppImage executable and open it in Desktop Mode.
2. Select the supported MagiPack ZIP and an output folder with **Browse**.
3. Choose a resolution. **1280×720 is recommended for Steam Deck.**
4. Click **Build AppImage**. Follow the progress until **BUILD COMPLETE** appears, then open the finished game AppImage from the output folder.

**An internet connection is required for the first build.** The tool downloads free compatibility components, verifies their hashes and caches them for later builds. Expect about 95 MB of downloads, plus roughly 130 MB if the required 32-bit Flatpak runtime is not installed. The game ZIP stays local throughout.

**Back up saves & settings** copies save slots and settings to a new backup folder without overwriting an older backup. Close the game first, then choose the backup destination. The builder also writes `hp1-builder.log` in the output folder if a build needs diagnosing.

## What has been tested

- On **x86_64 Bazzite and Steam Deck**, the builder workflow produced game AppImages that launched with Direct3D and working sound.
- A save remained available on a second launch. The builder's **Build** and **Back up saves & settings** buttons have had hands-on tests.
- This exact public builder AppImage has opened on Bazzite. A complete Deck build with this notice-only repack has not yet been repeated. Other Linux distributions are untested.

Saves and settings live outside the game AppImage, normally at `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`. Rebuilding can therefore pick up existing saves. A resolution changed at game launch is also remembered there and can take precedence over a later build's default.

For Steam Deck controls, map the **left stick to the arrow keys** in the game's Steam Input layout. The default WASD mapping does not move Harry in this setup.

## Known beta issues

- **1280×800 on Steam Deck is experimental.** It can reopen a Direct3D selection window that is difficult or impossible to use in Gaming Mode. Use 1280×720 for now.
- **Visual glitches:** thin lines can appear on title or loading screens at 1280×800. Dumbledore's glasses can show a thick shadow, sometimes even at 720p. These have not stopped gameplay in testing.
- **Builder text:** some text appears uneven at the tested desktop scale.
- **Other systems:** Linux distributions and GPU/driver combinations beyond the tested Bazzite and Steam Deck setups are not verified.

If you report a problem, include your Linux version, which of the two ZIP layouts you used, the selected resolution and the relevant part of `hp1-builder.log`. Check logs before sharing them because they may contain local paths.

The [builder source](hp1-builder/README.md) is published under GPLv3. The beta AppImage includes notices for its bundled free tools. A game AppImage made with the builder contains game files and is for personal use; do not upload it to this repository.

## Older alternative

The [v1.0.0 release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v1.0.0) was an **early Bottles-based experiment**, not the main builder. Its [instructions](docs/LEGACY_BOTTLES_BUILDER.md) remain available for anyone who needs that alternative. New users should start with the v2 builder above.
