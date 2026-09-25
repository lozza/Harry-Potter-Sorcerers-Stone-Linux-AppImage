# Third-party notices and release gate

## Private M30 package audit (not public-release clearance)

- The private M30 builder AppDir contains a copied Soda/Wine runner (~489 MB), a Freedesktop-derived 32-bit userspace runtime (~294 MB), a DXVK prefix template (~12 MB), the SDK-built GUI, and packaging/extraction tools. Its generated game AppImage contains commercial game data; neither AppImage may be uploaded as a GitHub release on the strength of private tests.
- The packaged innoextract directory contains its own `license/` files for innoextract, Boost, libbz2, liblzma, libstdc++ and zlib. A filename/path scan of `private-reference/` found no LICENSE, COPYING, copyright or NOTICE files for the bundled runner/runtime. This does not prove those components are unlicensed; it means the current package lacks the attribution/source documentation needed to review redistribution.
- The exact Soda runner and runtime copy still need an origin/version/hash-to-source mapping, per-component licence inventory, required notice texts and any corresponding-source offer. The Slint GUI distribution route also needs to be selected and followed. Do not label the private M30 binary redistributable or attach it to a public release until those checks are complete.

## Licensing choice update

The project owner has chosen GPLv3 for the builder source and Slint GUI. This
supersedes the older "route pending" wording above; the copied Wine/runtime
binary still lacks complete release clearance.

## Current source-only dependencies

| Component | Version | Source | Licence/release status | Purpose |
| --- | --- | --- | --- | --- |
| Rust | 2021 edition source; test compiler 1.98.1 | https://www.rust-lang.org/ | Toolchain is not distributed by this project | Core and CLI |
| Slint | pinned `1.13.1` | https://github.com/slint-ui/slint | Must be reviewed against the selected distribution licence before release | Native GUI |

No third-party component is cleared for a distributable builder or game
AppImage. The private integration artifact currently copies a locally audited
runner/runtime and the verified DXVK `dxgi.dll`/`d3d11.dll`, and uses local
innoextract and AppImage tooling; it is not a release artifact. Cargo's
development cache and temporary Rust toolchain are not release artifacts.

## Components that must be cleared before bundling

- ZIP/Inno extraction tooling and its licences.
- AppImage runtime and SquashFS packaging tooling.
- A standalone 32-bit Wine runner and its supporting userspace libraries.
- PulseAudio client route and any bundled fonts.
- dgVoodoo/D3D graphics correction or an alternative, including its exact
  redistribution permission.
- Desktop/icon assets and all required attribution/source-offer material.

The existing Soda/Bottles runner, Flatpak compatibility runtime, current game
AppImage, and graphics-wrapper files began as read-only compatibility
references. The private integration uses selected copies only for a local test;
they must not be copied into a public artifact based on this audit. Host Mesa
and NVIDIA drivers remain discovery-only and are never copied.
