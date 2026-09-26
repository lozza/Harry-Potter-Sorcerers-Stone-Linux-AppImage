# Harry Potter 1 on Linux

I've put the new ZIP-to-AppImage builder up as a **public beta**. Give it one of the two supported MagiPack ZIP layouts, choose a resolution, and it builds a private game AppImage on your own computer. No Bottles setup or system Wine installation is needed.

**[Download the v2.0.0 beta builder](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v2.0.0-beta.1)** — grab the `hp1-m46-online-builder-beta.AppImage` file under **Assets**. The `.sha256` file beside it is there if you want to check the download.

This is an unofficial fan project, not connected to Warner Bros., EA or the original developers. The download is **only the builder**. It doesn't contain the game, a game download, or a finished game AppImage. You supply your own supported ZIP; it stays on your computer and is never uploaded.

## Build your game AppImage

1. Download the builder, make the AppImage executable and open it in Desktop Mode.
2. Click **Browse** to choose your local MagiPack ZIP and an output folder.
3. Pick a resolution. If you're on a Steam Deck, start with **1280×720**.
4. Click **Build AppImage**. You'll see progress and a clear **BUILD COMPLETE** message when it's ready. Open the finished game AppImage from your output folder.

**You need an internet connection for the first build.** The builder downloads the free tools it needs, checks their hashes and keeps them for later builds. Expect about 95 MB of downloads, plus roughly 130 MB if the required 32-bit Flatpak runtime isn't already installed. It never downloads the game. Both ZIP layouts I've verified are supported: one puts the installer files inside a parent folder; the other has them at the ZIP's top level.

There's also a **Back up saves & settings** button. Close the game before using it, choose where the backup should go, and the builder will copy your saves and settings into a new backup folder without overwriting an older one.

## What I've tested

I've built and launched private game AppImages on **x86_64 Bazzite and Steam Deck**. The game reached Direct3D with sound, and a save was still there on a second launch. The builder's Build and Backup buttons have had hands-on tests. This exact public builder file has opened on Bazzite; I haven't repeated a full build on the Deck with this exact notice-only repack. Other Linux systems are untested, so please treat this as a beta.

Your saves and settings live outside the game AppImage, normally under `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`. That's why rebuilding can pick up your existing saves. A resolution you choose when launching the game can be remembered there too, even if you later build a new AppImage with a different default.

On Steam Deck, set the **left stick to the arrow keys** in the game's Steam Input layout. The default WASD layout doesn't move Harry in this setup.

## Known beta issues

- **Steam Deck resolution:** I recommend 1280×720. The 1280×800 option is experimental; on some launches it brings back a Direct3D choice window that's awkward or impossible to use in Gaming Mode.
- **Visual glitches:** Thin lines can appear on title or loading screens at 1280×800. Dumbledore's glasses can show a thick shadow, sometimes even at 720p. Harry's glasses have looked normal. These haven't stopped gameplay in my tests, but I want to fix them.
- **Builder text:** Some text looks a little uneven at the desktop scale I've tested.
- **Other PCs:** I haven't tested other Linux distributions or every GPU/driver combination yet.

If something goes wrong, please tell me your Linux version, which of the two ZIP layouts you used, and the resolution you chose. The builder writes an `hp1-builder.log` in your output folder; check it before sharing it, since logs can contain local paths.

The [builder source](hp1-builder/README.md) is here under GPLv3. The beta AppImage includes notices for the free tools it bundles. A game AppImage made by the builder is for your own use and shouldn't be uploaded here because it contains game files.

## Older method

The [v1.0.0 release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v1.0.0) is my **older Bottles-based method**. I'm keeping its [instructions](docs/LEGACY_BOTTLES_BUILDER.md) for anyone who needs that alternative, but if you're starting now, use the new v2 beta above.
