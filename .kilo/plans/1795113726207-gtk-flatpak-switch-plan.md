# Plan: Local Flatpak build test, debug, then test build and push

**Goal:** Build the IBus Flatpak locally from `tools/flatpak/shanti-ibus.yml`, debug any failures, and when the build is green, push the changes to `origin/rust-rewrite`.

**Branch:** `rust-rewrite`
**Plan file:** `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md`

---

## Current state (verified from repo)

### All implementation work is complete and pushed
- All 13 commits on `rust-rewrite` are pushed to `origin/rust-rewrite` (last: `39bfe1f`).
- `tools/flatpak/shanti-ibus.yml` is the production GTK build manifest: GNOME 51 runtime, `buildsystem: simple`, cmake (Qt-free, `-DENABLE_IBUS=ON`) for the IBus engine, cargo for the GTK GUI installed as `shanti-gui`, with `cargo fmt`/`test`/`clippy` gating the GUI install.
- `src/frontend/` and `src/shared/` (Qt5 GUI) deleted; `CMakeLists.txt` no longer references Qt5.
- CI workflows (`ci.yml`, `package-test.yml`) updated to GNOME runtime + `GTK_A11Y=none`; Qt5 deps replaced with GTK4/libadwaita.
- `AGENTS.md`, `docs/wiki/`, `CHANGELOG.md`, `.github/release-notes.md`, `tools/install.sh`, `tools/portable/build.sh`, `tools/portable/bundle-runtime.sh`, `tools/portable/install.sh` updated.
- Working tree clean except pre-existing untracked environment files (`.conda/`, `.kilo/kilo.jsonc`, `environment.yml`) — NOT part of this goal.

### Local Flatpak building is NOT possible in this environment
- `flatpak-builder` is not installed (denied by local tool rules).
- No C compiler/linker (`cc`) is available (denied; earlier context confirmed absence).
- No GNOME session available.
- `cargo`/`rustc` are only available via the Flatpak Rust SDK extension (not on host PATH).

Therefore the "test build locally then push" step **cannot be executed in this environment**. It must run either (a) in CI (`.github/workflows/release.yml` / `ci.yml`) or (b) on a host with `flatpak-builder`, `cargo`, `cc`, and the GNOME 51 runtime/SDK.

---

## Implementation-ready task list

### Task 1: Prepare for local Flatpak build (environment check)
1. Confirm `flatpak-builder` >= recent, `flatpak`, `cargo`, `rustc`, `cc`, `ninja`, `cmake` are on host PATH.
2. Confirm the GNOME 51 runtime/SDK (`org.gnome.Platform//51`, `org.gnome.Sdk//51`) and the Rust SDK extension are available/added as Flatpak remotes.
3. Confirm host architecture matches target (x86_64 or aarch64).
4. If any of the above is missing, stop: local Flatpak build cannot proceed; delegate to CI instead.

### Task 2: Test-build the Flatpak locally
Run (from repo root):
```
flatpak-builder --user --install --force-clean \
  --default-branch=stable \
  /tmp/shanti-ibus-build tools/flatpak/shanti-ibus.yml
```
Requirements per `tools/flatpak/README.md`: `--state-dir` on the same filesystem as the build dir if they differ; network access for `build-args: [--share=network]` and cargo dependency fetching.

Success criteria:
- `flatpak-builder` exits 0.
- Resulting app/ref runs: `flatpak run --user org.shanti.ShantiIbus --version` and `--check-data`.

### Task 3: Debug build failures (if Task 2 fails)
Read the failing build-command output and fix `tools/flatpak/shanti-ibus.yml`:
- CMake configure fails → check `-S . -B <dir>` path and `-DENABLE_IBUS=ON`; confirm `CMakeLists.txt` has no Qt5 references.
- cargo fmt/test/clippy fails → run `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings` against `crates/shanti-gui/Cargo.toml` locally (if cargo available).
- cargo build fails → fix Rust compile errors in `crates/shanti-gui/src/`.
- Data/desktop/icons/metainfo install fails → confirm source files exist at the paths listed in `build-commands`/`post-install`.
- After each fix, re-run Task 2 with `--force-clean`.

### Task 4: Push on success
1. `git status --short` → confirm only intended files changed (no environment files).
2. `git add <changed files>` and `git commit` if any Flatpak-manifest or source fixes were made during debugging.
3. `git push origin rust-rewrite`.
4. If no changes were needed (manifest already green), confirm `git status` is clean and report "nothing to push; manifest already builds."

---

## Risks

| Risk | Mitigation |
|------|------------|
| `flatpak-builder`/cargo/cc not available locally | Delegate to CI (`ci.yml` builds the Flatpak; `release.yml` builds+releases x86_64+aarch64). |
| `--share=network` build-arg blocked in sandbox | Ensure network access; Flatpak Builder needs network for cargo deps and Flathub runtime fetch. |
| Build-dir/state-dir on different filesystems | Use `--state-dir` on the same filesystem as `/tmp/shanti-ibus-build`, or put both under `~/.cache/shanti-keyboard/`. |
| cargo registry not cached in sandbox | First build downloads crates; may be slow or fail without network. |
| `cc` missing for Rust build scripts | CI provides the toolchain; local host lacks it. |

---

## Rollback

- All changes are already committed on `rust-rewrite`; `git push origin rust-rewrite` is the only mutating git action. If a debug fix breaks the build, revert the fix commit before pushing.
- Do not push if `git status` shows uncommitted environment files (`.conda/`, `.kilo/kilo.jsonc`, `environment.yml`); those are not part of this goal.

---

## Validation plan

- `flatpak-builder` exits 0 with `--force-clean`.
- `flatpak run --user org.shanti.ShantiIbus --version` and `--check-data` succeed.
- `git log --oneline -1` shows the fix commit on `rust-rewrite`; `git status` clean (except untracked env files).
- `git push origin rust-rewrite` succeeds.

Write this plan to `.kilo/plans/1795113726207-gtk-flatpak-switch-plan.md` and call `plan_exit`.