# Harry Potter and the Sorcerer's Stone on Linux

I'm making a builder that turns a supported game ZIP into a Linux AppImage. Pick the ZIP, choose where the finished AppImage should go and what resolution you want, then press **Build**. That's the aim.

This is an unofficial fan project. It's not connected to Warner Bros., EA or the original developers.

## Where things stand

The [ZIP-builder source](hp1-builder/README.md) is now here under GPLv3, but the ready-to-run builder AppImage is still a **private beta**. I've tested the game it makes on Bazzite and Steam Deck, in Desktop and Gaming Modes. The source is available now; the binary isn't a public download yet.

The [v1.0.0 release](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases/tag/v1.0.0) and `build-appimage.sh` are the **older Bottles-based version**. I'm keeping them here for anyone who wants that method, with the [old instructions clearly marked](docs/LEGACY_BOTTLES_BUILDER.md). **You don't need to install Bottles for the new ZIP builder.**

## What the new builder does

It accepts either of two ZIP layouts I've checked: one has a parent folder, and the other has the same installer files at the top level. It checks the whole ZIP, not just its name. It then extracts the installer files without running the installer and builds your own private game AppImage. You supply the ZIP yourself; I don't provide a game download, and the builder doesn't download or upload one.

So far, the private tests have shown:

- The game launches with Direct3D and sound on Bazzite and Steam Deck.
- You can choose a starting resolution, including 720p, Steam Deck's 1280×800, 1080p, ultrawide or windowed.
- Saves and settings stay on your computer, not inside the AppImage. Rebuilding the game normally picks up the same saves. The usual folder is `${XDG_DATA_HOME:-$HOME/.local/share}/hp1-magipack-private/`.
- On Steam Deck, map the left stick to the **arrow keys** in that game's Steam Input layout. The default WASD mapping won't move Harry.
- The builder now has a **Back up saves & settings** button. Its automated tests pass; I'm still waiting for the hands-on check. Close the game before making a backup. The button makes a new `HP1-backup-*` folder and doesn't overwrite your saves.

## Bits I'm still working on

At 1280×800, thin lines can show up on the title or loading screens, and Dumbledore's glasses can have a thick shadow. We've seen that on Bazzite and Deck, but it hasn't stopped the game being playable. Some text in the builder also looks a bit uneven. The 720p option has been the most reliable-looking one so far.

I've only tested Bazzite and Steam Deck. I can't promise it works on every Linux PC or graphics setup yet.

Before I put up a public builder AppImage, I need to finish checking the Wine and runtime files it would include, plus the last few hands-on tests. I won't upload a built **game** AppImage here because that would contain the game's files. When there's a public beta ready, this page and [Releases](https://github.com/lozza/Harry-Potter-Sorcerers-Stone-Linux-AppImage/releases) will say so clearly.
