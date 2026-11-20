# Development — Shanti Keyboard

Development happens on `rust-rewrite`. Flatpak bundles are the primary
distribution format; source builds via `tools/build.sh` are a fallback for
development and diagnosing the build system.

## Flatpak (primary build target)

The production artifact is the GNOME IBus Flatpak, built by CI for x86_64
and aarch64 and released through GitHub Releases. Build it locally with:

```sh
flatpak-builder --user --install --force-clean \
  --default-branch=stable \
  /tmp/shanti-ibus-build tools/flatpak/shanti-ibus.yml
```

See [the Flatpak guide](../../tools/flatpak/README.md) for building,
installing, upgrading and removing the Flatpak bundle.

## Source-build fallback

For development or local source builds:

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

The build script can use Toolbx, Distrobox, Podman or Docker to isolate
dependencies. Host builds are available for CI/release packaging. Build and
staging state uses the XDG cache directory; if `XDG_CACHE_HOME` is unset, the
default is `~/.cache`. To force a clean CMake configuration, remove only the
project's build cache, not the installed application or user data:

```bash
rm -rf "${XDG_CACHE_HOME:-$HOME/.cache}/shanti-keyboard/build"
```

## Testing

CI builds the Flatpak on x86_64 and aarch64 and smoke-tests each bundle
(GUI startup, version and data checks). Source builds are not exercised by CI
and are a local fallback only.

These are build and packaging checks. They do not replace testing on a real
desktop. For input-method changes, test at minimum:
- GUI startup and settings persistence.
- IBus registration and actual Bengali typing in a host application.
- actual Bengali typing through IBus in a real GNOME session.
- Logout/login or reboot persistence where user-service configuration changes.
- Setup, conflict refusal, recovery and removal for the GNOME IBus Flatpak integration.

For Flatpak work, follow [the IBus Flatpak guide](../../tools/flatpak/README.md). Do not treat a successful bundle build as proof that host IBus integration works.

## Release workflow

The release workflow is configured for tags matching `shanti-alpha.*`. It
builds signed-update-ready IBus Flatpak bundles for x86_64 and aarch64,
generates checksums and provenance attestations, then attaches the bundles
and install descriptors to a GitHub prerelease. The first tagged run still
needs to complete before remote installation is available.