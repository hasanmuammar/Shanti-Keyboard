# Contributing to this fork

This repository is an experimental Linux-focused fork of OpenBangla Keyboard, rebranded as Shanti Keyboard.

## Scope

Current work targets the GNOME IBus Flatpak and its bundled desktop integration. Source build tooling and the portable IBus installer remain available. Fcitx-specific work belongs on the `fcitx-support` branch. Windows and macOS support are out of scope.

## Development branch

Use `rust-rewrite` for the current IBus rewrite work.

## Testing

For IBus registration, inspect the engine list with:

```bash
ibus list-engine | grep -i -A3 -B2 shanti
```

A manual CMake build enables IBus only:

```bash
cmake -S . -B build -G Ninja \
  -DENABLE_IBUS=ON

cmake --build build --parallel 2
```

For Flatpak packaging, build `tools/flatpak/shanti-ibus.yml` and test its host integration in a real GNOME session. A successful bundle build does not prove that Bengali input reaches host applications.

## Documentation

Fork-specific documentation belongs in `docs/wiki/`. Keep installation and integration instructions specific to this branch's IBus bundle.

## Commit messages

Keep commit messages concise and describe the actual change made in this fork.
