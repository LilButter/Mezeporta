import { assetUrl } from "../store";

export const SETTINGS_SECTIONS = Object.freeze([
  { id: "launcher", labelKey: "settings-launcher-title", infoKey: "nav-launcher" },
  { id: "version", labelKey: "settings-version-title", infoKey: "nav-version" },
  { id: "settings", labelKey: "settings-screen-title", infoKey: "nav-settings" },
  { id: "graphics", labelKey: "settings-graphics-title", infoKey: "nav-graphics" },
  { id: "audio", labelKey: "settings-audio-title", infoKey: "nav-audio" },
  { id: "controls", labelKey: "settings-controls-title", infoKey: "nav-controls" },
  { id: "advanced", labelKey: "settings-advanced-title", infoKey: "nav-advanced" },
]);

export const SETTINGS_NAV_SPRITE_WIDTH = 803;
export const SETTINGS_NAV_TILE_WIDTH = 113;
export const SETTINGS_NAV_TILE_HEIGHT = 70;
export const SETTINGS_NAV_SOURCE_OFFSETS = Object.freeze([0, 113, 226, 339, 458, 577, 695]);

export const SETTINGS_NAV_BASE_IMAGE = assetUrl("/extra/SettingButtons/SettingNavSelected.png");
export const SETTINGS_NAV_ACTIVE_IMAGE = assetUrl("/extra/SettingButtons/SettingNav.png");
export const SETTINGS_DESCRIPTION_SPLIT_URL = assetUrl("/extra/SettingButtons/DescriptionSplit.png");

export const GAME_BRANCH_INFO_ICONS = Object.freeze([
  {
    id: "online",
    src: assetUrl("/extra/SettingButtons/Online.ico"),
    labelKey: "branch-online-label",
    labelFallback: "Online",
  },
  {
    id: "forward",
    src: assetUrl("/extra/SettingButtons/Forward.ico"),
    labelKey: "branch-forward-label",
    labelFallback: "Forward",
  },
  {
    id: "g",
    src: assetUrl("/extra/SettingButtons/G.ico"),
    labelKey: "branch-g-label",
    labelFallback: "G",
  },
  {
    id: "z",
    src: assetUrl("/extra/SettingButtons/Z.ico"),
    labelKey: "branch-z-label",
    labelFallback: "Z",
  },
]);

export const GAME_VERSION_DISPLAY_LABELS = Object.freeze({
  Z2T: "Z2TW",
});

export const GAME_VERSION_BRANCH_LABELS = Object.freeze({
  S6: "Online",
  S7K: "Online",
  F4: "Forward",
  F5: "Forward",
  G1: "G",
  G2: "G",
  G3: "G",
  "G3.1": "G",
  "G3.2": "G",
  GG: "G",
  G5: "G",
  "G5.1": "G",
  "G5.2": "G",
  G6: "G",
  G7: "G",
  "G9.1": "G",
  "G10.1": "G",
  Z1: "Z",
  Z2: "Z",
  Z2T: "Z",
  ZZ: "Z",
});

export const SETTINGS_INFO_BODY_LOREM =
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

export function versionDisplayLabel(version) {
  return GAME_VERSION_DISPLAY_LABELS[version] ?? version;
}

export function versionInfoKey(version) {
  return `game-version-${String(version)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")}`;
}

export function versionImageFileName(version) {
  return versionDisplayLabel(version);
}

export function versionInfoImageUrl(version) {
  return assetUrl(`/extra/SettingButtons/Versions/${versionImageFileName(version)}.png`);
}

export function versionInfoBodyFallback() {
  return SETTINGS_INFO_BODY_LOREM;
}

export const GAME_VERSION_INFO_IMAGES = Object.freeze(
  Object.fromEntries(
    Object.keys(GAME_VERSION_BRANCH_LABELS).map((version) => [
      String(version).toLowerCase(),
      {
        id: String(version).toLowerCase(),
        rawVersion: version,
        display: versionDisplayLabel(version),
        src: versionInfoImageUrl(version),
      },
    ])
  )
);

export const GAME_VERSION_INFO_ENTRIES = Object.freeze(
  Object.fromEntries(
    Object.keys(GAME_VERSION_BRANCH_LABELS).map((version) => [
      versionInfoKey(version),
      {
        titleKey: null,
        titleFallback: versionDisplayLabel(version),
        bodyKey: `settings-info-${versionInfoKey(version)}-body`,
        bodyFallback: versionInfoBodyFallback(version),
        versionImageKey: String(version).toLowerCase(),
      },
    ])
  )
);

