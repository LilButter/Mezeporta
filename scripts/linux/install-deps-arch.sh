#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Mezeporta Linux runtime setup (Arch)

Usage:
  ./scripts/linux/install-deps-arch.sh --wine
  ./scripts/linux/install-deps-arch.sh --verify
  ./scripts/linux/install-deps-arch.sh --prefix /path/to/prefix
  ./scripts/linux/install-deps-arch.sh --portable-prefix /path/to/game-root
  ./scripts/linux/install-deps-arch.sh --portable-prefix /path/to/game-root --controller-fix
  ./scripts/linux/install-deps-arch.sh --dotnet-installer /path/to/runtime.exe --prefix /path/to/prefix
  ./scripts/linux/install-deps-arch.sh --all --portable-prefix /path/to/game-root

Flags:
  --wine               Install Wine, AppImage runtime support, and media/runtime packages.
  --verify             Verify Mezeporta Linux runtime requirements on this host.
  --prefix PATH        Bootstrap the given Wine prefix.
  --portable-prefix    Bootstrap <game-root>/Mezeporta/WinePrefix.
  --controller-fix     Apply xinput1_3, dinput, and d8input native,builtin overrides.
  --dotnet-installer   Optionally install a Windows .NET runtime into the selected prefix.
  --all                Alias for --wine.
  --help               Show this message.
EOF
}

log() {
  printf '[mezeporta] %s\n' "$*"
}

die() {
  printf '[mezeporta][ERROR] %s\n' "$*" >&2
  exit 1
}

warn() {
  printf '[mezeporta][WARN] %s\n' "$*" >&2
}

has_command() {
  command -v "$1" >/dev/null 2>&1
}

verify_runtime_setup() {
  local status=0

  for tool in wine wineserver winetricks; do
    if ! has_command "$tool"; then
      warn "Missing runtime tool: $tool"
      status=1
    fi
  done

  if ! has_command gst-inspect-1.0; then
    warn "Missing gst-inspect-1.0; install gstreamer"
    status=1
  else
    if ! gst-inspect-1.0 appsink >/dev/null 2>&1; then
      warn "GStreamer appsink plugin is missing; install gst-plugins-base"
      status=1
    fi
    if ! gst-inspect-1.0 autoaudiosink >/dev/null 2>&1; then
      warn "GStreamer autoaudiosink is missing; install gst-plugins-good plus an audio sink package"
      status=1
    fi
    if ! gst-inspect-1.0 pipewiresink >/dev/null 2>&1 \
      && ! gst-inspect-1.0 pulsesink >/dev/null 2>&1 \
      && ! gst-inspect-1.0 alsasink >/dev/null 2>&1; then
      warn "No working GStreamer audio sink plugin was found; install gst-plugin-pipewire or gst-plugins-good with PulseAudio/ALSA support"
      status=1
    fi
  fi

  return "$status"
}

INSTALL_WINE=0
VERIFY_ONLY=0
PREFIX_PATH=""
PORTABLE_ROOT=""
DOTNET_INSTALLER=""
APPLY_CONTROLLER_FIX=0

while (($#)); do
  case "$1" in
    --wine) INSTALL_WINE=1 ;;
    --verify) VERIFY_ONLY=1 ;;
    --all) INSTALL_WINE=1 ;;
    --controller-fix) APPLY_CONTROLLER_FIX=1 ;;
    --prefix)
      shift
      PREFIX_PATH="${1:-}"
      ;;
    --portable-prefix)
      shift
      PORTABLE_ROOT="${1:-}"
      ;;
    --dotnet-installer)
      shift
      DOTNET_INSTALLER="${1:-}"
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

if [[ -n "$PORTABLE_ROOT" ]]; then
  PREFIX_PATH="${PORTABLE_ROOT%/}/Mezeporta/WinePrefix"
fi

install_runtime_deps() {
  log "Installing Mezeporta Linux runtime dependencies"
  sudo pacman -Syu --needed --noconfirm \
    fuse2 \
    gstreamer \
    wine \
    winetricks \
    cabextract \
    p7zip \
    vulkan-tools \
    mesa-utils \
    gamemode \
    gst-plugins-base \
    gst-plugins-good \
    gst-plugins-bad \
    gst-plugins-ugly \
    gst-plugin-pipewire \
    gst-libav
}

apply_controller_fix_overrides() {
  log "Applying controller DLL overrides"
  for dll in xinput1_3 dinput d8input; do
    wine reg add "HKCU\\Software\\Wine\\DllOverrides" \
      /v "$dll" /t REG_SZ /d "native,builtin" /f >/dev/null
  done
  wineserver -w
}

bootstrap_prefix() {
  local prefix_path="$1"
  [[ -n "$prefix_path" ]] || die "No prefix path provided"
  log "Bootstrapping Wine prefix at $prefix_path"
  mkdir -p "$prefix_path"
  export WINEPREFIX="$prefix_path"
  export WINEDEBUG=-all
  wineboot -u
  wineserver -w
  WINETRICKS_DISABLE_LNK=1 WINETRICKS_OPT_SHAREDPREFIX=0 \
    winetricks -q d3dcompiler_47 dxvk vcrun2022
  wineserver -w
  if (( APPLY_CONTROLLER_FIX )); then
    apply_controller_fix_overrides
  fi
}

install_dotnet() {
  [[ -n "$DOTNET_INSTALLER" ]] || die "Missing --dotnet-installer path"
  [[ -f "$DOTNET_INSTALLER" ]] || die "Installer not found: $DOTNET_INSTALLER"
  [[ -n "$PREFIX_PATH" ]] || die "Select a prefix before installing .NET"
  log "Installing optional .NET runtime into $PREFIX_PATH"
  export WINEPREFIX="$PREFIX_PATH"
  export WINEDEBUG=-all
  wine "$DOTNET_INSTALLER"
  wineserver -w
}

if (( !INSTALL_WINE && !VERIFY_ONLY )) && [[ -z "$PREFIX_PATH" ]] && [[ -z "$DOTNET_INSTALLER" ]]; then
  usage
  exit 1
fi

(( INSTALL_WINE )) && install_runtime_deps
[[ -n "$PREFIX_PATH" ]] && bootstrap_prefix "$PREFIX_PATH"
[[ -n "$DOTNET_INSTALLER" ]] && install_dotnet

verify_status=0
if (( INSTALL_WINE || VERIFY_ONLY )) || [[ -n "$PREFIX_PATH" ]] || [[ -n "$DOTNET_INSTALLER" ]]; then
  verify_runtime_setup || verify_status=1
fi
if (( verify_status != 0 )); then
  exit 1
fi
