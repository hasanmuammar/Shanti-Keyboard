# Byte-exact IBus contract fixtures

These `.golden` files are captured output of the **real Qt 5.15 writer** in
`src/frontend/FlatpakIbusIntegration.cpp` — they are the ground truth for the
Rust port in `shanti-desktop`. Contract files must stay byte-identical: the
IBus component parser, the systemd drop-in, and the uninstall allowlist all
depend on the exact paths and content.

Captured on 2026-11-19 with `org.kde.Sdk//5.15-25.08` (the same runtime pin as
`tools/flatpak/shanti-ibus.yml`) using a small program that replays
`componentXml()`, `hostWrapper()` and `serviceDropIn()` verbatim. The sample
home is `/home/testuser`.

| Fixture | Producer in C++ | Inputs |
|---|---|---|
| `component.xml.golden` | `componentXml(p.launcher)` | launcher `/home/testuser/.local/share/shanti-ibus/ibus-flatpak-host-wrapper.sh`, version `3.0.0` |
| `wrapper.sh.golden` | `hostWrapper()` | constant |
| `dropin.conf.golden` | `serviceDropIn(p)` | component dir `/home/testuser/.local/share/ibus/component` |

Note `QXmlStreamWriter::setAutoFormatting(true)` indents with **4 spaces per
level**, ends each element with `\n`, and emits the Bengali glyph `অ` as raw
UTF-8. Do not hand-edit these files; regenerate with the SDK program used for
`crates/shanti-config`'s Qt fixture if the C++ writer ever changes.
