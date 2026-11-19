# Shanti Rewrite Plan — Rust core, GTK4/libadwaita GUI

**Status:** In progress on `rust-rewrite`; phases land incrementally and keep the GNOME/IBus branch shippable.
**Last reviewed:** 2026-11-19
**Development branch:** `rust-rewrite`
**Related docs:** [Project Checklist](PROJECT-CHECKLIST.md) · [Development wiki](wiki/Development.md) · `.github/workflows/` CI contracts

**Current development scope:** GNOME with IBus. Fcitx-specific implementation and workflows belong on `fcitx-support`; this branch builds and releases only the IBus Flatpak. The remaining Fcitx source files in this checkout are legacy carry-over and are not a target for new work here.

## Implementation checklist

- [x] Add a separate Rust CLI migration shell with `--version`, `--check-data`, and required-data validation. Keep the Qt `shanti-gui` as the production launcher.
- [x] Configure a separate GNOME 51 Flatpak SDK build environment at `tools/flatpak/shanti-gtk-dev.yml` and build the Rust shell there.
- [x] Add the first GTK4/libadwaita application shell and preserve `--version` / `--check-data` in the separate development binary. Compiles in the GNOME SDK; desktop presentation still needs a real session check.
- [x] Port the settings and user-data model; verify compatibility with existing Shanti and migrated legacy settings. The typed `shanti-config::Settings` model reads/writes the QSettings INI byte-format, and migration helpers are exercised by fixtures.
- [ ] Port desktop integration and setup/removal flows, preserving exact file paths and uninstall behavior.
- [ ] Port dialogs and layout viewing; compare rendered layouts and autocorrect previews.
- [ ] Prototype the floating top bar on GNOME Wayland and X11, including multi-monitor behavior.
- [ ] Switch the IBus Flatpak to the GTK application after package and real-session checks pass.
- [ ] Retire the Qt frontend and remaining active C++ IBus glue; retain the IBus install contracts.

---

## 1. Goal

Rewrite the fork so that Bengali text processing, configuration, platform-glue logic, and the desktop GUI are all maintained in Rust with a modern GTK UI:

1. **C++ → Rust:** Eliminate the C++ codebase (~10k lines: engine plugins, shared utilities, Qt frontend) in favor of Rust crates, building on the ~5.3k-line Rust foundation that already exists in `src/engine/riti/`.
2. **Qt5 → GTK4/libadwaita:** Replace the Qt frontend (`shanti-gui`) with a GTK4 + libadwaita application pinned to the GNOME runtime selected at phase-3 kickoff.

Non-negotiable invariants carried across the entire rewrite:

- Linux-only scope (Windows/macOS upstream code stays deleted).
- Portable, user-local installation model (`~/.local` targets, bundled install/uninstall scripts, exact uninstall allowlist paths).
- User data compatibility: settings store location/format, custom layout discovery, autocorrect data, and the `~/.local/share/shanti-keyboard` data layout — see §6 Compat Gates.
- The GNOME IBus Flatpak builds on x86_64 and aarch64 with green CI and releases as provenance-attested `shanti-alpha.*` prereleases. Fcitx5 is outside this branch's scope.
- Real-desktop Bengali typing validated for every engine-visible change (CI cannot prove this; see [Project Checklist §2](PROJECT-CHECKLIST.md)).

## 2. What we are starting with (verified facts)

| Component | Language | Size (2026-10) | Role |
|---|---|---|---|
| `src/engine/riti/` (git submodule) | Rust | ~5.3k lines | The real engine: phonetic algorithm, khipro parser, suggestions, keycodes, config model. Exposes a C API via cbindgen (`include/riti.h`). |
| `src/engine/ibus/main.cpp` | C++ | current IBus shell | `ibus-engine-shanti`: libibus rendering and event glue around the Rust state machine in `crates/shanti-core`. |
| `src/engine/fcitx/` | C++ | legacy carry-over | Out of scope for `rust-rewrite`; Fcitx work is maintained on `fcitx-support`. |
| `src/frontend/` (main, TopBar, KeyboardWidget, SettingsDialog, About/AutoCorrect dialogs, LayoutViewer, FlatpakIbusIntegration, PlatformConfig, SingleInstance, AvroPhonetic, …) | C++/Qt5 | ~4.9k lines + 6 `.ui` files | `shanti-gui`: floating top bar, settings, layout viewer, GNOME IBus registration panel, and desktop integration. |
| `src/shared/` (Settings, FileSystem, Log) | C++/Qt5 | ~500 lines | Legacy GUI settings/path helpers. The engines use `crates/shanti-config` and `crates/shanti-paths`; the active settings store is `$XDG_CONFIG_HOME/Shanti/Shanti.conf`. |
| `src/platform/` | Rust | small | Platform support crate, cbindgen C header (committed). |
| `data/`, `src/engine/riti/data/` | JSON | — | Layouts (v3 transliteration format), dictionary, suffix, regex, autocorrect. **Not rewritten.** |

