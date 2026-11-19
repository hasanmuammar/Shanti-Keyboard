# Plan: Switch IBus Flatpak to GTK app and retire Qt frontend

**Goal:** Complete the remaining REWRITE-PLAN.md checklist items: switch the production IBus Flatpak from the Qt `shanti-gui` on the KDE runtime to the GTK `shanti-gui-rust` on the GNOME 51 runtime, then retire the Qt frontend and remaining active C++ IBus glue while retaining the IBus install contracts.

**Branch:** `rust-rewrite`
**Plan file:** `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md`

---

## Current state (verified from repo)

- `crates/shanti-gui` GTK shell is implemented: layout viewer, settings dialog, autocorrect, about, top bar, platform config, layout converter — all committed and pushed on `rust-rewrite`.
- `tools/flatpak/shanti-gtk-dev.yml` already builds `shanti-gui-rust` on GNOME 51 runtime (dev shell).
- `tools/flatpak/shanti-ibus.yml` still builds the Qt `shanti-gui` on KDE `org.kde.Platform//5.15-25.08` via cmake-ninja.
- `src/frontend/` C++/Qt frontend still exists (~4.9k lines + 6 `.ui` files).
- `crates/CMakeLists.txt` imports `shanti-config`, `shanti-paths`, `shanti-keysyms`, `shanti-core` via corrosion — `shanti-layout` and `shanti-desktop` are NOT yet in the CMake corrosion list (GUI-only crates built standalone via cargo in the Flatpak).
- Working tree is clean; all prior work pushed to `origin/rust-rewrite`.

---

## Remaining checklist items (from REWRITE-PLAN.md §5)

1. [ ] Switch the IBus Flatpak to the GTK application after package and real-session checks pass.
2. [ ] Retire the Qt frontend and remaining active C++ IBus glue; retain the IBus install contracts.

Plus Phase 3 table leftovers:
- KeyboardWidget themes → libadwaita stylesheet + `StyleManager` dark-mode detection.

---

## Decisions needed (confirm with user or mark out of scope)

**Q1: Should the Flatpak switch plan prepare the manifest now, or wait for real-session GNOME typing validation?**
- The rewrite plan explicitly says the switch happens "after package and real-session checks pass."
- Real-session GNOME typing cannot be performed in this environment (no GNOME session, no `flatpak-builder`, no C compiler).
- **Recommended:** Prepare `shanti-ibus.yml` for the GTK build as a preparatory change, but mark the actual switch + real-session validation as gated on external testing.

**Q2: KeyboardWidget theming — hardcoded cairo colors vs libadwaita CSS/StyleManager?**
- The GTK keyboard widget currently uses hardcoded cairo colors (Qt QSS colors ported).
- The plan says "libadwaita stylesheet + StyleManager dark-mode detection; Drop Qt QSS entirely."
- **Recommended:** Add `gtk::CssProvider` theming keyed to Adwaita color classes, plus dark-mode detection via `gtk::Settings::gtk_application_prefer_dark_theme()`.

**Q3: Qt retirement scope — delete all of `src/frontend/`, or keep install-contract helpers?**
- The plan says "Retire the Qt frontend and remaining active C++ IBus glue; retain the IBus install contracts."
- `FlatpakIbusIntegration` is already ported to `shanti-desktop` (Rust). The remaining C++ frontend is the GUI only.
- **Recommended:** Delete `src/frontend/` C++ files (GUI), keep `src/shared/` and `src/platform/` as-is, remove Qt/CMake from the IBus build, drop corrosion when no active C++ consumer remains.

---

## Ordered task list

### Task 1: Prepare `shanti-ibus.yml` for GTK build (preparatory)
Update `tools/flatpak/shanti-ibus.yml`:
- `runtime: org.gnome.Platform`, `runtime-version: '51'`, `sdk: org.gnome.Sdk`
- `command: shanti-gui-rust`
- Replace `buildsystem: cmake-ninja` with `buildsystem: simple` + cargo build commands (mirror `shanti-gtk-dev.yml`)
- Update `finish-args` for GTK (keep wayland/x11 sockets, DRI, filesystem access for settings/data, D-Bus for IBus)
- Update post-install steps: install `shanti-gui-rust` binary as `shanti-gui` (preserve binary name contract), install data files, desktop file, icons, metainfo
- Add `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings` build-commands (mirror dev manifest)

