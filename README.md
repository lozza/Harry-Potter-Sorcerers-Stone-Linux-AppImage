# Harry Potter and the Sorcerer's Stone on Linux

I'm making a builder that turns a supported game ZIP into a Linux AppImage. Pick the ZIP, choose where the finished AppImage should go and what resolution you want, then press **Build**. That's the aim.

This is an unofficial fan project. It's not connected to Warner Bros., EA or the original developers.

## Where things stand

The [ZIP-only beta builder AppImage](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v2.0.0-beta.1) is now public, with the [builder source](hp1-builder/README.md) here under GPLv3. Download the builder from that Release, make it executable, and open it in Desktop Mode. Its Build button has made working **private game AppImages** on Bazzite and Steam Deck. This is a beta, not a finished release.

The [v1.0.0 release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v1.0.0) and `build-appimage.sh` are the **older Bottles-based version**. I'm keeping them here for anyone who wants that method, with the [old instructions clearly marked](docs/LEGACY_BOTTLES_BUILDER.md). **You don't need to install Bottles for the new ZIP builder.**

## What the new builder does

It accepts either of two ZIP layouts I've checked: one has a parent folder, and the other has the same installer files at the top level. It checks the whole ZIP, not just its name. It then extracts the installer files without running the installer and builds your own private game AppImage. You supply the ZIP yourself; I don't provide a game download, and the builder never downloads or uploads the game. **You need an internet connection for the first build**: the builder downloads pinned free compatibility tools (about 95 MB, plus about 130 MB if the 32-bit Flatpak runtime is missing), checks their hashes and caches them for later builds. Your game ZIP stays local. Bottles and system Wine aren't required.

So far, the private tests have shown:

- The game launches with Direct3D and sound on Bazzite and Steam Deck. At 1280×720, two Deck launches went straight into the game without the Direct3D selection window.
- You can choose a starting resolution: 720p (recommended for Deck), 1080p, ultrawide, windowed, or experimental 1280×800. A resolution you choose later is saved, so rebuilding does not necessarily reset it to the build's starting default.
- Saves and settings stay on your computer, not inside the AppImage. A save appeared again on a second Bazzite launch. Rebuilding the game normally picks up the same saves. The usual folder is `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`.
- On Steam Deck, map the left stick to the **arrow keys** in that game's Steam Input layout. The default WASD mapping won't move Harry.
- The builder's **Back up saves & settings** and **Build** buttons passed hands-on private checks. Close the game before making a backup. The backup goes into a new `HP1-backup-*` folder and doesn't overwrite your saves.

## Bits I'm still working on

At 1280×800, thin lines can show up on the title or loading screens. On the Deck, the Direct3D selection window returned on repeated 1280×800 launches and would block an unattended Gaming Mode launch. That's why I'm marking 1280×800 experimental and recommending 720p for now. Dumbledore's glasses can have a thick shadow at 1280×800 and, on one repeat Bazzite test, even at 720p; Harry's glasses looked normal. Some builder text also looks a bit uneven. These graphics issues still need work.

I've only tested Bazzite and Steam Deck. I can't promise it works on every Linux PC or graphics setup yet.

The [public beta Release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v2.0.0-beta.1) contains only the small builder and its checksum, not a built **game** AppImage or game files. Free Wine and runtime components are downloaded and verified during your own build rather than copied into the builder download. The older v1.0.0 Release is still the Bottles-based version. I still need more testing on other Linux systems, and 1280×800 remains experimental.