Key consequences of these facts:

- **The engines already are "Rust underneath".** The C++ in them is (a) the IM-platform API surface, (b) ~140-entry keysym translation tables, (c) settings plumbing. There is no hidden mass of business logic to port — it lives in `riti`.
- **The real rewrite volume is the frontend.** ~4.9k lines of Qt, including widgets upstream redesigned in 2026 (PR #475) and fork-only panels (GNOME IBus integration, `PlatformConfig` auto-setup).
- **The GNOME/IBus engine is already Qt-free.** Phase 1 removed Qt from the IBus engine build; the remaining active Qt dependency is the GUI. Fcitx sources that remain in this checkout are legacy carry-over and are outside this branch's build and roadmap.
- **Upstream merge compatibility decays with every phase.** The fork already manages three known upstream-merge conflict classes (Windows-engine deletions, frontend CMake source lists, SettingsDialog semantics); see §7 for the explicit divergence policy, which must be a conscious decision, not a drift.

## 3. Target architecture

```
~/.local/share/shanti-keyboard/                (Shanti user-data layout)
├── layouts/                                   (custom JSON layouts)
└── data/                                      (user data)
~/.config/Shanti/Shanti.conf                   (settings store — QSettings "Shanti"/"Shanti" INI; legacy OpenBangla settings are migrated)

workspace (new top-level Rust workspace, once the submodule constrains us)
├── crates/shanti-config       — Rust port of shared/Settings: parses the QSettings INI store,
│                                schema + defaults, unit-tested against real files
├── crates/shanti-paths        — Rust port of shared/FileSystem: PROGRAM/USER dir
│                                discovery, XDG rules, logging via `tracing`
├── crates/shanti-keysyms      — platform keysym → riti VirtualKey tables, single
│                                source, code-generated for both engines (§5 phase 2)
├── crates/shanti-core (or vendored riti) — engine core; ownership decision in §4
├── ibus-engine-shanti         — Rust engine via libibus (gir bindings) or an FFI core
│                                behind the existing C++ wrapper; decided in phase 2
└── shanti-gui                  — GTK4/libadwaita app (gtk4-rs + libadwaita-rs)
```

Build-system direction: Cargo owns Rust crates; CMake remains for the IBus wrapper and Qt frontend during migration. Remove Corrosion when the final active C++ consumer leaves the IBus build. Fcitx build-system changes are outside this roadmap.

## 4. Engine ownership decision

The current implementation keeps `src/engine/riti` as an upstream git submodule. `crates/shanti-core` depends on it directly, while fork-specific engine state and settings adaptation live in the Shanti crate. No vendoring decision is required for the GTK phase. Revisit ownership only if the fork needs to change riti internals or its API cadence blocks required work; record a vendoring hash if that decision changes.

## 5. Phases

Each phase has acceptance criteria (AC). Phases are sized so each can ship independently as a `shanti-alpha.*` prerelease.

### Phase 1 — Make settings Qt-free (engines first)

The first Rust-first milestone removed Qt from the IBus engine. *(Narrowed on 2026-10-22: Fcitx settings unification was deferred; Fcitx is now maintained on `fcitx-support`.)*

**Status: implemented on branch `rust-rewrite`.** Tasks:

- [x] `crates/shanti-paths` — Rust port of the `FileSystem.cpp` Linux contract: XDG base-dir discovery, bundled-vs-user data-dir resolution (`linuxDataDirectory()` semantics), settings-file location, user-dir bootstrap; unit-tested.
- [x] `crates/shanti-config` — read-only QSettings INI parser with typed getters matching the exact defaults in `src/shared/Settings.cpp`; unit-tested against a fixture generated by real Qt5 and used at `$XDG_CONFIG_HOME/Shanti/Shanti.conf`.
- [x] Rewire `src/engine/ibus/main.cpp` off Qt (`QCoreApplication`, `QStandardPaths`, `QFileInfo`, `QString`/`QDir` → `std::filesystem` + the new crates' C ABI).
- [x] CMake: root imports the crates via corrosion; IBus engine no longer links `libShared`/`Qt5::Core`.
- [x] package-test.yml: engine-binary Qt-free assertion (archive validation); CHANGELOG entry.
- [x] Real-desktop Bengali typing check through IBus — confirmed by the maintainer on 2026-10-22 (IBus Flatpak works fine).

**AC:** package tests prove the IBus engine has no Qt5 dependency; `--check-data` and offscreen GUI checks stay green; no regression in real-desktop Bengali typing.

### Phase 2 — IBus engine in Rust (implemented; desktop matrix remains)

Strangler-fig order: core first, wrapper stays until the port is proven.

**libibus GIR coverage scouting findings (2026-10-22, against Fedora 42 `IBus-1.0.gir`):**
- The API surface `main.cpp` actually uses is **fully introspectable**: `ibus_bus_*` (new, get_connection, register_component, request_name, set_global_engine_async), `ibus_component_new`/`add_engine`, `ibus_engine_desc_new`, `ibus_factory_new`, `ibus_engine_new`, preedit/auxiliary/lookup-table updates (`update_preedit_text_with_mode`, `update_lookup_table_fast`, `update_auxiliary_text`, hide/commit), all `ibus_lookup_table_*` manipulators, `ibus_text_new_from_string`/`get_length`, plus `ibus_main`/`ibus_quit`.
- All consumed engine signals are introspected with usable signatures, e.g. `process-key-event` → `gboolean (guint keyval, guint keycode, guint state)`; likewise `candidate-clicked`, `cursor-up/down`, `page-up/down`, `enable`, `disable`, `focus-in`, `focus-out`, `reset`, `property-activate`.
- 164 non-introspectable entries are benign for us: type/instance/class macros (`*_CLASS`, `IS_*`, `GET_CLASS`), and the varargs constructors (`*_new_varargs`, `ibus_text_new_from_printf`, `ibus_x_event_new`, emoji/unicode-data ctors, `serializable_*_attachment`) — none on the engine path.
- Quality caveats for generated bindings: several `IBusText` constructors are recorded with `return-value transfer-ownership="none"` (they actually produce floating `GInitiallyUnowned` refs), so generated Rust constructors would need ownership/metadata patching around text creation; `IBusText` and `IBusHotkeyProfile` similarly need annotation care. Signal wiring is fine.
- Verdict: glib-rs via GIR would cover ~95% of the surface, but needs a maintained GIR file, generation tooling in CI, and ownership-annotation surgery on the IBusText constructors — binding-plumbing risk layered on top of the port itself.

1. **Keysym tables single-sourced.** The per-platform `keycode.h` tables (ibus and fcitx) hold ~140 duplicated entries mapping platform enums → riti `VC_*`. Generate both engines' tables from one Rust source (`crates/shanti-keysyms`), emitting (a) the Rust map and (b) a tiny C header via cbindgen for any C++ shell still standing. (This codifies the known intentional duplication into build-time generation.)
2. **Binding strategy for libibus** (scout, then pick — record in the decision log):
   - *Preferred if annotations suffice:* gir-generated glib-rs bindings for `ibus-1.0`. libibus is GObject-introspectable — verify coverage of `IBusEngine` and the text/property APIs our plugin uses (preedit, lookup table, auxiliary text, commit).
   - *Fallback:* shrink the existing C++ `main.cpp` by moving all logic (candidates, preedit composition, riti communication, settings refresh) into Rust behind a tiny C ABI. The wrapper shrinks toward irrelevance and is removed in the GNOME/IBus retirement phase.
   - *Not recommended:* hand-rolling the IBus D-Bus protocol with `zbus` — far more protocol surface to re-implement for equal functionality.
3. Implement engine behaviors in Rust: preedit (`update_preedit_text_with_mode`), lookup table (`update_lookup_table_fast` — preserve today's suggestion → preedit/lookup-table paging semantics), auxiliary text, commits, session-start config refresh, key filtering.
4. Validate the existing `ibus-engine-shanti` descriptor, which already renders the Rust core through the small C++ libibus shell. Run the real-desktop Bengali typing matrix on GNOME (x86_64/aarch64). A parallel descriptor and default flip are unnecessary because this branch already uses the Rust core in its production engine.

**Task list:**
- [x] Keysym tables generated from `crates/shanti-keysyms` (single source; C header emitted for the C++ shells still standing — hand-written like the other crate headers, not cbindgen). Real-desktop IBus typing check confirmed by the maintainer on 2026-10-22 (local Flatpak built from `rust-rewrite`).
- [x] Scouting write-up for libibus GIR coverage, recorded in the decision log.
- [x] Rust engine implementation (preedit, lookup table, auxiliary, commits, config refresh, key filtering). The state machine is in `crates/shanti-core` behind a C ABI; the IBus shell renders its decisions. The core has 11 deterministic unit tests over the real riti phonetic engine; real-desktop IBus typing was confirmed by the maintainer on 2026-10-22.
- [x] Parallel-run descriptor and opt-in packaging reviewed: superseded because `ibus-engine-shanti` already uses the Rust core in the production descriptor; there is no separate legacy C++ engine implementation to run beside it.
- [ ] GNOME real-desktop typing matrix for the existing Rust-backed IBus engine (x86_64/aarch64).

**AC:** the Rust engine passes on-device validation in GNOME and the existing IBus packaging remains valid. Fcitx5 parity and validation are outside this branch.

### Phase 3 — GNOME GUI: Qt → GTK4/libadwaita (next implementation phase)

The largest single phase (~4.9k lines). Ground rules:

- Use gtk4-rs + libadwaita-rs. Pin the Flathub runtime (`org.gnome.Platform`, newest stable at kickoff) and the matching libadwaita version; do not switch versions mid-phase without a decision-log entry (beta/previews are out of scope). This phase targets GNOME and IBus only.
- The GUI is a **control plane**, not an IME: it touches (a) the settings store, (b) desktop-integration registration files, (c) presentation. Port file actions faithfully; do not redesign the integration contract mid-flight.
- The settings **file format** does not change; only the writer moves from QSettings to `shanti-config`.

Port order (each row lands as its own milestone behind CI checks):

The Rust GTK shell opens a basic Adwaita application window, preserves version reporting and runtime-data checks under the separate `shanti-gui-rust` name, and now loads the typed `shanti-config::Settings` model (with first-run legacy migration) to display the active store. The next milestone is desktop integration and setup/removal flows; the production Qt `shanti-gui` stays in place until the replacement covers the required controls and passes package checks. The `rust-anything` analyzer was run on 2026-11-19 but misclassified this C++ repository as C: it returned header-only candidates, 20 parse errors, and no `start_here` units. Its generated behavior-recording workflow is not applicable to these C++ units; preserve behavior with focused fixtures and end-to-end package checks as each replacement lands.

| Qt piece | GTK target | Notes & risks |
|---|---|---|
| `main.cpp`, `SingleInstance` | `adw::Application` (GApplication provides unique-instance handling) | Straightforward. |
| `TopBar` + `KeyboardWidget` (floating always-on-top bar, drag positioning, persistence) | GTK window with `gdk::monitor`-aware placement; **Wayland vs X11 behavior differs** — honor the stored position per-platform and verify multi-monitor + fractional scaling explicitly | Highest-risk widget: existing users' stored `topBarWindowPosition` must still apply. Wayland spike before committing. |
| `SettingsDialog` + fork's GNOME IBus Integration panel (backup-or-remove conflict flow) | `adw::PreferencesWindow`-style pages; keep the panel between the page header and the general card (upstream's 2026 layout anchor) | Test setup/refusal/recovery per checklist §3 after every change. |
| `FlatpakIbusIntegration` (component descriptor, launcher, systemd drop-ins, dconf input sources) | Same file actions from the GTK app via `shanti-paths` | Contract files must stay byte-identical — the uninstall allowlist depends on the exact paths. |
| `PlatformConfig` (backend detection and IBus environment setup) | Logic into `shanti-paths`/`shanti-config`; GTK code stays thin | GNOME/IBus scope. |
| `AboutDialog`, `AboutFile`, `AutoCorrectDialog` | gtk4 `AboutDialog`, `adw` widgetry, list/factory pattern | `.ui` files are not ported 1:1; rebuild in code or Adwaita templates. AutoCorrectDialog's preview uses the legacy frontend-internal `AvroPhonetic` parser (phase-1 audit) — drop it and preview through the Rust engine API instead. |
| `LayoutViewer` + `LayoutConverter` (layout image rendering, QSvgRenderer) | `GtkPicture`/draw pipeline with librsvg (gir) or `resvg` | Verify rendered layout images on both backends. |
| `KeyboardWidget` themes (`light.qss`/`dark.qss`) | libadwaita stylesheet + `StyleManager` dark-mode detection | Drop Qt QSS entirely. |
| `Layout` class (v3 layout JSON loading, rule conversion) | Rust port; coordinate the parser with riti's khipro module (decision log) | Golden tests against shipped `data/*.json`. |

Flatpak strategy: keep the existing Qt IBus Flatpak **green through this phase**; introduce a GNOME-runtime manifest for the GTK app and switch the IBus bundle only after the GTK variant passes the checklist's real-desktop GNOME integration tests (app ID, wrapper permission stories, scope rules unchanged). Only the IBus Flatpak is built and released on this branch.

**AC:** package-test grows GTK equivalents of the offscreen GUI checks (headless `GDK_BACKEND=x11` run; `--version` and `--check-data` flags preserved on the GTK binary); settings written by the GTK app are read correctly by the IBus engine and vice-versa; the checklist's GNOME/IBus validation matrix passes after each milestone and formally at the phase-3 release.

### Phase 4 — Retirement of C++ and Qt (GNOME/IBus scope)

1. Delete the remaining C++ in the GNOME/IBus path (including the IBus wrapper); remove Qt/CMake from the IBus build and manifest; drop `corrosion` only when no active consumer remains. Fcitx-specific sources are maintained on `fcitx-support` and are not part of this build.
2. Rename binaries/allowlist entries **in one commit together with the uninstall-allowlist migration** — install paths are user-visible contracts (see §6.4).
3. Update `docs/wiki/` for the new build instructions; `tools/build.sh`'s `--backend` interface stays unchanged.
4. Final GNOME/IBus real-desktop typing matrix + release.

**AC:** GNOME/IBus release artifacts contain no Qt. Fcitx5 artifacts and their implementation live on `fcitx-support`. The uninstall allowlist test still passes (it must *not* change — it is the contract); CHANGELOG milestone entry exists.

## 6. Compatibility gates (checked per phase, in package-test where possible)

1. Settings store path + INI format unchanged: `$XDG_CONFIG_HOME/Shanti/Shanti.conf` (QSettings `("Shanti", "Shanti")` org/app store) — legacy OpenBangla stores migrate through the existing path migration; fixture coverage remains required.
2. User data/interfaces unchanged: custom-layout discovery, autocorrect data, dictionary behavior.
3. IBus registration-file paths and desktop-integration contracts remain intact — the uninstall allowlist in `package-test.yml` must pass untouched.
4. Shanti binary names remain `shanti-gui` and `ibus-engine-shanti`; user-local install paths and the uninstall allowlist remain compatibility contracts.
5. CLI affordances preserved through the GUI rewrite: `shanti-gui --version`, `--check-data`.
6. No regression in the portable install model or archive layout.

## 7. Upstream divergence policy (explicit, decision-backed)

Through phase 1, upstream-merge semantics stay mostly intact except `src/shared/` (small conflict load; resolved by taking the Rust side). Phases 2–4 progressively make the fork unable to merge `upstream/develop` beyond:

- `data/*.json` layout/data updates (always mergeable),
- upstream riti algorithm fixes (mergeable until Decision 0 vendoring; afterwards cherry-picked),

…plus nothing else — engines and the entire frontend diverge permanently. **This is accepted as the price of the rewrite.** Record the milestone where upstream sync is formally removed from the routine merge process. From that point, monitor upstream only for security fixes or critical behavioral regressions worth re-implementing manually.

## 8. Risks (tracked, updated per phase)

| Risk | Mitigation |
|---|---|
| Upstream riti evolves its C API mid-transition | Vendoring (Decision 0); pin the cbindgen API version; CHANGELOG notes any migration. |
| libibus GObject-introspection coverage insufficient | Phase 2 fallback: shrink the C++ wrapper instead of a full rewrite — a Rust-logic engine still satisfies the §1 goal. |
| GTK floating bar (TopBar) Wayland window-positioning semantics differ from Qt | Dedicated GNOME Wayland + X11 spike in phase 3 before committing; Qt stays during the spike. |
| Settings INI edge cases (escaping of `/`, backslashes, arrays in QSettings) | Fixture-based round-trip tests against real-world plist files (upstream-era + fork-era). |
| GNOME IBus integration flow regressions | Checklist §3 validation matrix after each panel change; no panel merge without a real-session test. |
| aarch64-only breakage in Rust dependencies | CI already builds both architectures; extend package-test to cover both Flatpaks during the transition. |
| Maintainer bandwidth stalls mid-phase | Phases land shippable by design; phase 1 alone (Qt-free engines) is worth having even if later phases never happen. |

## 9. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-10-22 | Plan created; phase 1 defined as the first milestone | Settings/paths are small and testable and de-Qt the engines independently of the GUI. |
| 2026-11-19 | Current rewrite scope set to GNOME/IBus; Fcitx-specific code and workflows moved to `fcitx-support` | Focus current implementation and desktop validation on GNOME. Keep Fcitx-specific code and workflows on `fcitx-support`; build and release the GNOME IBus Flatpak here. |
| 2026-10-22 | Phase 1 started on branch `rust-rewrite`; narrowed to the IBus engine | Inspection showed the Fcitx5 module is already Qt-free (own INI reader). Decision 0 not required for phase 1: the new crates share no types with riti. |
| 2026-10-22 | Keysym tables single-sourced in `crates/shanti-keysyms`; engines consume it over a hand-written C ABI header, not cbindgen | The C surface is one function (`shanti_keysyms_lookup`), far below cbindgen's threshold; hand-written headers match the phase-1 crate convention. Table verified 112/112 identical across the engines and against the real IBus and Fcitx enums (in-container probe); riti `VC_*` numbers resolved from the submodule header at build time so riti bumps need no regeneration. |
| 2026-10-22 | Keep riti as the upstream submodule for the current engine rewrite | `shanti-core` depends on the Rust crate directly and holds fork-specific state externally; revisit only if internal riti changes become necessary. |
| 2026-10-22 | IBus binding strategy resolved to the **shrunken C++ wrapper**: engine logic moves into Rust behind a small C ABI; `main.cpp` shrinks to libibus plumbing. GIR/glib-rs (gir-generated) stays the documented alternative | Scouting verdict above: the used surface is introspectable, but gir-generation needs ownership-annotation surgery (`IBusText` ctors) plus CI generation tooling. The shrunken wrapper keeps all new risk in Rust logic, matches the strangler-fig plan default and the Fcitx5 model, and `main.cpp` is only ~550 lines — well below the effort threshold where hand-FFI stops being sensible. Revisit only if the GUI phase already produces a shared glib-rs layer that the engine could piggyback on. |
| 2026-10-22 | Engine state machine extracted into `crates/shanti-core` (riti as direct Rust path dependency, not over its C header); IBus `main.cpp` renders core decisions, `keycode.h` deleted | The action-decision model (per-key `SHANTI_KEY_*` codes, logical selection owned by the core, commit text hand-off) keeps ibus call shapes recognisable while making the whole key→suggestion path unit-testable: 11 tests drive the real riti phonetic engine (sessions, backspace, AltGr, modifiers, selection, commit/pass-after), pinned to a parsed settings store so they never depend on the host's `Keyboard.conf`. `EnterKeyClosesPrevWin` semantics preserved exactly (key forwarded when the setting is off). The Fcitx5 addon can adopt the same core later. |
| 2026-10-22 | **Fcitx5 addon (`openbangla.cpp`/`.h`) rewired onto shanti-core** — its duplicated riti settings mapping, keysym table and suggestion handling are gone; `keycode.h` deleted. Addon = Fcitx5 rendering of the shared core | Intentional cross-engine unifications (tested engines can't stay bit-identical while sharing one decision core): (1) candidate click commits the *pre-edit* text at the index — with ANSI enabled this is the Bijoy text — matching old Fcitx semantics; IBus previously committed the raw unicode candidate there and now matches Fcitx (arguably the correct behavior in ANSI mode). (2) Arrow/Tab navigation unified to the IBus semantics (Right/Left move when the window is horizontal, Up/Down when vertical, Tab always moves); old Fcitx committed in vertical mode for all of them and gated Tab behind horizontal — Fcitx vertical users gain working navigation. (3) The vestigial `database_dir` is validated against `engine_data_dir()/data` for both engines (riti reads its actual data via `include_bytes!`/user dir, so only the existence check differs). Old Fcitx-only bits: release events are signalled by `KeyEvent.isRelease()` and get the IBus release bit OR-ed in (Fcitx masks otherwise match IBus shift/ctrl/mod1); per-IC AltGr tracking became core-global; the old `context()` IC-switch was a dormant no-op (never seeded its tracker) — the new `syncInputContext()` actually seeds and resets, besides `reset()` which Fcitx already calls on focus changes. Fcitx keeps the timestamp-less `reloadConfig()` hook (config-dialog edits re-apply immediately). |
| 2026-10-23 | **Full rebrand to "Shanti Keyboard"** — every visible artifact renamed (binaries `shanti-gui`/`ibus-engine-shanti`, Fcitx addon id `shanti`, IBus component `org.freedesktop.IBus.Shanti`, data tree `~/.local/share/shanti-keyboard`, settings store `~/.config/Shanti/Shanti.conf`; metainfo `org.shanti.ShantiKeyboard`; Fcitx Flatpak is the Flathub addon `org.fcitx.Fcitx5.Addon.Shanti` because Flatpak pins addon IDs under the parent's extension point; legacy `openbangla*`/`OpenBangla*` names kept only in migration/uninstall paths) | User decision: brand the fork as its own project instead of shipping a confusing hybrid name. One-time copy-based migration (`ensure_migration` in shanti-paths + `migrateLegacyPaths()` in FileSystem.cpp; legacy files deliberately left behind for the uninstaller). Fcitx5 addon manifest added with `BUILD_FRONTEND=OFF` (org.kde.Sdk//6.11 has no Qt5); Rust sdk-extension in an extension-of-app build is still unverified — first real CI run will tell. |
| 2026-10-23 | **Fcitx5 delivery reworked into a self-contained app Flatpak** (`org.shanti.ShantiFcitx5`, manifest `tools/flatpak/fcitx-app.yml`), replacing the addon `org.fcitx.Fcitx5.Addon.Shanti`; **releases become Flatpak-only** (portable archives retired as release assets; older pre-releases keep theirs) | The addon-of-app model was broken by Flatpak itself: an extension bundle installed alone never pulls its parent (`[ExtensionOf]` is informational and the parent's extension point sets `no-autodownload: true`), so users got an unusable half-install. Final model (user direction: **one frontend, two backend variants in two bundles; no Qt6 GUI port; eventually "behaves exactly as the GTK4 GUI" = the phase-4 GUI replaces the frontend in both bundles at once**): the manifest builds the whole Fcitx5 stack **itself** on the same Qt5 runtime as the IBus app (`org.kde.Platform//5.15-25.08`; pinned `xcb-imdkit` 1.0.9 + `libuv` 1.53.0 + `json-c` 0.16 + `fcitx5` 5.1.14 + `fcitx5-qt` 5.1.16 with `-DENABLE_QT6=Off` + `fcitx5-configtool` 5.1.6 with `-DUSE_QT6=Off`, the last Qt5/KF5 configtool; fcitx5 pinned at 5.1.14 — the newest release whose headers still provide `stringutils.h` transitively via `standardpath.h` (configtool 5.1.6's assumption) and which needs no `PlasmaWaylandProtocols` >= 1.20 the Qt5 SDK lacks; current fcitx5 5.1.23 broke that pairing, so the stack line is version-matched, not just "newest") and then the single Shanti module with `-DBUILD_FRONTEND=ON` — engine, confs, data AND the same `shanti-gui`; desktop-file/icon/metainfo renamed per-app like the IBus manifest, GUI binary wrapped for the fcitx5-qt platforminputcontext plugin. Intermediate approach first tried and abandoned: manifest `base: org.fcitx.Fcitx5` (borrow Flathub's stack) — rejected because Flathub's Fcitx5 moved to the Qt6-only 6.11 platform (zero Qt5 libs measured in the runtime/SDK), where shanti-gui cannot run, and self-building Qt5 in-manifest (~45–90 min/arch, +150 MB double-Qt) lost to self-building the small Fcitx5 modules on the shared Qt5 runtime (SDK ships Qt5 369 libs + KF5 69 dev packages). Self-building pros: one runtime for both bundles, no Qt from source, upgrades = tag bumps (verified SHAs via `git ls-remote`; never invent pins — flathub's yaml + ls-remote are the pin source). Engine installs at the canonical `/app` Fcitx5 paths; the bundle owns Fcitx5's session bus — must not run beside Flathub's Fcitx5 app. Neither our bundle nor Flathub's Fcitx5 ships a systemd user unit — login startup documented as an XDG autostart entry (`~/.config/autostart/org.shanti.ShantiFcitx5.desktop`); the settings GUI opens from the desktop entry. The addon conf format was confirmed NOT a bug: upstream OpenBangla's `develop` fcitx conf is identical in shape (`[Dependencies]` group, `OnDemand=True`) and works on Flathub via tazihad's build; the earlier "invisible addon" saga traced to test artifacts (renamed user-dir conf copies breaking `Addon=shanti`) and hand-written profile junk, not the conf. Real-deployment insight: `OnDemand=True` addons are lazy-loaded and legitimately absent from the eager "Loaded addon" list, like emoji/spell. |
| 2026-10-23 | **shanti-core layout-load fallback** — when the settings store points at an absolute layout file that cannot be loaded (stale path, missing file), fall back to the bundled Probhat layout with a diagnostic instead of failing | Real deployment hit: a host `~/.config/Shanti/Shanti.conf` retained `path=/home/.../openbangla-keyboard/layouts/UniBijoy.json` (not in the repo's data tree); riti's `FixedMethod::new` `.unwrap()` on the failed load aborted the whole input method (the `killall5`-style crash users would see as a dead IME). Fix lives in shanti-core (`settings.rs`) rather than a riti submodule fork — upstream riti is pinned at afee54a and stays untouched; verified live in the Fcitx bundle ("Falling back to bundled layout", no panic). |

### Build-dependency audit — 2026-10-23 (audit-only; no changes applied)

| Dependency | Delivery paths | Verdict |
|---|---|---|
| `libuv` 1.53.0 (manifest module) | Fcitx5 bundle | **Necessary** — fcitx5 5.1.14's accepted event-loop backends are systemd/`libuv`/none; we build with `USE_SYSTEMD=Off`. |
| `json-c` 0.16 (manifest module) | Fcitx5 bundle | **Necessary** — fcitx5 5.1.14 requires it under `ENABLE_KEYBOARD`, which our X11/Wayland frontends need. |
| `xcb-imdkit` 1.0.9 | Fcitx5 bundle | **Necessary** — XCB/XIM frontend support. |
| vendored `en_dict` (spell data) | Fcitx5 bundle | **Necessary** — spell module ships without data otherwise; enchant is disabled (`-DENABLE_ENCHANT=Off`). |
| `Qt5::Widgets / Qt5::DBus` | `shanti-gui` | **Necessary** — Widgets is the GUI; DBus is used by `FlatpakIbusIntegration.cpp` (systemd user-unit probe) and the IBus integration panel. |
| libzstd | `shanti-gui` + engine cmake | **Necessary** — zstd-decompresses layout viewer images (`LayoutViewer.cpp`), plus engine-side deps. |
| `Fcitx5Core`/`ibus-1.0` dev, Qt5 runtime, corrosion, riti submodule, rust sdk-extension | per-backend | **Necessary** — core build chain. |
| Rust crates (`shanti-*`) | all | **Lean** — std-only, zero external deps. |
| TODO-1: `Qt5::Network` | `shanti-gui` | **Unnecessary link dep** — grep over `src/` finds no `QNetwork*`/`QtNetwork` usage at all; drop from `src/frontend/CMakeLists.txt` and re-verify offscreen GUI run. |
| TODO-2: `Qt5::Svg` | `shanti-gui` | **Likely unnecessary** — the only SVG usage is `QIcon(":/images/*.svg")` for exe-embedded theme assets, served at runtime by Qt's `qsvgicon` icon-engine *plugin* (ships in `org.kde.Platform`), not by linking the module. Drop the link only after confirming the plugin is present in the bundle and on host distros (Kinoite). |
| TODO-3: runtime `--share=network` | `fcitx-app.yml` finish-args | **Candidate to drop** (sandbox hardening) — copied from Flathub parity; neither the daemon (spell uses the vendored dict, enchant off) nor `shanti-gui` uses the network at runtime. Keep the build-time `--share=network` for the Rust fetch. |
| Cosmetic: dead cleanup entries | `fcitx-app.yml` | Harmless no-ops copied from Flathub's recipe — rime paths (`/bin/rime_deployer`, `/bin/rime_patch`) for a component we never build. Tidy whenever the manifest is next touched for real. |
| 2026-11-19 | Pin the GTK rewrite development environment to GNOME 51 | GNOME 51 is the stable runtime line at phase-3 kickoff; keep it separate from the production KDE runtime until the GTK app passes packaging and GNOME-session checks. |
| 2026-11-19 | Typed settings writing lives in `shanti-config`; the C ABI stays read-only until the Qt frontend retires | The GTK app writes the same QSettings INI bytes as Qt (fixture byte-comparison test); a single writer during the transition avoids format divergence. Full QSettings escaping for arbitrary string values remains deferred until the GTK app becomes the production writer. |
| 2026-11-19 | `shanti-desktop` mirrors `QDir::homePath()` + literal `.local/share`/`.config`, not shanti-paths' XDG resolution | `FlatpakIbusIntegration` writes host paths outside the XDG config/data roots; routing them through `shanti-paths` would change the file names inside the Flatpak and break the uninstall allowlist contract. The contract-file bytes are pinned by goldens captured from the real Qt 5.15 writer in `org.kde.Sdk//5.15-25.08`. The systemd DBus backend is supplied by the GTK settings panel (next task) behind the `ServiceControl` seam. |

*Update this log — with a commit — every time a pending row is resolved.*
