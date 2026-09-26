# Harry Potter and the Sorcerer's Stone on Linux

**HP1 Builder** is a desktop tool that turns a supported prepackaged game ZIP into a Linux AppImage you can launch directly. Choose the ZIP, an output folder and a resolution; the tool builds the game app for you. No Bottles setup or system Wine installation is needed.

The original Windows game runs inside the AppImage using bundled compatibility software. It behaves like a standalone Linux app, but the game's code has **not** been rewritten as native Linux software. This is a **public beta**.

**[Download the latest v2 beta builder](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v2.0.0-beta.2)** — under **Assets**, choose `HP1-Builder-v2.0.0-beta.2-x86_64.AppImage`. A `.sha256` file is provided to check the download.

## Why this builder exists

Harry Potter and the Sorcerer's Stone was made for Windows and never had an official Linux release. Getting it running on Linux can mean piecing together Wine, graphics and sound fixes, a suitable resolution and a place for saves. This builder brings those steps together: give it one of the verified MagiPack ZIPs below and it makes a game AppImage for your own copy. You do not have to set up Bottles or system Wine yourself.

## The MagiPack ZIP required

This beta accepts **only two verified English ZIPs containing the MagiPack Repack v3 installer**:

- `Harry_Potter_and_the_Philosophers_Stone_MagiPack_Repack_Win_Setup_EN.zip` — the installer sits inside a parent folder.
- `Harry-Potter-and-the-Sorcerer-s-Stone_Win_EN_Repack.zip` — the installer files sit at the ZIP's top level.

Looking for the ZIP? [My Abandonware](https://www.myabandonware.com/) and [Old Games Download](https://oldgamesdownload.com/) both list the MagiPack repack. They are independent of this project; please make sure you're allowed to download and use the game. The builder only accepts the two verified ZIP versions named above.

The ZIP is checked by its contents, size and SHA-256, not just its filename. A different archive with a similar name will be rejected. **ISOs, installed game folders and other repacks are not supported.** No separate no-CD or compatibility file is needed.

The builder download contains **no game files**. You supply the supported ZIP locally; it is never uploaded or downloaded by the tool. This is an unofficial fan project, not connected to Warner Bros., EA or the original developers.

## Build the game AppImage

**An internet connection is required for the first build.** The tool downloads free compatibility components, verifies their hashes and caches them for later builds. Expect about 95 MB of downloads, plus roughly 130 MB if the required 32-bit Flatpak runtime is not installed. The game ZIP stays local throughout.

### Linux desktop

1. Download the builder, mark its AppImage as executable in your file manager (usually under **Properties → Permissions**), then open it.
2. Select the supported MagiPack ZIP and an output folder with **Browse**.
3. Choose a resolution, then click **Build AppImage**.
4. Wait for **BUILD COMPLETE**, then open the finished game AppImage from the output folder.

### Steam Deck

Switch to **Desktop Mode** and follow the same build steps. Choose **1280×720** for the tested Deck setup; 1280×800 is experimental. To play in Gaming Mode, right-click the finished game AppImage in Dolphin and choose **Add to Steam**.

### Steam Deck controls

In Gaming Mode, open the game's **Steam Input** settings and choose the **Keyboard (WASD) and Mouse** profile. Then map the **left stick to the arrow keys**—the game uses arrow keys for movement, so leaving that stick on WASD will not move Harry.

### Backups and logs

**Back up saves & settings** copies save slots and settings to a new backup folder without overwriting an older backup. Close the game first, then choose the backup destination. The builder also writes `hp1-builder.log` in the output folder if a build needs diagnosing.

## What has been tested

- On **x86_64 Bazzite and Steam Deck**, the builder workflow produced game AppImages that launched with Direct3D and working sound.
- A save remained available on a second launch. The builder's **Build** and **Back up saves & settings** buttons have had hands-on tests.
- This exact public builder AppImage has opened on Bazzite. A complete Deck build with this notice-only repack has not yet been repeated. Other Linux distributions are untested.

Saves and settings live outside the game AppImage, normally at `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`. Rebuilding can therefore pick up existing saves. A resolution changed at game launch is also remembered there and can take precedence over a later build's default.

## Known beta issues

- **1280×800 on Steam Deck is experimental.** It can reopen a Direct3D selection window that is difficult or impossible to use in Gaming Mode. Use 1280×720 for now.
- **Visual glitches:** thin lines can appear on title or loading screens at 1280×800. Dumbledore's glasses can show a thick shadow, sometimes even at 720p. These have not stopped gameplay in testing.
- **Builder text:** some text appears uneven at the tested desktop scale.
- **Other systems:** Linux distributions and GPU/driver combinations beyond the tested Bazzite and Steam Deck setups are not verified.

If you report a problem, include your Linux version, which of the two ZIP layouts you used, the selected resolution and the relevant part of `hp1-builder.log`. Check logs before sharing them because they may contain local paths.

The [builder source](hp1-builder/README.md) is published under GPLv3. The beta AppImage includes notices for its bundled free tools. A game AppImage made with the builder contains game files and is for personal use; do not upload it to this repository.

## Older alternative

The [v1.0.0 release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v1.0.0) was an **early Bottles-based experiment**, not the main builder. Its [instructions](docs/LEGACY_BOTTLES_BUILDER.md) remain available for anyone who needs that alternative. New users should start with the v2 builder above.
