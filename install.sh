#!/bin/sh
# Install a verified hnm GitHub Release. Keep main wrapped for safe piped use.
set -eu

main() {
    fail() { printf 'hnm install: %s\n' "$*" >&2; exit 1; }
    for cmd in curl tar uname mktemp awk chmod mv; do
        command -v "$cmd" >/dev/null 2>&1 || fail "required command missing: $cmd"
    done
    if command -v sha256sum >/dev/null 2>&1; then
        checksum() { sha256sum "$1" | awk '{print $1}'; }
    elif command -v shasum >/dev/null 2>&1; then
        checksum() { shasum -a 256 "$1" | awk '{print $1}'; }
    else
        fail 'sha256sum or shasum is required'
    fi
    case "$(uname -s)" in
        Darwin) os=apple-darwin ;;
        Linux) os=unknown-linux-gnu ;;
        *) fail 'unsupported OS; supported: macOS and Linux (glibc 2.35+)' ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch=x86_64 ;;
        arm64|aarch64) arch=aarch64 ;;
        *) fail 'unsupported architecture; supported: x86_64 and ARM64' ;;
    esac
    repo=https://github.com/AkaraChen/hnm/releases
    version=${HNM_VERSION:-latest}
    if [ "$version" = latest ]; then
        url=$(curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 --max-time 120 -o /dev/null -w '%{url_effective}' "$repo/latest") || fail 'cannot resolve latest release'
        version=${url##*/}
    fi
    printf '%s\n' "$version" | awk '/^v[0-9]+\.[0-9]+\.[0-9]+$/ {ok=1} END {exit !ok}' || fail 'version must be a stable vX.Y.Z tag'
    dest=${HNM_INSTALL_DIR:-${HOME:?HOME must be set}/.local/bin}
    case "$dest" in /*) ;; *) fail 'HNM_INSTALL_DIR must be an absolute path' ;; esac
    [ ! -d "$dest/hnm" ] || fail "$dest/hnm is a directory"
    tmp=$(mktemp -d) || fail 'cannot create temporary directory'
    stage=
    trap 'rm -rf "$tmp"; if [ -n "$stage" ]; then rm -rf "$stage"; fi' 0
    trap 'exit 1' 1 2 3 15
    asset=hnm-$version-$arch-$os.tar.gz
    download() {
        curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 --max-time 120 "$repo/download/$version/$1" -o "$tmp/$1" || fail "download failed: $1 (check release/platform availability)"
    }
    download "$asset"
    download "$asset.sha256"
    expected=$(awk 'NR == 1 {print $1}' "$tmp/$asset.sha256")
    [ "${#expected}" -eq 64 ] || fail 'invalid checksum file'
    case "$expected" in *[!0-9a-f]*) fail 'invalid checksum file' ;; esac
    [ "$(checksum "$tmp/$asset")" = "$expected" ] || fail 'SHA-256 mismatch'
    tar -xzf "$tmp/$asset" -C "$tmp" hnm || fail 'invalid release archive'
    [ -f "$tmp/hnm" ] && [ ! -L "$tmp/hnm" ] || fail 'archive does not contain a regular hnm executable'
    chmod 755 "$tmp/hnm"
    actual=$("$tmp/hnm" --version) || fail 'binary cannot run on this system (Linux requires glibc 2.35+, macOS 11+)'
    [ "$actual" = "hnm ${version#v}" ] || fail "binary version mismatch: $actual"
    mkdir -p "$dest" || fail "cannot create $dest"
    stage=$(mktemp -d "$dest/.hnm-install.XXXXXX") || fail "cannot write to $dest"
    cp "$tmp/hnm" "$stage/hnm"
    chmod 755 "$stage/hnm"
    mv -f "$stage/hnm" "$dest/hnm" || fail "cannot install to $dest"
    printf 'Installed %s to %s/hnm\n' "$actual" "$dest"
    case ":${PATH:-}:" in
        *":$dest:"*) ;;
        *) printf 'Add this directory to your PATH: %s\n' "$dest" ;;
    esac
}
main "$@"
