# Shanti Keyboard

Shanti Keyboard is a Linux-focused fork of [OpenBangla Keyboard](https://github.com/OpenBangla/OpenBangla-Keyboard), rebranded from the pre-rebrand "OpenBangla Keyboard Shanti" naming. The project focuses on a dependable user-local installation, package integrity, and compatibility with modern atomic Linux systems.

**Project status:** Highly Experimental · Vibe-Coded

## Scope

The primary target systems are Bluefin/Dakota, Bazzite, and similar modern atomic Linux distributions. Other Linux desktops may work, but compatibility should be treated as unverified unless supported by a documented test.

The GNOME IBus app `org.shanti.ShantiIbus` is distributed as an architecture-specific Flatpak bundle through [GitHub Releases](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti/releases). See the [Flatpak guide](../../tools/flatpak/README.md).

## Project priorities

1. **Installation and removal:** predictable user-local setup and clean uninstallation, without unnecessary changes to the base operating system.
2. **Integrity:** validate package archives and downloads; verify provenance where available.
3. **Input-method integration:** keep Bengali typing working through the relevant host input-method framework.
4. **Maintainability:** reproducible build and packaging steps, automated checks, and actionable troubleshooting documentation.
5. **Distribution choices:** Flatpak bundles (`shanti-ibus-<arch>.flatpak`) are the release format; the source-build path (`tools/build.sh`) is a fallback for development and local builds only.

## Documentation

### Installation and support

- [Installation](Installation.md) — installation methods and prerequisites.
- [Uninstallation](Uninstallation.md) — removing Shanti and its user-local integration.
- [Input Methods](Input-Methods.md) — GNOME IBus integration notes.
- [Troubleshooting](Troubleshooting.md) — known issues and diagnostic steps.

### Implementation and contribution

- [Architecture](Architecture.md) — project components and runtime layout.
- [Development](Development.md) — build and development workflow.
- [Project Checklist](../PROJECT-CHECKLIST.md) — tracked project work.

### Packaging and distribution

- [Shanti Flatpaks](../../tools/flatpak/README.md) — both packages, build and publishing status.

## Development principles

- Keep installation and removal steps explicit and scoped to the user's installation.
- Prefer reproducible builds and verifiable release artifacts.
- Do not assume that success on one desktop proves compatibility with another.
- Test Bengali typing in host applications; a GUI launch alone is not an input-method integration test.
- Request no broader filesystem or session permissions than the implementation demonstrably requires.
- Distinguish verified behaviour from assumptions and unresolved issues.

## Project links

- [Shanti source repository](https://github.com/hasanmuammar/OpenBangla-Keyboard-Shanti)
- [Upstream OpenBangla Keyboard](https://github.com/OpenBangla/OpenBangla-Keyboard)