**Validation:** `flatpak-builder --repo=... tools/flatpak/shanti-ibus.yml` succeeds; resulting bundle contains `shanti-gui` binary.

### Task 2: Add KeyboardWidget GTK theming (libadwaita CSS)
In `crates/shanti-gui/src/keyboard.rs`:
- Add a `GtkCssProvider` with Adwaita color classes (bezel, keycap, border, text colors)
- Read dark-mode preference via `gtk::Settings` `gtk-application-prefer-dark-theme`
- Remove hardcoded cairo color literals in favor of theme-derived colors where possible
- Keep the paint logic identical (geometry, key layout, glyph placement)

**Validation:** `cargo build --locked --manifest-path crates/shanti-gui/Cargo.toml` compiles; keyboard renders in both light and dark themes.

### Task 3: Real-session GNOME typing validation (external dependency)
- Build the GTK IBus Flatpak from the updated `shanti-ibus.yml`.
- Run on a real GNOME session (x86_64 and aarch64): add Shanti as input source, type Bangla through IBus, verify preedit/commit/candidate behavior.
- This is a **blocker** for Task 4 — cannot proceed without green validation.

**Validation:** Maintainer-confirmed real-desktop Bengali typing through IBus on GNOME.

### Task 4: Switch IBus Flatpak to GTK app (after Task 3 passes)
- Flip `tools/flatpak/shanti-ibus.yml` to the GTK build as the production manifest.
- Build and release x86_64 + aarch64 IBus Flatpaks.
- Update `CHANGELOG.md` with the milestone entry.

**Validation:** Flatpak builds green on both architectures; package-test.yml passes.

### Task 5: Retire Qt frontend (Phase 4)
- Delete `src/frontend/` C++ files (GUI: TopBar, LayoutViewer, SettingsDialog, etc.) — GUI is now in `crates/shanti-gui`.
- Remove Qt5/CMake from `src/engine/ibus/` IBus engine build (no longer links `libShared`/`Qt5::Core`).
- Remove `corrosion` from CMake when no active C++ consumer remains.
- Confirm binary names remain `shanti-gui` and `ibus-engine-shanti`; uninstall allowlist unchanged.
- Update `docs/wiki/` build instructions.
- Final GNOME/IBus real-desktop typing matrix + release.

**Validation:** `package-test.yml` uninstall allowlist test passes (must not change); no Qt in release artifacts; CHANGELOG milestone entry.

---

## Risks

| Risk | Mitigation |
|------|------------|
| GTK app not yet feature-complete vs Qt GUI | Keep Qt `shanti-gui` as production launcher until GTK covers required controls and passes package checks (per plan). |
| Real-session GNOME typing validation unavailable locally | Gate Task 4 on maintainer confirmation; do not switch manifest prematurely. |
| Binary name / uninstall allowlist contract | Preserve `shanti-gui` binary name; uninstall allowlist must not change. |
| `shanti-layout` / `shanti-desktop` not in CMake corrosion list | GUI crates built standalone via cargo in Flatpak; CMake corrosion only needed if IBus engine consumes them (it doesn't yet). |

---

## Open questions

1. Confirm Q1: prepare Flatpak manifest now or wait for real-session validation?
2. Confirm Q2: libadwaita CSS theming for KeyboardWidget?
3. Confirm Q3: delete all of `src/frontend/` or keep any install-contract helpers?

---

## Rollback

- Keep the Qt `shanti-gui` build working in `shanti-ibus.yml` until the GTK switch is validated.
- Do not delete `src/frontend/` until Task 4 passes.
- All changes are on `rust-rewrite`; can revert to `origin/rust-rewrite` if needed.

---

## Validation plan

- `cargo build --locked --manifest-path crates/shanti-gui/Cargo.toml` compiles (GTK app).
- `cargo test --locked --manifest-path crates/shanti-gui/Cargo.toml` passes.
- `cargo clippy --locked --manifest-path crates/shanti-gui/Cargo.toml -- -D warnings` clean.
- `flatpak-builder` builds `shanti-ibus.yml` successfully.
- `package-test.yml` passes (GTK binary present, uninstall allowlist intact).
- Real-desktop Bengali typing through IBus on GNOME confirmed by maintainer.

---

Write this plan to `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md` and call `plan_exit`.
