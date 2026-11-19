# Architecture — Shanti Linux Fork

Shanti Keyboard is a Linux-focused fork of [OpenBangla Keyboard](https://github.com/OpenBangla/OpenBangla-Keyboard). It currently targets GNOME and IBus. Delivery includes a user-local installer and a bundled IBus Flatpak (`org.shanti.ShantiIbus`).

## Delivery paths

### Portable user-local installation

- `tools/install.sh` downloads and validates a published release archive, then installs it for the current user.
- `tools/build.sh` is the source-build and release-packaging entry point.
- `tools/portable/common.sh` defines paths and shared helpers.
- `tools/portable/environment.sh` selects the backend and build environment.
- `tools/portable/build.sh` configures, compiles and stages the application.
- `tools/portable/bundle-runtime.sh` collects runtime libraries and required modules.
- `tools/portable/install.sh` installs files and registers desktop/input-method resources.
- `tools/uninstall.sh` removes the user-local installation; user data is preserved unless the corresponding purge option is requested.

The default prefix is `~/.local`. Bundled runtime libraries are placed under `~/.local/lib/shanti`; build and staging state is kept under the XDG cache directory. The portable installer uses IBus.

The portable installation registers an IBus user component and configures the user service environment so the host IBus daemon can discover it.

### IBus-only Flatpak

The developer-built Flatpak manifest is `tools/flatpak/shanti-ibus.yml`, with app ID `org.shanti.ShantiIbus`. It includes the GUI and IBus engine. Its GUI settings and layout data live in the Flatpak's sandboxed data/config locations.

## Build and validation boundaries

CI builds the IBus Flatpak for x86_64 and aarch64 as downloadable artifacts; tagged alpha releases attach direct-download Flatpak bundles, with checksums and provenance attestations. Passing build/package workflows do not prove real desktop input, setup recovery, reboot persistence or cross-distribution compatibility.

The IBus Flatpak is distributed as a downloadable bundle rather than through an OSTree remote, and releases are Flatpak-only — the portable archives are no longer published as release assets (older pre-releases still carry them). Flatpak bundles are listed on the [GitHub Releases](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/releases) page. See [Installation](Installation.md), [Development](Development.md), and the [Flatpak guide](../../tools/flatpak/README.md).

## External projects

The project integrates with [IBus](https://github.com/ibus/ibus), [Qt](https://github.com/qt/qtbase), [CMake](https://github.com/Kitware/CMake), [Corrosion](https://github.com/corrosion-rs/corrosion), and [Zstandard](https://github.com/facebook/zstd). Toolbx, Distrobox, Podman and Docker are supported options for isolated source builds.
