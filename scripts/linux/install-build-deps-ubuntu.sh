#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Mezeporta Linux build-host setup (Ubuntu / Debian)

Usage:
  ./scripts/linux/install-build-deps-ubuntu.sh --install
  ./scripts/linux/install-build-deps-ubuntu.sh --verify
  ./scripts/linux/install-build-deps-ubuntu.sh --all

Flags:
  --install            Install native Linux build packages for Tauri v1.
  --verify             Verify build-host requirements for Linux builds.
  --all                Install packages and then verify the host.
  --help               Show this message.
EOF
}

log() {
  printf '[mezeporta-build] %s\n' "$*"
}

die() {
  printf '[mezeporta-build][ERROR] %s\n' "$*" >&2
  exit 1
}

warn() {
  printf '[mezeporta-build][WARN] %s\n' "$*" >&2
}

has_command() {
  command -v "$1" >/dev/null 2>&1
}

apt_package_available() {
  apt-cache show "$1" >/dev/null 2>&1
}

select_webkit_pkg() {
  if apt_package_available "libwebkit2gtk-4.0-dev"; then
    printf '%s\n' "libwebkit2gtk-4.0-dev"
  else
    printf '%s\n' "libwebkit2gtk-4.1-dev"
  fi
}

select_webkit_module() {
  if apt_package_available "libwebkit2gtk-4.0-dev"; then
    printf '%s\n' "webkit2gtk-4.0"
  else
    printf '%s\n' "webkit2gtk-4.1"
  fi
}

verify_build_setup() {
  local status=0
  local webkit_module
  webkit_module="$(select_webkit_module)"

  for tool in node npm cargo rustc rustup pkg-config; do
    if ! has_command "$tool"; then
      warn "Missing build tool: $tool"
      status=1
    fi
  done

  if has_command pkg-config && ! pkg-config --exists "$webkit_module"; then
    warn "pkg-config cannot resolve $webkit_module"
    status=1
  fi

  if has_command rustup && ! rustup target list --installed 2>/dev/null | grep -qx 'x86_64-unknown-linux-gnu'; then
    warn "Rust target x86_64-unknown-linux-gnu is not installed"
    status=1
  fi

  return "$status"
}

INSTALL_NATIVE=0
VERIFY_ONLY=0

while (($#)); do
  case "$1" in
    --install) INSTALL_NATIVE=1 ;;
    --verify) VERIFY_ONLY=1 ;;
    --all)
      INSTALL_NATIVE=1
      VERIFY_ONLY=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "Unknown argument: $1"
      ;;
  esac
  shift
done

if (( !INSTALL_NATIVE && !VERIFY_ONLY )); then
  usage
  exit 1
fi

if (( INSTALL_NATIVE )); then
  webkit_pkg="$(select_webkit_pkg)"
  log "Installing Tauri v1 Linux build packages (${webkit_pkg})"
  sudo apt update
  sudo apt install -y \
    "$webkit_pkg" \
    build-essential \
    curl \
    wget \
    file \
    pkg-config \
    libssl-dev \
    libsoup2.4-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
fi

if (( INSTALL_NATIVE || VERIFY_ONLY )); then
  verify_build_setup
fi
