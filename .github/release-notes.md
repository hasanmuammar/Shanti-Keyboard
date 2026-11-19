Shanti Keyboard Shanti is an experimental Linux-focused fork. This alpha release is distributed as Flatpak bundles only:

- IBus app bundle (`org.shanti.ShantiIbus`) for x86_64 and aarch64, integrating with the host's GNOME IBus setup.
- Both bundles build on the same Qt5 runtime (`org.kde.Platform//5.15-25.08`) and give the same frontend experience; only the engine backend differs per desktop.
- SHA-256 checksums and GitHub build-provenance attestations for the published Flatpak bundles.

The bundle pulls its runtime from Flathub during installation when Flathub is configured in the same scope. See the [Flatpak guide](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/blob/develop/tools/flatpak/README.md) for installation and GNOME setup.
