# Plan: Switch IBus Flatpak to GTK app and retire Qt frontend

**Goal:** Complete the remaining REWRITE-PLAN.md checklist items: switch the production IBus Flatpak from the Qt `shanti-gui` on the KDE runtime to the GTK `shanti-gui-rust` on the GNOME 51 runtime, then retire the Qt frontend while retaining the IBus install contracts.

**Branch:** `rust-rewrite`
**Plan file:** `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md`

---

## Current state (verified from repo)

### Committed
- Task 1 (commit 586a8e4): `tools/flatpak/shanti-ibus.yml` rewritten for GNOME 51 runtime, `buildsystem: simple`, manual cmake+ninja for IBus engine (`-DBUILD_FRONTEND=OFF`), cargo build for GTK GUI installed as `shanti-gui`, data/desktop/icons/metainfo installed.
- Task 2 (commit 295db71): `crates/shanti-gui/src/keyboard.rs` — hardcoded cairo color literals replaced with theme-aware palette variables already defined in `paint_keyboard()`.

### Uncommitted (working tree)
- `.github/workflows/ci.yml` — smoke test: `runtime=org.kde.Platform/` → `runtime=org.gnome.Platform/`, `QT_QPA_PLATFORM=offscreen` → `GTK_A11Y=none`.
- `.github/workflows/package-test.yml` — `QT_QPA_PLATFORM=offscreen` → `GTK_A11Y=none` (lines 206-207, 249).
- `CMakeLists.txt` — removed `BUILD_FRONTEND` option, `find_package(Qt5 ...)`, `add_subdirectory(src/shared)`, `add_subdirectory(src/frontend)`. Still references `src/engine` and `src/platform`.
- `src/frontend/` directory still exists on disk (~4.9k lines + 6 `.ui` files) — needs `git rm -r`.
- `src/shared/` directory still exists on disk (links Qt5::Core, only used by `src/frontend`) — needs `git rm -r`.

### Remaining
- Task 3 (blocked): Real-session GNOME typing validation — cannot run locally (no GNOME session, no `flatpak-builder`, no C compiler). Gate for Task 4.
- Task 4: Switch IBus Flatpak to GTK app — after Task 3 passes.
- Task 5: Retire Qt frontend — delete `src/frontend/` and `src/shared/`, commit all changes, update docs (CHANGELOG.md, docs/wiki/ build instructions).

---

## Decisions (already made / to confirm)

**Q1: Prepare Flatpak manifest now, or wait for real-session validation?** → Prepared now (committed as 586a8e4), but actual switch gated on Task 3.

**Q2: KeyboardWidget theming — hardcoded cairo vs libadwaita CSS?** → Palette variables applied (committed as 295db71). Full libadwaita CSS/StyleManager can be a follow-up; not blocking.

**Q3: Qt retirement scope?** → Delete `src/frontend/` (GUI only). `src/shared/` links Qt5::Core and is only used by `src/frontend`, so delete it too. Keep `src/platform/` and `src/engine/` as-is. Binary names `shanti-gui`/`ibus-engine-shanti` preserved; uninstall allowlist unchanged.

---

## Ordered task list (implementation-ready)

### Task A: Delete retired Qt directories and commit pending changes
1. `git rm -r src/frontend src/shared`
2. Stage the uncommitted edits to `ci.yml`, `package-test.yml`, `CMakeLists.txt` plus the directory deletions.
3. Commit with message: `refactor: retire Qt5 frontend directories and CMake wiring`
4. Push `rust-rewrite`.

### Task B: Update docs and workflows for GTK-only build
1. `tools/portable/build.sh` — remove Qt5 dependency install (`qtbase5-dev`, `qtbase5-dev-tools`, `libqt5svg5-dev`), qmake query, Qt library bundling, `qt.conf` generation (these are now unnecessary since the Qt GUI is retired).
2. `tools/install.sh` — replace `QT_QPA_PLATFORM=offscreen` with `GTK_A11Y=none` (lines 276, 281).
3. `.github/workflows/package-test.yml` line 32 — replace `qtbase5-dev qtbase5-dev-tools libqt5svg5-dev` with GTK4/libadwaita dev packages (or remove if only needed for Qt GUI); verify GTK_A11Y changes already applied.
4. `docs/wiki/` build instructions — remove Qt5 build prerequisites, add GTK4/libadwaita.
5. `CHANGELOG.md` — add milestone entry for Qt retirement / GTK switch.
6. Commit all doc/workflow changes.

### Task C: Task 3 — Real-session GNOME typing validation (external dependency, BLOCKED locally)
- Build GTK IBus Flatpak from updated `shanti-ibus.yml`.
- Run on a real GNOME session (x86_64 and aarch64): add Shanti as input source, type Bangla through IBus, verify preedit/commit/candidate behavior.
- **Cannot be performed in this environment.** Gate Task D on maintainer confirmation.

### Task D: Task 4 — Switch IBus Flatpak to GTK app (after Task C passes)
- `shanti-ibus.yml` is already GTK-ready (committed as 586a8e4).
- Build and release x86_64 + aarch64 IBus Flatpaks.
- Validate with `package-test.yml`.
- **Blocked until Task C green.**

---

## Risks

| Risk | Mitigation |
|------|------------|
| `src/frontend`/`src/shared` deletion breaks unrelated code | Verified: `src/shared` only used by `src/frontend`; `src/platform` and `src/engine` unaffected by CMakeLists.txt change. |
| Binary name / uninstall allowlist contract | `shanti-gui` binary name preserved (Flatpak installs cargo binary as `shanti-gui`); uninstall allowlist unchanged. |
| Real-session GNOME validation unavailable locally | Gate Task D on maintainer confirmation; do not switch manifest prematurely. |
| `tools/portable/build.sh` Qt bundling removed but Qt GUI still needed by some users | Per plan, Qt `shanti-gui` stays until GTK covers required controls and passes package checks; coordinate with user before removing Qt from portable build. |

---

## Rollback

- Keep the Qt `shanti-gui` build working until GTK switch validated.
- Do not delete `src/frontend/` until Task C passes.
- All changes are on `rust-rewrite`; can revert to `origin/rust-rewrite` if needed.

---

## Validation plan

- `git status` clean after commits; `src/frontend` and `src/shared` removed from tracking.
- `CMakeLists.txt` configures without Qt5 (no `find_package(Qt5)`).
- `package-test.yml` passes (GTK binary present, uninstall allowlist intact).
- Real-desktop Bengali typing through IBus on GNOME confirmed by maintainer.

---

Write this plan to `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md` and call `plan_exit`.