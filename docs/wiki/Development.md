# Development — Shanti Keyboard

Use the `develop` branch for fork development. The preferred entry point for source builds and package generation is `tools/build.sh`; avoid duplicating manual CMake instructions unless diagnosing the build system itself.

## Update and build

```bash
git pull --recurse-submodules
bash tools/build.sh
```

Select the IBus backend explicitly when needed:

```bash
bash tools/build.sh --backend ibus
```

Build a release archive locally:

```bash
bash tools/build.sh --backend ibus --package
```

The build script can use Toolbx, Distrobox, Podman or Docker to isolate dependencies. Host builds are available for CI/release packaging. Build and staging state uses the XDG cache directory; if `XDG_CACHE_HOME` is unset, the default is `~/.cache`. To force a clean CMake configuration, remove only the project's build cache, not the installed application or user data:

```bash
rm -rf "${XDG_CACHE_HOME:-$HOME/.cache}/shanti-keyboard/build"
```

## Testing

The CI workflow builds the Flatpak on x86_64 and aarch64 and compiles the IBus portable variant across its configured compiler/architecture matrix. The package-test workflow checks shell syntax, archive/checksum integrity, archive paths and symlink containment, package contents, GUI startup with the offscreen Qt platform, data discovery, and uninstaller safety.

These are build and packaging checks. They do not replace testing on a real desktop. For input-method changes, test at minimum:
- GUI startup and settings persistence.
- IBus registration and actual Bengali typing in a host application.
- actual Bengali typing through IBus in a real GNOME session.
- Logout/login or reboot persistence where user-service configuration changes.
- Setup, conflict refusal, recovery and removal for the GNOME IBus Flatpak integration.

For portable IBus installations, inspect registration with:

```bash
ibus list-engine | grep -i -A3 -B2 shanti
```

For Flatpak work, follow [the IBus Flatpak guide](../../tools/flatpak/README.md). Do not treat a successful bundle build as proof that host IBus integration works.

## Release workflow

The release workflow is configured for tags matching `shanti-alpha.*`. It
builds signed-update-ready IBus Flatpak bundles for x86_64 and aarch64, imports
them into a signed OSTree repository, and deploys that repository through
GitHub Pages. It generates checksums and provenance attestations, then attaches
the bundles and install descriptors to a GitHub prerelease. The first tagged
run still needs to complete before remote installation is available.
