#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/portable/common.sh"
source "$SCRIPT_DIR/portable/environment.sh"
source "$SCRIPT_DIR/portable/build.sh"
source "$SCRIPT_DIR/portable/package.sh"
source "$SCRIPT_DIR/portable/install.sh"

ACTION=install
REQUESTED_BACKEND=""
NONINTERACTIVE="${SHANTI_NONINTERACTIVE:-0}"

usage() {
    cat <<'EOF'
Usage: tools/build.sh [options]

Build Shanti Keyboard from source (source-build fallback only).

Flatpak bundles (shanti-ibus-x86_64.flatpak / shanti-ibus-aarch64.flatpak)
are the primary distribution format. Use tools/flatpak/shanti-ibus.yml with
flatpak-builder for production builds, or download bundles from GitHub
Releases. This source-build path is provided for development, local builds,
and diagnosing the build system; it is not the recommended installation path.

Options:
  --backend ibus      Build the IBus engine (the only backend in this branch).
  --package           Create a prebuilt Linux release archive in ./dist
                      instead of installing the build locally.
  -y, --yes           Do not ask before creating a temporary build environment.
  -h, --help          Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --backend)
            [[ $# -ge 2 ]] || die "--backend requires ibus."
            REQUESTED_BACKEND="$2"
            shift 2
            ;;
        --package)
            ACTION=package
            shift
            ;;
        -y|--yes)
            NONINTERACTIVE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "Unknown option: $1"
            ;;
    esac
done

[[ -z "$REQUESTED_BACKEND" || "$REQUESTED_BACKEND" == ibus ]] ||
    die "Unsupported backend: $REQUESTED_BACKEND. This branch supports IBus only."

main() {
    require_linux
    detect_desktop_backend

    if [[ -n "$REQUESTED_BACKEND" ]]; then
        OBK_BACKEND="$REQUESTED_BACKEND"
        export OBK_BACKEND
    fi

    select_builder
    prepare_paths

    if [[ "$ACTION" == package ]]; then
        NONINTERACTIVE=1
    fi

    log "Backend: $OBK_BACKEND"
    log "Build environment: $OBK_BUILDER"
    log "Build mode: $ACTION (source-build fallback; Flatpak is primary)"

    if [[ "$NONINTERACTIVE" == 1 ]]; then
        if [[ "$OBK_BUILDER" != host ]] && ! builder_exists; then
            if [[ "${OBK_NEEDS_PODMAN_BOOTSTRAP:-0}" == 1 ]]; then
                die "Non-interactive mode will not install Podman or change host packages. Install a supported builder manually, then rerun."
            fi
            OBK_CLEANUP_BUILDER_ON_SUCCESS=1
            export OBK_CLEANUP_BUILDER_ON_SUCCESS
        fi
    else
        confirm_temporary_build
    fi

    build_shanti

    if [[ "$ACTION" == package ]]; then
        package_shanti
        cleanup_builder
        log "Prebuilt source package was created in ./dist/ (source-build fallback; Flatpak bundles are the release format)."
    else
        install_shanti
        verify_installation
        cleanup_builder
        log "Shanti Keyboard was built from source and installed for the current user. Prefer Flatpak bundles for production use."
    fi
}

main "$@"