export const SETTINGS_INFO = Object.freeze({
  "nav-launcher": {
    titleKey: "settings-launcher-title",
    titleFallback: "Launcher",
    bodyKey: "settings-info-nav-launcher-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-version": {
    titleKey: "settings-version-title",
    titleFallback: "Version",
    bodyKey: "settings-info-nav-version-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-settings": {
    titleKey: "settings-screen-title",
    titleFallback: "Screen",
    bodyKey: "settings-info-nav-settings-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-graphics": {
    titleKey: "settings-graphics-title",
    titleFallback: "Graphics",
    bodyKey: "settings-info-nav-graphics-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-audio": {
    titleKey: "settings-audio-title",
    titleFallback: "Audio",
    bodyKey: "settings-info-nav-audio-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-controls": {
    titleKey: "settings-controls-title",
    titleFallback: "Controls",
    bodyKey: "settings-info-nav-controls-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "nav-advanced": {
    titleKey: "settings-advanced-title",
    titleFallback: "Advanced",
    bodyKey: "settings-info-nav-advanced-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "launcher-theme": {
    titleKey: "style-label",
    titleFallback: "Theme",
    bodyKey: "settings-info-launcher-theme-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "font-selection": {
    titleKey: "font-style-label",
    titleFallback: "Font",
    bodyKey: "settings-info-font-selection-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "launcher-resolution": {
    titleKey: "launcher-resolution-label",
    titleFallback: "Launcher Resolution",
    bodyKey: "settings-info-launcher-resolution-body",
    bodyFallback: "Window size for the launcher only.",
  },
  "launcher-custom-resolution": {
    titleKey: "custom-resolution-label",
    titleFallback: "Resolution",
    bodyKey: "settings-info-launcher-custom-resolution-body",
    bodyFallback:
      "Type a custom launcher width and height for the current style.",
  },
  "launcher-resolution-reset": {
    titleKey: "launcher-resolution-reset-label",
    titleFallback: "Reset",
    bodyKey: "settings-info-launcher-resolution-reset-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "launcher-controller": {
    titleKey: "launcher-controller-label",
    titleFallback: "Controller",
    bodyKey: "settings-info-launcher-controller-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "launcher-hardware-acceleration": {
    titleKey: "launcher-hardware-acceleration-label",
    titleFallback: "Hardware Acceleration",
    bodyKey: "settings-info-launcher-hardware-acceleration-body",
    bodyFallback: "Linux GPU rendering. Turn off only for display issues. Restart required.",
  },
  "launcher-sfx": {
    titleKey: "launcher-sfx-label",
    titleFallback: "Sound Effects",
    bodyKey: "settings-info-launcher-sfx-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "launcher-sfx-volume": {
    titleKey: "sfx-volume-label",
    titleFallback: "UI sound volume",
    bodyKey: "settings-info-launcher-sfx-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-branch": {
    titleKey: "game-branch-label",
    titleFallback: "Branch",
    bodyKey: "settings-info-game-branch-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-branch-online": {
    titleKey: "branch-online-label",
    titleFallback: "Online",
    bodyKey: "settings-info-game-branch-online-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
    iconGroup: "game-branches",
    activeBranch: "online",
  },
  "game-branch-forward": {
    titleKey: "branch-forward-label",
    titleFallback: "Forward",
    bodyKey: "settings-info-game-branch-forward-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
    iconGroup: "game-branches",
    activeBranch: "forward",
  },
  "game-branch-g": {
    titleKey: "branch-g-label",
    titleFallback: "G",
    bodyKey: "settings-info-game-branch-g-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
    iconGroup: "game-branches",
    activeBranch: "g",
  },
  "game-branch-z": {
    titleKey: "branch-z-label",
    titleFallback: "Z",
    bodyKey: "settings-info-game-branch-z-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
    iconGroup: "game-branches",
    activeBranch: "z",
  },
  "game-version": {
    titleKey: "game-version-label",
    titleFallback: "Version",
    bodyKey: "settings-info-game-version-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  ...GAME_VERSION_INFO_ENTRIES,
  "friend-signature": {
    titleKey: "friend-signature-label",
    titleFallback: "Signature",
    bodyKey: "settings-info-friend-signature-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "hd-version": {
    titleKey: "hd-version-label",
    titleFallback: "HD Mode",
    bodyKey: "settings-info-hd-version-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-folder": {
    titleKey: "game-folder-label",
    titleFallback: "Location",
    bodyKey: "settings-info-game-folder-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-folder-browse": {
    titleKey: "browse-folder-label",
    titleFallback: "Browse Folder",
    bodyKey: "settings-info-game-folder-browse-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-folder-edit": {
    titleKey: "edit-folder-label",
    titleFallback: "Edit Path",
    bodyKey: "settings-info-game-folder-edit-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "brightness": {
    titleKey: "brightness-label",
    titleFallback: "Brightness",
    bodyKey: "settings-info-brightness-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "display-mode": {
    titleKey: "fullscreen-label",
    titleFallback: "Display Mode",
    bodyKey: "settings-info-display-mode-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "window-resolution": {
    titleKey: "window-resolution-label",
    titleFallback: "Resolution",
    bodyKey: "settings-info-window-resolution-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "window-custom-resolution": {
    titleKey: "custom-resolution-label",
    titleFallback: "Resolution",
    bodyKey: "settings-info-window-custom-resolution-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "fullscreen-resolution": {
    titleKey: "fullscreen-resolution-label",
    titleFallback: "Resolution",
    bodyKey: "settings-info-fullscreen-resolution-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "fullscreen-custom-resolution": {
    titleKey: "custom-resolution-label",
    titleFallback: "Custom Size",
    bodyKey: "settings-info-fullscreen-custom-resolution-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "match-monitor-resolution": {
    titleKey: "match-monitor-resolution-label",
    titleFallback: "Match monitor resolution",
    bodyKey: "settings-info-match-monitor-resolution-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "texture-compression": {
    titleKey: "texture-compression-label",
    titleFallback: "Texture compression",
    bodyKey: "settings-info-texture-compression-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "max-char-display": {
    titleKey: "max-char-display-label",
    titleFallback: "Max character display",
    bodyKey: "settings-info-max-char-display-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "hd-advanced-toggle": {
    titleKey: "advance-settings-label",
    titleFallback: "Advance Settings",
    bodyKey: "settings-info-hd-advanced-toggle-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "hd-graphics-toggle": {
    titleKey: "hd-version-settings-label",
    titleFallback: "HD Mode Settings",
    bodyKey: "settings-info-hd-graphics-toggle-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "hd-graphics-numeric": {
    titleKey: "hd-version-settings-label",
    titleFallback: "HD Mode Settings",
    bodyKey: "settings-info-hd-graphics-numeric-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "disable-sound-output": {
    titleKey: "disable-sound-output-label",
    titleFallback: "Disable sound output",
    bodyKey: "settings-info-disable-sound-output-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "general-volume": {
    titleKey: "general-volume-label",
    titleFallback: "General volume",
    bodyKey: "settings-info-general-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "unfocused-volume": {
    titleKey: "unfocused-volume-label",
    titleFallback: "Unfocused volume",
    bodyKey: "settings-info-unfocused-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "minimized-volume": {
    titleKey: "minimized-volume-label",
    titleFallback: "Minimized volume",
    bodyKey: "settings-info-minimized-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-bgm-volume": {
    titleKey: "game-bgm-volume-label",
    titleFallback: "BGM volume",
    bodyKey: "settings-info-game-bgm-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "game-se-volume": {
    titleKey: "game-se-volume-label",
    titleFallback: "SFX volume",
    bodyKey: "settings-info-game-se-volume-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "sound-frequency": {
    titleKey: "sound-frequency-label",
    titleFallback: "Sample rate",
    bodyKey: "settings-info-sound-frequency-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "sound-buffer-num": {
    titleKey: "sound-buffer-num-label",
    titleFallback: "Buffer size",
    bodyKey: "settings-info-sound-buffer-num-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "controller-vibration": {
    titleKey: "controller-vibration-label",
    titleFallback: "Vibration",
    bodyKey: "settings-info-controller-vibration-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "controller-fix": {
    titleKey: "disable-xinput-preload-label",
    titleFallback: "R-Analog Patch",
    bodyKey: "settings-info-controller-fix-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "dev-mode": {
    titleKey: "dev-mode-label",
    titleFallback: "Dev Mode",
    bodyKey: "settings-info-dev-mode-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "offline-images": {
    titleKey: "offline-images-label",
    titleFallback: "Offline-Images",
    bodyKey: "settings-info-offline-images-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
  "wine-prefix-mode": {
    titleKey: "wine-prefix-label",
    titleFallback: "Wine Prefix",
    bodyKey: "settings-info-wine-prefix-mode-body",
    bodyFallback: "Choose the Wine prefix mode used by Linux launches.",
  },
  "wine-prefix-portable": {
    titleKey: "wine-prefix-portable-label",
    titleFallback: "Portable (Mezeporta)",
    bodyKey: "settings-info-wine-prefix-portable-body",
    bodyFallback: "Uses Mezeporta/WinePrefix inside this game folder.",
  },
  "wine-prefix-system": {
    titleKey: "wine-prefix-system-label",
    titleFallback: "System Wine Prefix",
    bodyKey: "settings-info-wine-prefix-system-body",
    bodyFallback: "Uses your default Wine prefix.",
  },
  "wine-prefix-proton": {
    titleKey: "wine-prefix-proton-label",
    titleFallback: "Proton",
    bodyKey: "settings-info-wine-prefix-proton-body",
    bodyFallback:
      "For launching through Steam as a non-Steam game. Steam manages Proton and its compat data.",
  },
  "wine-prefix-custom": {
    titleKey: "wine-prefix-custom-label",
    titleFallback: "Custom Prefix",
    bodyKey: "settings-info-wine-prefix-custom-body",
    bodyFallback: "Uses a Wine prefix folder you choose.",
  },
  "wine-prefix-custom-path": {
    titleKey: "wine-prefix-custom-path-label",
    titleFallback: "Custom Prefix Path",
    bodyKey: "settings-info-wine-prefix-custom-path-body",
    bodyFallback: "Folder used when Custom Prefix is selected.",
  },
  "wine-prefix-custom-browse": {
    titleKey: "wine-prefix-browse-label",
    titleFallback: "Browse",
    bodyKey: "settings-info-wine-prefix-custom-browse-body",
    bodyFallback: "Browse to an existing Wine prefix folder.",
  },
  "reset-patch": {
    titleKey: "reset-patch-label",
    titleFallback: "Reset Patch",
    bodyKey: "settings-info-reset-patch-body",
    bodyFallback: SETTINGS_INFO_BODY_LOREM,
  },
});

