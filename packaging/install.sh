#!/usr/bin/env bash
# Install lazygitlab from its GitHub release.
#   curl -fsSL https://raw.githubusercontent.com/arnaudbletterer/lazygitlab/master/packaging/install.sh | bash
# Environment:
#   LAZYGITLAB_VERSION      tag to install, e.g. v0.16.0 (default: latest release)
#   LAZYGITLAB_INSTALL_DIR  target directory (default: ~/.local/bin)
#   LAZYGITLAB_ALLOW_ROOT   set to 1 to allow running as root
#   LAZYGITLAB_BASE_URL     release URL prefix override (testing)
set -euo pipefail

REPO="arnaudbletterer/lazygitlab"
BASE="${LAZYGITLAB_BASE_URL:-https://github.com/${REPO}/releases}"
PROTO="=https"
WGET_PROTO="--https-only"
if [ -n "${LAZYGITLAB_BASE_URL:-}" ]; then
  PROTO="=http,https"
  WGET_PROTO="--no-verbose"
fi
INSTALL_DIR="${LAZYGITLAB_INSTALL_DIR:-$HOME/.local/bin}"

die() {
  echo "error: $*" >&2
  exit 1
}

if [ "$(id -u)" -eq 0 ] && [ "${LAZYGITLAB_ALLOW_ROOT:-0}" != 1 ]; then
  die "refusing to run as root; run as a regular user or set LAZYGITLAB_ALLOW_ROOT=1"
fi

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --proto "$PROTO" --tlsv1.2 -o "$2" "$1"
  elif command -v wget >/dev/null 2>&1; then
    wget -q $WGET_PROTO -O "$2" "$1"
  else
    die "curl or wget is required"
  fi
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    die "sha256sum or shasum is required"
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os-$arch" in
    Linux-x86_64)
      if ls /lib/ld-musl-* /usr/lib/ld-musl-* >/dev/null 2>&1; then
        die "musl libc detected (Alpine); no prebuilt binary, use: cargo install lazygitlab"
      fi
      echo "x86_64-unknown-linux-gnu"
      ;;
    Darwin-arm64 | Darwin-aarch64) echo "aarch64-apple-darwin" ;;
    Darwin-x86_64) echo "x86_64-apple-darwin" ;;
    *) die "unsupported platform ${os}/${arch} (Windows: use scoop; others: cargo install lazygitlab)" ;;
  esac
}

resolve_tag() {
  local tag="${LAZYGITLAB_VERSION:-}"
  if [ -z "$tag" ]; then
    command -v curl >/dev/null 2>&1 || die "curl is required to resolve the latest version; set LAZYGITLAB_VERSION"
    tag="$(curl -fsSLI --proto "$PROTO" --tlsv1.2 -o /dev/null -w '%{url_effective}' "${BASE}/latest")"
    tag="${tag##*/}"
  fi
  case "$tag" in
    v[0-9]*.[0-9]*.[0-9]*) ;;
    [0-9]*.[0-9]*.[0-9]*) tag="v${tag}" ;;
    *) die "invalid version tag: ${tag}" ;;
  esac
  case "$tag" in
    *[!0-9A-Za-z.v-]*) die "invalid version tag: ${tag}" ;;
  esac
  echo "$tag"
}

main() {
  local target tag name archive tmp expected actual
  target="$(detect_target)"
  tag="$(resolve_tag)"
  name="lazygitlab-${tag}-${target}"
  archive="${name}.tar.gz"

  tmp="$(mktemp -d)"
  trap "rm -rf '$tmp'" EXIT

  echo "Downloading ${archive}"
  fetch "${BASE}/download/${tag}/${archive}" "${tmp}/${archive}"
  fetch "${BASE}/download/${tag}/SHA256SUMS" "${tmp}/SHA256SUMS"

  expected="$(awk -v f="$archive" '$2 == f || $2 == "*" f { print $1; exit }' "${tmp}/SHA256SUMS")"
  [ -n "$expected" ] || die "${archive} not listed in SHA256SUMS"
  actual="$(sha256_of "${tmp}/${archive}")"
  if [ "$expected" != "$actual" ]; then
    die "checksum mismatch for ${archive}: expected ${expected}, got ${actual}"
  fi
  echo "Checksum verified"

  tar -xzf "${tmp}/${archive}" -C "$tmp"
  [ -f "${tmp}/${name}/lazygitlab" ] || die "binary not found in archive"

  mkdir -p "$INSTALL_DIR"
  install -m 755 "${tmp}/${name}/lazygitlab" "${INSTALL_DIR}/lazygitlab"
  echo "Installed lazygitlab ${tag} to ${INSTALL_DIR}/lazygitlab"

  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) echo "Note: ${INSTALL_DIR} is not in your PATH" ;;
  esac
}

main "$@"
