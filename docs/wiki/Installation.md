# Install Shanti Keyboard

Flatpak bundles (`shanti-ibus-x86_64.flatpak` / `shanti-ibus-aarch64.flatpak`)
are the primary distribution format. Download them from
[GitHub Releases](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/releases).

## GNOME: IBus Flatpak (recommended)

Flatpak needs a Flathub remote in the same scope as the app. For a per-user install, add the user remote if needed:

```bash
flatpak remote-add --user --if-not-exists flathub \
  https://flathub.org/repo/flathub.flatpakrepo
```

Install the bundle:

```bash
flatpak install --user ./shanti-ibus-x86_64.flatpak
```

Flatpak should fetch the required GNOME runtime (`org.gnome.Platform//51`) from Flathub automatically. Use the `aarch64` bundle on ARM64. Keep the scope consistent: use `--user` for both commands, or omit it for both to install system-wide. If Flatpak says the runtime was not found, install it explicitly with `flatpak install --user flathub org.gnome.Platform//51` (omit `--user` for system scope). Launch the app and accept the first-run integration prompt. Then add **Shanti** from **GNOME Settings → Keyboard → Input Sources → Add Input Source**.

See the [Flatpak guide](../../tools/flatpak/README.md) for step-by-step setup, verification and removal instructions.

## Source-build fallback

The portable source-build path is maintained as a fallback for development
and local builds when a Flatpak bundle is not available. Release assets are
Flatpak-only; there are no portable download archives for current releases.

From a source checkout, build and install for the current user:

```bash
bash tools/build.sh
```

See the [development guide](Development.md) for build options. To remove the
source-build installation, run `bash tools/uninstall.sh`. Flatpak removal is
covered in the [Flatpak guide](../../tools/flatpak/README.md).
