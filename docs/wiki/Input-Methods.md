# Input Methods — GNOME and IBus

This branch supports the IBus input-method framework on GNOME.

## IBus

The portable component file is installed at `~/.local/share/ibus/component/shanti.xml`. The engine is installed at `~/.local/libexec/ibus-engine-shanti`. The portable installer configures `IBUS_COMPONENT_PATH` for user services.

## Bundled Flatpak

The GNOME app is `org.shanti.ShantiIbus`. Its Settings panel offers GNOME IBus registration status, setup/repair and removal. Setup writes a user component descriptor, launcher script and systemd user-service drop-in, then reloads systemd and restarts GNOME IBus. See the [Flatpak guide](../../tools/flatpak/README.md) for setup, permissions and removal.

## Projects

- [IBus](https://github.com/ibus/ibus) provides the Linux input-method framework used on GNOME.
