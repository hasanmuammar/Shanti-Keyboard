#!/usr/bin/env bash

build_shanti() {
    local jobs="${SHANTI_BUILD_JOBS:-2}"

    [[ "$jobs" =~ ^[1-9][0-9]*$ ]] ||
        die "SHANTI_BUILD_JOBS must be a positive integer."

    export OBK_JOBS="$jobs"

    run_builder '
        set -Eeuo pipefail

        if [[ "$(id -u)" -eq 0 ]]; then
            sudo_cmd=""
        else
            command -v sudo >/dev/null 2>&1 ||
                { echo "sudo is required inside the build environment." >&2; exit 1; }
            sudo_cmd=sudo
        fi

        deps_stamp="/var/lib/shanti-builder/deps-v4-ibus"

        if [[ ! -f "$deps_stamp" ]]; then
            if command -v apt-get >/dev/null 2>&1; then
                export DEBIAN_FRONTEND=noninteractive
                $sudo_cmd apt-get update
                $sudo_cmd apt-get install -y --no-install-recommends \
                    g++ cmake ninja-build pkg-config \
                    rustc cargo libzstd-dev \
                    ca-certificates curl

                $sudo_cmd update-ca-certificates

                $sudo_cmd apt-get install -y --no-install-recommends libibus-1.0-dev
            elif command -v dnf >/dev/null 2>&1; then
                $sudo_cmd dnf install -y \
                    gcc gcc-c++ cmake ninja-build pkgconf-pkg-config \
                    rust cargo libzstd-devel ca-certificates curl

                $sudo_cmd dnf install -y ibus-devel

                if command -v update-ca-trust >/dev/null 2>&1; then
                    $sudo_cmd update-ca-trust
                fi
            else
                echo "Unsupported package manager in build environment." >&2
                exit 1
            fi

            $sudo_cmd mkdir -p "$(dirname "$deps_stamp")"
            $sudo_cmd touch "$deps_stamp"
        fi

        if [[ -f /etc/ssl/certs/ca-certificates.crt ]]; then
            ca_bundle=/etc/ssl/certs/ca-certificates.crt
        elif [[ -f /etc/pki/tls/certs/ca-bundle.crt ]]; then
            ca_bundle=/etc/pki/tls/certs/ca-bundle.crt
        else
            echo "A system CA certificate bundle was not found in the build environment." >&2
            exit 1
        fi

        export SSL_CERT_FILE="$ca_bundle"
        export CARGO_HTTP_CAINFO="$ca_bundle"

        source_dir="$OBK_WORKSPACE"
        build_dir="$OBK_BUILD"
        stage_dir="$OBK_STAGE"

        [[ "$stage_dir" == /* && "$stage_dir" != "/" && -d "$stage_dir" && ! -L "$stage_dir" ]] || {
            echo "Refusing to use an invalid staging directory: $stage_dir" >&2
            exit 1
        }

        if [[ -n "$(find "$stage_dir" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
            echo "Staging directory is not empty: $stage_dir" >&2
            echo "No files were removed. Start a new build so a fresh staging directory is created." >&2
            exit 1
        fi

        mkdir -p "$build_dir"

        cmake -S "$source_dir" -B "$build_dir" \
            -GNinja \
            -DCMAKE_BUILD_TYPE=Release \
            -DCMAKE_INSTALL_PREFIX="$HOME/.local" \
            -DCMAKE_INSTALL_RPATH=\$ORIGIN/../lib/shanti \
            -DENABLE_IBUS=ON \
            -DCMAKE_EXE_LINKER_FLAGS=-Wl,--disable-new-dtags \
            -DCMAKE_SHARED_LINKER_FLAGS=-Wl,--disable-new-dtags

        cmake --build "$build_dir" --parallel "$OBK_JOBS"

        DESTDIR="$stage_dir" cmake --install "$build_dir"

        "$source_dir/tools/portable/bundle-runtime.sh" \
            "$runtime_lib_dir" \
            "$runtime_plugin_dir" \
            "$stage_dir"
    '
}
