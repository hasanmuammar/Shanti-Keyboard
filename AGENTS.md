# Repository Guidelines

## Project Overview

Shanti Keyboard is a Linux-only fork of OpenBangla Keyboard focused on GNOME and IBus. Fork development currently happens on `rust-rewrite`; Windows/macOS are out of scope. Fcitx-specific code and workflows are kept on the separate `fcitx-support` branch.

Layout: `src/engine/ibus/` contains the C++ libibus shell, `crates/` contains the Rust settings (`shanti-config`), path (`shanti-paths`), engine (`shanti-core`) and keysym (`shanti-keysyms`) crates, `src/frontend/` contains the Qt5 GUI, `src/shared/` contains shared C++ utilities, and `data/` contains layouts and language data. The GNOME IBus Flatpak is `tools/flatpak/shanti-ibus.yml`; CI and release workflows are in `.github/workflows/`. Consult `docs/REWRITE-PLAN.md` before rewrite-adjacent work.

## Naming

- Binaries: `shanti-gui`, `ibus-engine-shanti`.
- IBus bus/component: `org.freedesktop.IBus.Shanti`; engine name `Shanti`.
- Flatpak app: `org.shanti.ShantiIbus`.
- User data: `~/.local/share/shanti-keyboard`; settings: `~/.config/Shanti/Shanti.conf`; build cache: `~/.cache/shanti-keyboard`.
- Release bundles: `shanti-ibus-<arch>.flatpak` with checksum and provenance attestation.
- Preserve legacy IBus migration and uninstall references marked as legacy.

## Submodules

`src/engine/riti` and `cmake/corrosion-rs` are build dependencies. Initialize them after a fresh clone and update them recursively:

```bash
git pull --recurse-submodules
# or after a fresh clone:
git submodule update --init --recursive
```

## Build and Development

Prefer the Flatpak workflow for bundled packaging. Build an IBus source package when needed with:

```bash
bash tools/build.sh --backend ibus --package
```

The default `tools/build.sh` action builds and installs to the current user's home; use `--package` to produce `./dist` without installing. Build state is in `${XDG_CACHE_HOME:-$HOME/.cache}/shanti-keyboard/build`. The script may use Toolbx, Distrobox, Podman or Docker; it will not install Podman or change host packages in non-interactive mode.

Manual CMake builds are for diagnosing the build system. Enable IBus, build out of tree with Ninja, and do not build from the source directory. The project requires Linux, CMake ≥ 3.18, Ninja, Rust/Cargo, Qt5 (Widgets, Network, Svg, DBus), libzstd and IBus development files. `-DBUILD_FRONTEND=OFF` skips the Qt GUI.

## Generated Headers

`src/platform/include/platform.h` is cbindgen-generated and must not be edited by hand. Regenerate from `src/platform` with:

```bash
cbindgen --config cbindgen.toml --output include/platform.h
```

## Testing and Verification

- `.github/workflows/ci.yml` builds the GNOME IBus Flatpak for x86_64 and aarch64 and compiles IBus source builds.
- `.github/workflows/package-test.yml` checks shell syntax, archive integrity, package contents, GUI startup and data discovery.
- For GUI smoke checks, use `QT_QPA_PLATFORM=offscreen` with `shanti-gui --version` and `shanti-gui --check-data`.
- Engine behavior changes require typing Bangla through IBus in a real GNOME session; CI cannot verify input delivery.
- Flatpak builds use `tools/flatpak/shanti-ibus.yml`. On atomic desktops, `appstreamcli compose` may need the distro AppStream tools; see the shanti-alpha.1.4 changelog entry and `docs/wiki/Development.md`.
- Keep the Flatpak Builder state directory on the same filesystem as its target directory.

## Releases

Pushing a `shanti-alpha.*` tag builds x86_64 and aarch64 IBus Flatpak bundles with checksums and provenance attestations, then attaches them to a GitHub prerelease. Release preparation updates `CHANGELOG.md` and `release-tag.txt`; `version.txt` remains at the inherited upstream version.

## Conventions

- Use concise imperative commit subjects.
- Always commit and push the latest changes when wrapping up work in this repository.
- Keep user-local installs under the user's home; do not write to system paths.
- Do not commit build output, credentials or machine-specific configuration.
