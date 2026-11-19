#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST="$SCRIPT_DIR/shanti-gtk-dev.yml"
BUILD_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/shanti-keyboard/gtk-dev"
BUILD_DIR="$BUILD_ROOT/build"
STATE_DIR="$BUILD_ROOT/.flatpak-builder"

usage() {
    cat <<'EOF'
Usage: tools/flatpak/gtk-dev.sh COMMAND [ARGS...]

Commands:
  build       Build the GTK migration app in the GNOME 51 SDK
  shell       Open a shell in the built development app sandbox
  version     Run shanti-gui-rust --version in the sandbox
  check-data  Run shanti-gui-rust --check-data in the sandbox
EOF
}

command_name="${1:-}"
if [[ -n "$command_name" ]]; then
    shift
fi

case "$command_name" in
    build)
        command -v flatpak-builder >/dev/null 2>&1 || {
            echo "flatpak-builder is required." >&2
            exit 1
        }
        mkdir -p "$BUILD_ROOT"
        flatpak-builder --user --disable-rofiles-fuse --force-clean --build-only \
            --state-dir="$STATE_DIR" "$BUILD_DIR" "$MANIFEST"
        ;;
    shell)
        flatpak-builder --run "$BUILD_DIR" "$MANIFEST" sh "$@"
        ;;
    version)
        flatpak-builder --run "$BUILD_DIR" "$MANIFEST" shanti-gui-rust --version
        ;;
    check-data)
        flatpak-builder --run "$BUILD_DIR" "$MANIFEST" shanti-gui-rust --check-data
        ;;
    -h|--help|"")
        usage
        ;;
    *)
        echo "Unknown command: $command_name" >&2
        usage >&2
        exit 2
        ;;
esac
