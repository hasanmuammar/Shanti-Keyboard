# Shanti Project Checklist

**Status last reviewed:** 2026-11-19
**Development branch:** `rust-rewrite`
This checklist distinguishes repository implementation, CI evidence, real-desktop validation, and publication. A green build does not imply that the input method has been functionally tested.

**Current development scope:** bundled GNOME IBus Flatpak packaging. Fcitx-specific implementation is maintained on the separate `fcitx-support` branch.

## 1. Project scope

- [x] Linux-focused fork of OpenBangla Keyboard.
- [x] User-local portable installation architecture.
- [x] IBus engine build for x86_64 and aarch64.
- [x] x86_64 and aarch64 build targets.
- [x] Fork-owned alpha tags use the `shanti-alpha.*` pattern.
- [x] IBus Flatpak has its own app ID: `org.shanti.ShantiIbus` (pre-rebrand was `org.openbangla.Shanti.IBus`).

## 2. Portable installer and runtime

- [x] `tools/install.sh` validates release checksums and archive paths before installation.
- [x] Archive and symlink-containment validation is exercised by package CI.
- [x] Portable package checks validate GUI startup and data discovery.
- [x] Uninstaller deletion allowlist and symlink/mount-boundary safeguards are exercised by package CI.
- [ ] Publish a Shanti alpha release and test the installer against the actual release assets.
- [ ] Test installation and removal on real x86_64 and aarch64 systems.
- [ ] Test actual Bengali input through IBus on GNOME.
- [ ] Test custom absolute `XDG_DATA_HOME` end-to-end for IBus on GNOME.

## 3. Flatpak and GNOME IBus integration

- [x] IBus-only Flatpak manifest exists at `tools/flatpak/shanti-ibus.yml`.
- [x] GitHub Actions builds architecture-specific Flatpak bundles for x86_64 and aarch64.
- [x] Flatpak build is included in the main CI workflow.
- [x] App includes a GNOME-only prompt and tray-menu action for managing host IBus registration.
- [x] Settings panel shows GNOME IBus registration status and offers setup/repair and removal actions.
- [x] Setup writes a component descriptor, Flatpak launcher and GNOME IBus user-service drop-in.
- [x] Setup checks for certain conflicting user drop-ins and refuses to overwrite unmanaged files.
- [x] Setup and GNOME input-source discovery confirmed working in a GNOME session.
- [ ] Repeat setup and input-source discovery on a clean GNOME account.
- [ ] Test existing/conflicting IBus service drop-ins.
- [ ] Test failure recovery if the systemd reload or IBus service restart fails partway through setup/removal.
- [ ] Verify actual Bengali typing in host applications through the Flatpak engine.
- [ ] Verify logout/login and reboot persistence.
- [ ] Verify cleanup before/after Flatpak removal and confirm unrelated user files remain untouched.
- [ ] Decide whether the Flatpak's host-integration permissions are appropriately scoped after real-session testing.
- [ ] Add an automated Flatpak bundle install/metadata/launch smoke check to CI.
- [ ] Configure a hosted OSTree repository and public signing key before publishing a `.flatpakref`.

## 4. Current CI evidence

Latest observed workflow runs for commit `f91018f0` on 2026-10-15:

- [x] [CI run #323](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/actions/runs/38758748698) passed.
- [x] Flatpak bundle build succeeded on x86_64 and aarch64.
- [x] All configured portable source-build jobs passed.
- [x] [Prebuilt package test run #139](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/actions/runs/38758748633) passed.
- [x] Historical package tests passed on the prior multi-backend branch.
- [ ] The latest CI result for this IBus-only branch has not yet been recorded. Real Bengali input and host-integration recovery are not covered by CI.

These statuses describe the runs observed on the review date. Check the [Actions page](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/actions) for newer results.

## 5. Release and distribution

- [x] Release workflow exists for tags matching `shanti-alpha.*`.
- [x] Workflow is configured to create a prerelease and upload Flatpak bundles/checksums.
- [x] Workflow is configured to create build-provenance attestations.
- [x] Release workflow builds x86_64 and aarch64 IBus Flatpak bundles with checksums and provenance attestations and attaches them to the prerelease.
- [ ] Verify a successful tagged release run and inspect both IBus Flatpak bundles, checksums and attestations.
- [ ] Publish the first Shanti GitHub Release.
- [ ] Publish the Flatpak through a hosted OSTree repository before describing remote/`.flatpakref` installation as available.

**Current distribution state:** release publication status has not yet been rechecked. The tagged workflow is configured to attach direct-download IBus Flatpak bundles. `release-tag.txt` contains the next release tag; `version.txt` remains the inherited CMake project version.

## 6. Documentation

- [x] English and Bangla README describe the fork.
- [x] Installation, development, architecture, input-method, troubleshooting and uninstallation guides exist.
- [x] Flatpak guide documents the new in-app GNOME IBus setup and current publishing limitation.
- [x] Documentation distinguishes CI/build evidence from functional desktop verification.
- [ ] Re-review documentation whenever the host integration implementation or its permissions change.

## 7. Release readiness definition

Do not describe Shanti as generally released until:
- [ ] A tagged release run passes and publishes x86_64 and aarch64 IBus Flatpak bundles, checksums and provenance.
- [ ] GUI startup and actual Bengali input are verified through IBus on GNOME.
- [ ] Uninstallation preserves unrelated files and handles the intended data/cache choices.
- [ ] Flatpak GNOME registration is tested for conflicts, recovery, session persistence and removal.
- [ ] User-facing installation instructions match the actual published artifacts and supported environments.
