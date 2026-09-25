# Third-party notices and release gate

## Private M30 package audit (not public-release clearance)

- The private M30 builder AppDir contains a copied Soda/Wine runner (~489 MB), a Freedesktop-derived 32-bit userspace runtime (~294 MB), a DXVK prefix template (~12 MB), the SDK-built GUI, and packaging/extraction tools. Its generated game AppImage contains commercial game data; neither AppImage may be uploaded as a GitHub release on the strength of private tests.
- The packaged innoextract directory contains its own `license/` files for innoextract, Boost, libbz2, liblzma, libstdc++ and zlib. A filename/path scan of `private-reference/` found no LICENSE, COPYING, copyright or NOTICE files for the bundled runner/runtime. This does not prove those components are unlicensed; it means the current package lacks the attribution/source documentation needed to review redistribution.
- The exact Soda runner and runtime copy still need an origin/version/hash-to-source mapping, per-component licence inventory, required notice texts and any corresponding-source offer. The Slint GUI distribution route also needs to be selected and followed. Do not label the private M30 binary redistributable or attach it to a public release until those checks are complete.

## Licensing choice update

The project owner has chosen GPLv3 for the builder source and Slint GUI. This
supersedes the older "route pending" wording above; the copied Wine/runtime
binary still lacks complete release clearance.

## M37 private-copy provenance findings

- Entire M30 runner tree matches the installed Bottles Soda 9.0-1 tree (`diff -rq` exit 0); `bin/wine` SHA-256 `77ef3686bdee1d0ddc0dfff367ecf6875204473b1c5f14fb25f9e349725cc757`. See the [Soda release](https://github.com/bottlesdevs/wine/releases/tag/soda-9.0-1).
- Entire M30 `runtime32` tree matches installed `org.freedesktop.Platform.Compat.i386//25.08` at Flathub commit `10c43710cbba7c67183615a06816d8b5a6ef4a079478c9484e9550a1d438241f` (`diff -rq` exit 0; 1,538 regular files, 403 symlinks).
- Both native DXVK DLLs match Bottles `dxvk-2.7.1-6-fc848a4/x32`. The packaged `unzip` matches Fedora `unzip-6.0-69.fc44`; innoextract reports 1.9 and includes a licence directory.
- These exact-origin matches do not clear the binary for publication. The M30 package still needs matching-source and component-notice mapping for Soda, the 32-bit Freedesktop SDK files, DXVK and all packaged tools. The related Freedesktop Platform installation has a licence collection, but its build commit differs from this Compat.i386 extension; coverage remains unverified.

## M38 Soda source-reconstruction gap

The local Bottles cache contains `soda-9.0-1-x86_64.tar.xz` at 64,564,696
bytes, SHA-256 `c38fe0ad3c12a49b61ec1fcaea5c5d8da4a3d1afc5991befe2af6b125f014c28`.
That exactly matches the asset digest currently reported by the upstream
[Soda 9.0-1 release](https://github.com/bottlesdevs/wine/releases/tag/soda-9.0-1).
The release tag points to build-workflow commit
`1bd3662c0fa6ab0e58af18a5192ce69374a1f108`, not a vendored Wine source
tree. Its [workflow](https://github.com/bottlesdevs/wine/blob/soda-9.0-1/.github/workflows/build-soda.yml)
clones Wine-TKG's default branch and fetches the Soda configuration from the
build-tools `main` branch without pinned commits. The exact source/patch set
for this particular archive has therefore not been reconstructed. Do not
treat a link to the moving upstream repositories as a complete corresponding
source package for a public binary.
The archive itself contains a top-level `LICENSE` stating Wine is
LGPL-2.1-or-later and referring to `COPYING.LIB`; the copied runner directory
omits that top-level notice, and the archive has no `COPYING.LIB` entry.

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
