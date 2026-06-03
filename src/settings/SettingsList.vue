<script setup>
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { readDir } from "@tauri-apps/api/fs";
import { platform } from "@tauri-apps/api/os";
import { join } from "@tauri-apps/api/path";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";
import { computed, getCurrentInstance, nextTick, onMounted, reactive, ref, watch } from "vue";
import {
  CLASSIC_STYLE,
  formatResolutionValue,
  getLauncherCustomResolutionPrefKey,
  getLauncherWindowDefaults,
  parseResolutionValue,
  isLauncherWindowResizable,
  PS4_STYLE,
  CHARACTERS_PAGE,
} from "../common";
import {
  storeMut,
  effectiveFolder,
  store,
  assetUrl,
  setSetting,
  setUiPref,
  setRange,
  setGameVersion,
  refreshLinuxPrefixStatus,
  dialogOpenLinuxPrefixInstall,
  getStoredLauncherResolution,
  getStoredLauncherRecentResolutions,
  rememberLauncherResolution,
  setLauncherPrefs,
  friendSignatureEntriesForVersion,
  friendSignatureDisplayLabel,
} from "../store";
import SettingsItem from "./SettingsItem.vue";
import SettingsCheckbox from "./SettingsCheckbox.vue";
import SettingsButton from "./SettingsButton.vue";
import SettingsDropdown from "./SettingsDropdown.vue";
import { versionInfoKey } from "./settingsInfo";
import { settingsPanelState } from "./settingsPanelState";
import { playHover, playSelect } from "../sfx";

const vueInstance = getCurrentInstance();
function t(key, fallback = key) {
  return String(vueInstance?.proxy?.$t?.(key, fallback) ?? fallback);
}

const HD_CAPABLE_VERSIONS = ["G6", "G7", "G9.1", "G10.1", "Z1", "Z2", "Z2T", "ZZ"];
const GAME_BRANCHES = ["Online", "Forward", "G", "Z"];
const GAME_BRANCH_VERSIONS = {
  Online: ["S6", "S7K"],
  Forward: ["F4", "F5"],
  G: ["G1", "G2", "G3", "G3.1", "G3.2", "GG", "G5", "G5.1", "G5.2", "G6", "G7", "G9.1", "G10.1"],
  Z: ["Z1", "Z2", "Z2T", "ZZ"],
};
const DEV_ONLY_GAME_VERSIONS = ["G5", "Z2", "Z2T"];
const GAME_VERSION_DISPLAY_LABELS = {
  Z2T: "Z2TW",
};
const SOUND_LEVEL_OPTIONS = Array.from({ length: 8 }, (_, index) => index);
const SAMPLE_RATE_OPTIONS = [11025, 22050, 44100, 48000];
const RESOLUTION_OPTIONS = [
  { width: 640, height: 480, value: "640x480", label: "640x480" },
  { width: 800, height: 480, value: "800x480", label: "800x480" },
  { width: 800, height: 600, value: "800x600", label: "800x600" },
  { width: 1024, height: 480, value: "1024x480", label: "1024x480" },
  { width: 1024, height: 600, value: "1024x600", label: "1024x600" },
  { width: 1024, height: 768, value: "1024x768", label: "1024x768" },
  { width: 1088, height: 612, value: "1088x612", label: "1088x612" },
  { width: 1152, height: 648, value: "1152x648", label: "1152x648" },
  { width: 1152, height: 864, value: "1152x864", label: "1152x864" },
  { width: 1280, height: 600, value: "1280x600", label: "1280x600" },
  { width: 1280, height: 720, value: "1280x720", label: "1280x720" },
  { width: 1280, height: 768, value: "1280x768", label: "1280x768" },
  { width: 1280, height: 800, value: "1280x800", label: "1280x800" },
  { width: 1280, height: 854, value: "1280x854", label: "1280x854" },
  { width: 1280, height: 960, value: "1280x960", label: "1280x960" },
  { width: 1280, height: 1024, value: "1280x1024", label: "1280x1024" },
  { width: 1366, height: 768, value: "1366x768", label: "1366x768" },
  { width: 1400, height: 600, value: "1400x600", label: "1400x600" },
  { width: 1400, height: 1050, value: "1400x1050", label: "1400x1050" },
  { width: 1440, height: 900, value: "1440x900", label: "1440x900" },
  { width: 1450, height: 1050, value: "1450x1050", label: "1450x1050" },
  { width: 1500, height: 900, value: "1500x900", label: "1500x900" },
  { width: 1600, height: 900, value: "1600x900", label: "1600x900" },
  { width: 1600, height: 1200, value: "1600x1200", label: "1600x1200" },
  { width: 1680, height: 1050, value: "1680x1050", label: "1680x1050" },
  { width: 1920, height: 1080, value: "1920x1080", label: "1920x1080" },
  { width: 1920, height: 1200, value: "1920x1200", label: "1920x1200" },
  { width: 2048, height: 960, value: "2048x960", label: "2048x960" },
  { width: 2048, height: 1080, value: "2048x1080", label: "2048x1080" },
  { width: 2048, height: 1152, value: "2048x1152", label: "2048x1152" },
  { width: 2560, height: 1080, value: "2560x1080", label: "2560x1080" },
  { width: 2560, height: 1440, value: "2560x1440", label: "2560x1440" },
  { width: 3440, height: 1440, value: "3440x1440", label: "3440x1440" },
  { width: 3840, height: 2160, value: "3840x2160", label: "3840x2160" },
];
const hdToggleSettings = [
  { key: "hdGraphicShadowQuest", labelKey: "hdGraphicShadowQuest-label", fallback: "Quest shadows" },
  { key: "hdGraphicShadowLobby", labelKey: "hdGraphicShadowLobby-label", fallback: "Lobby shadows" },
  { key: "hdGraphicDof", labelKey: "hdGraphicDof-label", fallback: "Depth of field" },
  { key: "hdGraphicBloom", labelKey: "hdGraphicBloom-label", fallback: "Bloom" },
  { key: "hdGraphicSsao", labelKey: "hdGraphicSsao-label", fallback: "SSAO" },
  { key: "hdGraphicGodray", labelKey: "hdGraphicGodray-label", fallback: "Godray" },
  { key: "hdGraphicAntiAliasing", labelKey: "hdGraphicAntiAliasing-label", fallback: "Anti-aliasing" },
  { key: "hdGraphicSoftParticle", labelKey: "hdGraphicSoftParticle-label", fallback: "Soft particles" },
];
const hdNumericSettings = [
  { key: "hdGraphicDofFarBlurSize", labelKey: "hdGraphicDofFarBlurSize-label", fallback: "DOF far blur size" },
  { key: "hdGraphicBloomDispersion", labelKey: "hdGraphicBloomDispersion-label", fallback: "Bloom dispersion" },
  { key: "hdGraphicBloomThreshold", labelKey: "hdGraphicBloomThreshold-label", fallback: "Bloom threshold" },
  { key: "hdGraphicBloomColor", labelKey: "hdGraphicBloomColor-label", fallback: "Bloom color" },
  { key: "hdGraphicGaussianBlurDispersion", labelKey: "hdGraphicGaussianBlurDispersion-label", fallback: "Gaussian blur dispersion" },
  { key: "hdGraphicGaussianBlurBlendRate", labelKey: "hdGraphicGaussianBlurBlendRate-label", fallback: "Gaussian blur blend rate" },
  { key: "hdGraphicSsaoDensity", labelKey: "hdGraphicSsaoDensity-label", fallback: "SSAO density" },
  { key: "hdGraphicShadowmapColor", labelKey: "hdGraphicShadowmapColor-label", fallback: "Shadowmap color" },
  { key: "hdGraphicPlLightShadowAttenuation", labelKey: "hdGraphicPlLightShadowAttenuation-label", fallback: "Player light shadow attenuation" },
  { key: "hdGraphicBgLightShadowAttenuation", labelKey: "hdGraphicBgLightShadowAttenuation-label", fallback: "Background light shadow attenuation" },
  { key: "hdGraphicAntiAliasingWeightScale", labelKey: "hdGraphicAntiAliasingWeightScale-label", fallback: "Anti-aliasing weight scale" },
];
const sessionSettingsLocked = computed(
  () => storeMut.settingsReturnPage === CHARACTERS_PAGE && store.characters.length > 0
);
const settingsLockTooltip = computed(() =>
  t("settings-session-lock-tooltip", "Log-out to adjust")
);

async function onChooseFolder() {
  if (sessionSettingsLocked.value) return;
  const folder = await open({ directory: true });
  if (typeof folder === "string") {
    storeMut.gameFolder = folder;
    isEditingGameFolder.value = false;
    syncGameFolderDraft();
  }
}

async function onChooseWinePrefixFolder() {
  const defaultPath =
    store.settings.winePrefixCustomPath ?? effectiveFolder.value ?? undefined;
  const folder = await open({
    directory: true,
    defaultPath,
  });
  if (typeof folder === "string") {
    await setLauncherPrefs({
      winePrefixMode: "custom",
      winePrefixCustomPath: folder,
    });
    syncWinePrefixCustomDraft();
    isEditingWinePrefixCustomPath.value = false;
  }
}

const gameFolderDisplayValue = computed(() => {
  const value = String(effectiveFolder.value ?? "").trim();
  return value || t("game-folder-empty-label", "No folder selected");
});
const gameFolderPlaceholder = computed(() =>
  String(effectiveFolder.value ?? "").trim()
);
const gameFolderInputRef = ref(null);
const isEditingGameFolder = ref(false);
const gameFolderDraft = ref("");
const winePrefixCustomDisplayValue = computed(() => {
  const value = String(store.settings.winePrefixCustomPath ?? "").trim();
  return value || t("game-folder-empty-label", "No folder selected");
});
const winePrefixCustomPlaceholder = computed(() =>
  String(store.settings.winePrefixCustomPath ?? "").trim()
);
const winePrefixCustomInputRef = ref(null);
const isEditingWinePrefixCustomPath = ref(false);
const winePrefixCustomDraft = ref("");
const isLinuxHost = ref(false);
const hostPlatformReady = ref(false);
const controllerDllsMissing = ref(false);
const controllerDllsMissingTooltip = computed(() =>
  t("controller-dlls-not-found-tooltip", "dll's not found")
);
const shouldManageControllerDllFiles = computed(
  () => hostPlatformReady.value && !isLinuxHost.value
);
let controllerDllSyncNonce = 0;

function syncGameFolderDraft() {
  gameFolderDraft.value = String(storeMut.gameFolder ?? "").trim();
}

function normalizeGameFolderInput(value) {
  const normalized = String(value ?? "").trim();
  return normalized || null;
}

async function beginGameFolderEdit() {
  if (sessionSettingsLocked.value) return;
  syncGameFolderDraft();
  isEditingGameFolder.value = true;
  await nextTick();
  gameFolderInputRef.value?.focus?.();
  gameFolderInputRef.value?.select?.();
}

function commitGameFolderEdit() {
  if (sessionSettingsLocked.value) return;
  storeMut.gameFolder = normalizeGameFolderInput(gameFolderDraft.value);
  isEditingGameFolder.value = false;
}

function cancelGameFolderEdit() {
  syncGameFolderDraft();
  isEditingGameFolder.value = false;
}

async function onGameFolderEditButton() {
  if (sessionSettingsLocked.value) return;
  if (isEditingGameFolder.value) {
    commitGameFolderEdit();
    return;
  }
  await beginGameFolderEdit();
}

function syncWinePrefixCustomDraft() {
  winePrefixCustomDraft.value = String(store.settings.winePrefixCustomPath ?? "").trim();
}

function normalizeWinePrefixCustomInput(value) {
  const normalized = String(value ?? "").trim();
  return normalized || null;
}

async function beginWinePrefixCustomEdit() {
  syncWinePrefixCustomDraft();
  isEditingWinePrefixCustomPath.value = true;
  await nextTick();
  winePrefixCustomInputRef.value?.focus?.();
  winePrefixCustomInputRef.value?.select?.();
}

async function commitWinePrefixCustomEdit() {
  await setLauncherPrefs({
    winePrefixMode: "custom",
    winePrefixCustomPath: normalizeWinePrefixCustomInput(winePrefixCustomDraft.value),
  });
  isEditingWinePrefixCustomPath.value = false;
}

function cancelWinePrefixCustomEdit() {
  syncWinePrefixCustomDraft();
  isEditingWinePrefixCustomPath.value = false;
}

async function onWinePrefixCustomEditButton() {
  if (isEditingWinePrefixCustomPath.value) {
    await commitWinePrefixCustomEdit();
    return;
  }
  await beginWinePrefixCustomEdit();
}

const customFontPresets = ref([]);

function isCustomFontFile(fileName) {
  return /\.(ttf|ttc|otf)$/i.test(String(fileName ?? ""));
}

async function refreshCustomFontPresets() {
  customFontPresets.value = [];

  if (!hostPlatformReady.value) return;

  const folder = effectiveFolder.value;
  if (!folder) return;

  let customDirs = [];
  try {
    customDirs = [
      await join(folder, "Mezeporta", "fonts", "Custom"),
      await join(folder, "fonts", "Custom"),
    ];
  } catch (_error) {
    return;
  }

  for (const customDir of customDirs) {
    let entries = [];
    try {
      entries = await readDir(customDir);
    } catch (_error) {
      continue;
    }

    const presets = entries
      .filter((entry) => !entry?.children && typeof entry?.name === "string")
      .map((entry) => entry.name)
      .filter((name) => isCustomFontFile(name))
      .map((name) => ({
        value: `custom:${name}`,
        label: String(name),
      }))
      .sort((a, b) => a.label.localeCompare(b.label));

    if (presets.length > 0) {
      customFontPresets.value = presets;
      return;
    }
  }
}

watch(
  () => [hostPlatformReady.value, effectiveFolder.value],
  ([ready]) => {
    if (!ready) return;
    refreshCustomFontPresets().catch(() => {
      customFontPresets.value = [];
    });
  },
  { immediate: true }
);

async function controllerDllState(enabled, applyChanges = false) {
  if (!shouldManageControllerDllFiles.value) {
    return { available: true, files: [] };
  }

  const folder = String(effectiveFolder.value ?? "").trim();
  if (!folder) return { available: false, files: [] };

  try {
    return await invoke("sync_controller_dll_files", {
      gameFolder: folder,
      enabled: Boolean(enabled),
      applyChanges: Boolean(applyChanges),
    });
  } catch (_error) {
    return { available: false, files: [] };
  }
}

async function syncControllerDllState(enabled, persistPreference = false) {
  if (!hostPlatformReady.value) {
    controllerDllsMissing.value = false;
    return false;
  }

  if (!shouldManageControllerDllFiles.value) {
    controllerDllsMissing.value = false;
    if (persistPreference) {
      await setLauncherPrefs({ preloadControllerDlls: Boolean(enabled) });
    }
    return true;
  }

  const nonce = ++controllerDllSyncNonce;
  const state = await controllerDllState(enabled, true);
  if (nonce !== controllerDllSyncNonce) return false;

  controllerDllsMissing.value = !state.available;
  if (!state.available) {
    if (store.settings.preloadControllerDlls) {
      await setLauncherPrefs({ preloadControllerDlls: false });
    }
    return false;
  }

  if (persistPreference && !controllerDllsMissing.value) {
    await setLauncherPrefs({ preloadControllerDlls: Boolean(enabled) });
  }
  return !controllerDllsMissing.value;
}

function onControllerDllToggle(value) {
  syncControllerDllState(Boolean(value), true).catch(() => undefined);
}

watch(
  () => storeMut.gameFolder,
  () => {
    if (!isEditingGameFolder.value) {
      syncGameFolderDraft();
    }
  },
  { immediate: true }
);

watch(
  () => store.settings.winePrefixCustomPath,
  () => {
    if (!isEditingWinePrefixCustomPath.value) {
      syncWinePrefixCustomDraft();
    }
  },
  { immediate: true }
);

const draft = reactive({
  launcherW: "",
  launcherH: "",
  windowW: String(store.settings.windowW),
  windowH: String(store.settings.windowH),
  fullscreenW: String(store.settings.fullscreenW),
  fullscreenH: String(store.settings.fullscreenH),
  maxCharDisplay: String(store.settings.maxCharDisplay),
  soundBufferNum: String(store.settings.soundBufferNum),
});

watch(
  () => [
    store.settings.windowW,
    store.settings.windowH,
    store.settings.fullscreenW,
    store.settings.fullscreenH,
    store.settings.maxCharDisplay,
    store.settings.soundBufferNum,
  ],
  ([windowW, windowH, fullscreenW, fullscreenH, maxCharDisplay, soundBufferNum]) => {
    draft.windowW = String(windowW);
    draft.windowH = String(windowH);
    draft.fullscreenW = String(fullscreenW);
    draft.fullscreenH = String(fullscreenH);
    draft.maxCharDisplay = String(maxCharDisplay);
    draft.soundBufferNum = String(soundBufferNum);
  }
);

const props = defineProps({
  activeSection: {
    type: String,
    default: null,
  },
});

const showLauncher = computed(() => !props.activeSection || props.activeSection === "launcher");
const showVersion = computed(() => !props.activeSection || props.activeSection === "version");
const showSettings = computed(() => !props.activeSection || props.activeSection === "settings");
const showGraphics = computed(() => !props.activeSection || props.activeSection === "graphics");
const showAudio = computed(() => !props.activeSection || props.activeSection === "audio");
const showControls = computed(() => !props.activeSection || props.activeSection === "controls");
const showAdvanced = computed(() => !props.activeSection || props.activeSection === "advanced");
const showLauncherResolutionSetting = computed(() =>
  isLauncherWindowResizable(storeMut.style)
);
const showLinuxHardwareAccelerationSetting = computed(
  () => isLinuxHost.value && store.settings.devMode
);
const showWinePrefixSettings = computed(() =>
  isLinuxHost.value &&
  (storeMut.style === CLASSIC_STYLE || storeMut.style === PS4_STYLE)
);
const showPortableWinePrefixStatus = computed(
  () =>
    showWinePrefixSettings.value &&
    store.settings.winePrefixMode === "portable"
);
const showCustomWinePrefixPath = computed(
  () => showWinePrefixSettings.value && store.settings.winePrefixMode === "custom"
);
watch(showCustomWinePrefixPath, (visible) => {
  if (!visible) {
    cancelWinePrefixCustomEdit();
  }
});

watch(
  () => store.settings.devMode,
  (devMode) => {
    if (!devMode && store.settings.winePrefixMode === "proton") {
      setLauncherPrefs({ winePrefixMode: "portable" });
    }
  },
  { immediate: true }
);

watch(
  [
    isLinuxHost,
    () => store.settings.devMode,
    () => store.settings.linuxHardwareAcceleration,
  ],
  ([linuxHost, devMode, hardwareAcceleration]) => {
    if (linuxHost && !devMode && hardwareAcceleration === false) {
      setUiPref("linuxHardwareAcceleration", true);
    }
  },
  { immediate: true }
);

function branchForGameVersion(version) {
  const normalized = String(version ?? "").trim();
  for (const branch of GAME_BRANCHES) {
    if ((GAME_BRANCH_VERSIONS[branch] ?? []).includes(normalized)) {
      return branch;
    }
  }
  return "Z";
}

function versionDisplayLabel(version) {
  return GAME_VERSION_DISPLAY_LABELS[version] ?? version;
}

const selectedBranch = ref(branchForGameVersion(store.settings.gameVersion));
const visibleGameVersions = computed(() => {
  const versions = GAME_BRANCH_VERSIONS[selectedBranch.value] ?? [];
  return versions.filter(
    (version) => store.settings.devMode || !DEV_ONLY_GAME_VERSIONS.includes(version)
  );
});

watch(
  () => store.settings.gameVersion,
  (version) => {
    const nextBranch = branchForGameVersion(version);
    if (selectedBranch.value !== nextBranch) {
      selectedBranch.value = nextBranch;
    }
  },
  { immediate: true }
);

watch(
  [selectedBranch, () => store.settings.devMode],
  () => {
    if (!visibleGameVersions.value.includes(store.settings.gameVersion)) {
      const fallbackVersion = visibleGameVersions.value[0];
      if (fallbackVersion) {
        setGameVersion(fallbackVersion);
      }
    }
  },
  { immediate: true }
);
const supportsHdVersion = computed(() => HD_CAPABLE_VERSIONS.includes(store.settings.gameVersion));
const supportsHdGraphics = computed(() => HD_CAPABLE_VERSIONS.includes(store.settings.gameVersion));
const hdGraphicsLocked = computed(() => supportsHdGraphics.value && !store.settings.hdVersion);
const showHdAdvancedSettings = computed({
  get: () => settingsPanelState.showHdAdvancedSettings,
  set: (value) => {
    settingsPanelState.showHdAdvancedSettings = Boolean(value);
  },
});

function clampInt(value, min, max, fallback = min) {
  const parsed = Number.parseInt(String(value ?? "").trim(), 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function commitPositiveDraft(name) {
  const parsed = Number.parseInt(String(draft[name] ?? "").trim(), 10);
  if (Number.isFinite(parsed) && parsed > 0) {
    setSetting(name, parsed);
    draft[name] = String(parsed);
  } else {
    draft[name] = String(store.settings[name]);
  }
}

function commitClampedDraft(name, min, max) {
  const parsed = Number.parseInt(String(draft[name] ?? "").trim(), 10);
  if (Number.isFinite(parsed)) {
    const next = Math.min(max, Math.max(min, parsed));
    setSetting(name, next);
    draft[name] = String(next);
  } else {
    draft[name] = String(store.settings[name]);
  }
}

function clampBrightnessSlider(value) {
  const parsed = Number.parseInt(String(value ?? 0), 10);
  if (!Number.isFinite(parsed)) return 0;
  return Math.max(-35, Math.min(35, parsed));
}

const BRIGHTNESS_CONVERSION_TABLE = Array.from({ length: 71 }, (_, index) => {
  const slider = index - 35;
  const brightSigned = slider >= 0 ? -128 + slider : -128 + slider * 2;
  return {
    slider,
    brightSigned,
    brightByte: brightSigned & 0xff,
  };
});

const BRIGHTNESS_SLIDER_BY_SIGNED = new Map(
  BRIGHTNESS_CONVERSION_TABLE.map((entry) => [entry.brightSigned, entry.slider])
);

const BRIGHTNESS_SLIDER_BY_BYTE = new Map(
  BRIGHTNESS_CONVERSION_TABLE.map((entry) => [entry.brightByte, entry.slider])
);

function sliderToBrightSigned(sliderValue) {
  const slider = clampBrightnessSlider(sliderValue);
  return BRIGHTNESS_CONVERSION_TABLE[slider + 35].brightSigned;
}

function sliderToBrightByte(sliderValue) {
  const slider = clampBrightnessSlider(sliderValue);
  return BRIGHTNESS_CONVERSION_TABLE[slider + 35].brightByte;
}

function sliderToBright(sliderValue) {
  return sliderToBrightByte(sliderValue);
}

function brightToSlider(brightValue) {
  const numericValue = Number(brightValue);
  if (!Number.isFinite(numericValue)) return 0;

  const rounded = Math.round(numericValue);
  const signedMatch = BRIGHTNESS_SLIDER_BY_SIGNED.get(rounded);
  if (signedMatch !== undefined) return signedMatch;

  const byteMatch = BRIGHTNESS_SLIDER_BY_BYTE.get(rounded & 0xff);
  if (byteMatch !== undefined) return byteMatch;

  return 0;
}

function formatSignedNumber(value) {
  if (value > 0) return `+${value}`;
  return `${value}`;
}

const brightnessSliderValue = computed(() =>
  brightToSlider(store.settings.brightness)
);

const BRIGHTNESS_PREVIEW_LUT = new Map([
  [-35, { brightness: 0.4329 }],
  [-34, { brightness: 0.4481 }],
  [-33, { brightness: 0.4633 }],
  [-32, { brightness: 0.4787 }],
  [-31, { brightness: 0.4942 }],
  [-30, { brightness: 0.5082 }],
  [-29, { brightness: 0.5241 }],
  [-28, { brightness: 0.5395 }],
  [-27, { brightness: 0.5539 }],
  [-26, { brightness: 0.5688 }],
  [-25, { brightness: 0.5841 }],
  [-24, { brightness: 0.6001 }],
  [-23, { brightness: 0.6148 }],
  [-22, { brightness: 0.6299 }],
  [-21, { brightness: 0.6449 }],
  [-20, { brightness: 0.6612 }],
  [-19, { brightness: 0.6741 }],
  [-18, { brightness: 0.6894 }],
  [-17, { brightness: 0.7044 }],
  [-16, { brightness: 0.7204 }],
  [-15, { brightness: 0.7345 }],
  [-14, { brightness: 0.7503 }],
  [-13, { brightness: 0.7649 }],
  [-12, { brightness: 0.7813 }],
  [-11, { brightness: 0.7954 }],
  [-10, { brightness: 0.8101 }],
  [-9, { brightness: 0.8253 }],
  [-8, { brightness: 0.8414 }],
  [-7, { brightness: 0.8559 }],
  [-6, { brightness: 0.8705 }],
  [-5, { brightness: 0.8871 }],
  [-4, { brightness: 0.9005 }],
  [-3, { brightness: 0.9139 }],
  [-2, { brightness: 0.9286 }],
  [-1, { brightness: 0.9451 }],
  [0, { brightness: 1 }],
  [1, { brightness: 0.9696 }],
  [2, { brightness: 0.9696 }],
  [3, { brightness: 0.9783 }],
  [4, { brightness: 0.9871 }],
  [5, { brightness: 0.9871 }],
  [6, { brightness: 0.9959 }],
  [7, { brightness: 1.0046 }],
  [8, { brightness: 1.0135 }],
  [9, { brightness: 1.0133 }],
  [10, { brightness: 1.0221 }],
  [11, { brightness: 1.031 }],
  [12, { brightness: 1.031 }],
  [13, { brightness: 1.0397 }],
  [14, { brightness: 1.0483 }],
  [15, { brightness: 1.0483 }],
  [16, { brightness: 1.057 }],
  [17, { brightness: 1.0657 }],
  [18, { brightness: 1.0745 }],
  [19, { brightness: 1.0745 }],
  [20, { brightness: 1.0832 }],
  [21, { brightness: 1.092 }],
  [22, { brightness: 1.092 }],
  [23, { brightness: 1.1007 }],
  [24, { brightness: 1.1094 }],
  [25, { brightness: 1.1094 }],
  [26, { brightness: 1.1182 }],
  [27, { brightness: 1.1269 }],
  [28, { brightness: 1.1356 }],
  [29, { brightness: 1.1356 }],
  [30, { brightness: 1.1444 }],
  [31, { brightness: 1.1531 }],
  [32, { brightness: 1.1531 }],
  [33, { brightness: 1.1618 }],
  [34, { brightness: 1.1706 }],
  [35, { brightness: 1.1706 }],
]);

const brightnessPreviewImageStyle = computed(() => {
  const step = BRIGHTNESS_PREVIEW_LUT.get(brightnessSliderValue.value) ?? BRIGHTNESS_PREVIEW_LUT.get(0);
  const brightness =
    brightnessSliderValue.value >= 0
      ? Math.max(1, step.brightness)
      : step.brightness;
  return {
    filter: `brightness(${brightness.toFixed(4)})`,
  };
});

function percentToIniVolume(percentValue) {
  const percent = clampInt(percentValue, 0, 100, 100);
  return Math.round(256 * (1 - percent / 100));
}

function iniVolumeToPercent(iniValue) {
  const value = clampInt(iniValue, 0, 256, 0);
  return Math.round((1 - value / 256) * 100);
}

function onGameBranchClick(branch) {
  if (sessionSettingsLocked.value) return;
  if (selectedBranch.value === branch) return;
  playSelect();
  selectedBranch.value = branch;
}

function onGameVersionClick(version) {
  if (sessionSettingsLocked.value) return;
  playSelect();
  setGameVersion(version);
}

function branchNode(branch) {
  return `settings-game-branch-${String(branch).toLowerCase()}`;
}

function versionNode(version) {
  return `settings-game-version-${String(version).toLowerCase()}`;
}

function previousBranchNode(branch) {
  const index = GAME_BRANCHES.indexOf(branch);
  return index > 0 ? branchNode(GAME_BRANCHES[index - 1]) : null;
}

function nextBranchNode(branch) {
  const index = GAME_BRANCHES.indexOf(branch);
  return index >= 0 && index < GAME_BRANCHES.length - 1
    ? branchNode(GAME_BRANCHES[index + 1])
    : null;
}

const visibleGameVersionNodeIds = computed(() =>
  visibleGameVersions.value.map((version) => versionNode(version))
);

const selectedBranchNodeId = computed(() => branchNode(selectedBranch.value));
const selectedGameVersionNodeId = computed(() =>
  visibleGameVersions.value.includes(store.settings.gameVersion)
    ? versionNode(store.settings.gameVersion)
    : visibleGameVersionNodeIds.value[0] ?? null
);

function previousGameVersionNode(version) {
  const versions = visibleGameVersions.value;
  const index = versions.indexOf(version);
  return index > 0 ? versionNode(versions[index - 1]) : null;
}

function nextGameVersionNode(version) {
  const versions = visibleGameVersions.value;
  const index = versions.indexOf(version);
  return index >= 0 && index < versions.length - 1
    ? versionNode(versions[index + 1])
    : null;
}

const friendSignatureEntries = computed(() =>
  friendSignatureEntriesForVersion(
    store.settings.gameVersion,
    store.settings.hdVersion
  )
);

const launcherStyleOptions = computed(() => {
  return [
    { value: CLASSIC_STYLE, label: t("classic-style") },
    { value: PS4_STYLE, label: t("ps4-style") },
  ];
});

const winePrefixModeOptions = computed(() => {
  const options = [
    {
      value: "portable",
      label: t("wine-prefix-portable-label", "Portable (Mezeporta)"),
      infoKey: "wine-prefix-portable",
    },
    {
      value: "system",
      label: t("wine-prefix-system-label", "System Wine Prefix"),
      infoKey: "wine-prefix-system",
    },
  ];

  if (store.settings.devMode) {
    options.push({
      value: "proton",
      label: t("wine-prefix-proton-label", "Proton"),
      infoKey: "wine-prefix-proton",
    });
  }

  options.push({
    value: "custom",
    label: t("wine-prefix-custom-label", "Custom Prefix"),
    infoKey: "wine-prefix-custom",
  });

  return options;
});

const fontPresetOptions = computed(() => [
  { value: "default", label: t("font-default-label") },
  { value: "classic", label: t("font-classic-label") },
  ...customFontPresets.value,
]);

const gameFolderControllerUpNode = computed(() =>
  supportsHdVersion.value ? "settings-hd-version" : "settings-friend-signature"
);

const friendSignatureOptions = computed(() => [
  {
    value: "none",
    label: t("friend-signature-none-label", "None / Disabled"),
  },
  ...friendSignatureEntries.value.map((entry) => ({
    value: entry.signature,
    label: `${friendSignatureDisplayLabel(entry.signature)}${entry.enabled ? "" : " (Unavailable)"}`,
    disabled: !entry.enabled,
  })),
]);

const soundLevelDropdownOptions = computed(() =>
  SOUND_LEVEL_OPTIONS.map((level) => ({
    value: level,
    label: String(level),
  }))
);

const sampleRateDropdownOptions = computed(() =>
  SAMPLE_RATE_OPTIONS.map((rate) => ({
    value: rate,
    label: String(rate),
  }))
);

function onFriendSignatureChange(value) {
  setLauncherPrefs({ friendSignature: value });
}

function onWinePrefixModeChange(value) {
  if (value !== "custom") {
    cancelWinePrefixCustomEdit();
  }
  Promise.resolve(setLauncherPrefs({ winePrefixMode: value })).finally(() => {
    if (value === "portable") {
      refreshLinuxPrefixStatus().catch(() => undefined);
    }
  });
}

const winePrefixCustomPathError = computed(() =>
  showCustomWinePrefixPath.value &&
  !String(store.settings.winePrefixCustomPath ?? "").trim()
    ? t(
        "wine-prefix-custom-path-required-label",
        "Choose or enter a Wine prefix folder before launching."
      )
    : ""
);
const winePrefixControllerUpNode = computed(() =>
  "settings-wine-prefix-mode"
);

function onHdVersionChange(value) {
  setSetting("hdVersion", value);
  setLauncherPrefs({ friendSignature: store.settings.friendSignature });
}

const launcherResolution = computed(() =>
  getStoredLauncherResolution(storeMut.style)
);
const launcherResolutionSelection = ref("");
const launcherRecentResolutionOptions = computed(() => {
  const defaults = getLauncherWindowDefaults(storeMut.style);
  const defaultValue = formatResolutionValue(defaults.width, defaults.height);
  const values = [defaultValue, ...getStoredLauncherRecentResolutions(storeMut.style)];
  const options = [];

  for (const value of values) {
    if (options.some((entry) => entry.value === value)) continue;
    options.push({ value, label: value });
  }

  return options;
});
const launcherCustomResolutionPrefKey = computed(() =>
  getLauncherCustomResolutionPrefKey(storeMut.style)
);
const launcherResolutionIsCustom = computed(() =>
  Boolean(
    launcherCustomResolutionPrefKey.value &&
      store.settings[launcherCustomResolutionPrefKey.value]
  )
);
const portableWinePrefixStatusLabel = computed(() => {
  if (store.linuxPrefixStatus.loading) {
    return t("wine-prefix-status-checking-label", "Checking Mezeporta prefix...");
  }
  if (store.linuxPrefixStatus.error) {
    return store.linuxPrefixStatus.error;
  }
  if (store.linuxPrefixStatus.missingTools?.length) {
    return `${t("wine-prefix-status-missing-tools-label", "Missing runtime tools")}: ${store.linuxPrefixStatus.missingTools.join(", ")}`;
  }
  if (store.linuxPrefixStatus.ready) {
    return t("wine-prefix-status-ready-label", "Mezeporta prefix ready");
  }
  return t("wine-prefix-status-missing-label", "Mezeporta prefix not found");
});
const portableWinePrefixStatusClass = computed(() =>
  store.linuxPrefixStatus.ready &&
  !store.linuxPrefixStatus.error &&
  !(store.linuxPrefixStatus.missingTools?.length)
    ? "settings-prefix-status-ready"
    : "settings-prefix-status-warning"
);

function syncLauncherResolutionDraft() {
  const currentValue = formatResolutionValue(
    launcherResolution.value.width,
    launcherResolution.value.height
  );
  draft.launcherW = String(launcherResolution.value.width);
  draft.launcherH = String(launcherResolution.value.height);
  const defaults = getLauncherWindowDefaults(storeMut.style);
  const defaultValue = formatResolutionValue(defaults.width, defaults.height);
  launcherResolutionSelection.value = launcherRecentResolutionOptions.value.some(
    (option) => option.value === currentValue
  )
    ? currentValue
    : defaultValue;
}

async function createLauncherPhysicalSize(width, height) {
  return new PhysicalSize(width, height);
}

async function applyLauncherResolutionSize(width, height) {
  const defaults = getLauncherWindowDefaults(storeMut.style);
  const normalizedWidth = Number.parseInt(String(width ?? "").trim(), 10);
  const normalizedHeight = Number.parseInt(String(height ?? "").trim(), 10);
  const nextWidth =
    Number.isFinite(normalizedWidth) && normalizedWidth > 0
      ? normalizedWidth
      : defaults.width;
  const nextHeight =
    Number.isFinite(normalizedHeight) && normalizedHeight > 0
      ? normalizedHeight
      : defaults.height;

  rememberLauncherResolution(storeMut.style, {
    width: nextWidth,
    height: nextHeight,
  });

  try {
    await appWindow.setSize(await createLauncherPhysicalSize(nextWidth, nextHeight));
  } catch (_error) {
    // keep the stored preference even if the native resize call fails
  }
}

async function onLauncherResolutionChange(value) {
  const parsed = parseResolutionValue(value);
  if (!parsed) return;
  if (launcherCustomResolutionPrefKey.value) {
    setUiPref(launcherCustomResolutionPrefKey.value, false);
  }
  launcherResolutionSelection.value = parsed.value;
  draft.launcherW = String(parsed.width);
  draft.launcherH = String(parsed.height);
  await applyLauncherResolutionSize(parsed.width, parsed.height);
}

async function saveLauncherResolution() {
  const width = Number.parseInt(String(draft.launcherW ?? "").trim(), 10);
  const height = Number.parseInt(String(draft.launcherH ?? "").trim(), 10);
  const parsed =
    Number.isFinite(width) && width > 0 && Number.isFinite(height) && height > 0
      ? parseResolutionValue(formatResolutionValue(width, height))
      : null;
  if (!parsed) {
    syncLauncherResolutionDraft();
    return;
  }
  if (launcherCustomResolutionPrefKey.value) {
    setUiPref(launcherCustomResolutionPrefKey.value, true);
  }
  await applyLauncherResolutionSize(parsed.width, parsed.height);
  draft.launcherW = String(parsed.width);
  draft.launcherH = String(parsed.height);
}

async function resetLauncherResolution() {
  const defaults = getLauncherWindowDefaults(storeMut.style);
  if (launcherCustomResolutionPrefKey.value) {
    setUiPref(launcherCustomResolutionPrefKey.value, false);
  }
  await applyLauncherResolutionSize(defaults.width, defaults.height);
  launcherResolutionSelection.value = formatResolutionValue(
    defaults.width,
    defaults.height
  );
}

function onLauncherCustomResolutionToggle(value) {
  if (!launcherCustomResolutionPrefKey.value) return;
  setUiPref(launcherCustomResolutionPrefKey.value, Boolean(value));
  syncLauncherResolutionDraft();
}

async function onLauncherCustomResolutionEditButton() {
  const enableCustom = !launcherResolutionIsCustom.value;
  if (!enableCustom) {
    await saveLauncherResolution();
  }
  onLauncherCustomResolutionToggle(enableCustom);
}

onMounted(async () => {
  try {
    isLinuxHost.value = (await platform()) === "linux";
  } catch (_error) {
    isLinuxHost.value = false;
  } finally {
    hostPlatformReady.value = true;
  }
  if (isLinuxHost.value && showPortableWinePrefixStatus.value) {
    refreshLinuxPrefixStatus().catch(() => undefined);
  }
});

watch(
  () => [
    storeMut.style,
    launcherResolution.value.width,
    launcherResolution.value.height,
    ...launcherRecentResolutionOptions.value.map((option) => option.value),
  ],
  () => {
    syncLauncherResolutionDraft();
  },
  { immediate: true }
);

watch(
  () => [showPortableWinePrefixStatus.value, effectiveFolder.value],
  ([visible]) => {
    if (visible) {
      refreshLinuxPrefixStatus().catch(() => undefined);
    }
  },
  { immediate: true }
);

watch(
  () => [
    hostPlatformReady.value,
    isLinuxHost.value,
    effectiveFolder.value,
    store.settings.preloadControllerDlls,
  ],
  ([ready]) => {
    if (!ready) return;
    syncControllerDllState(Boolean(store.settings.preloadControllerDlls)).catch(() => undefined);
  },
  { immediate: true }
);

function normalizeSfxVolume(value) {
  const parsed = Number.parseInt(String(value ?? "").trim(), 10);
  if (!Number.isFinite(parsed)) return store.settings.sfxVolume;
  return Math.min(100, Math.max(0, parsed));
}

function onSfxVolumeRangeInput(event) {
  setRange("sfxVolume", event);
}

function onSfxVolumeTextInput(event) {
  setUiPref("sfxVolume", normalizeSfxVolume(event.target.value));
}

function onFullscreenToggle() {
  playSelect();
  setSetting("fullscreen", !store.settings.fullscreen);
}

function onBrightnessRangeInput(event) {
  setSetting("brightness", sliderToBright(event.target.value));
}

function onDropdownNumberChange(name, value) {
  setSetting(name, Number(value));
}

function onVolumePercentRangeInput(name, event) {
  setSetting(name, percentToIniVolume(event.target.value));
}

function onVolumePercentNumberInput(name, value) {
  setSetting(name, percentToIniVolume(value));
}

function onHdNumericRangeInput(name, event) {
  setSetting(name, clampInt(event.target.value, 0, 100, 100));
}

function onHdNumericNumberInput(name, value) {
  setSetting(name, clampInt(value, 0, 100, 100));
}

function rangeFillStyle(value, min, max) {
  const numericValue = Number(value);
  const numericMin = Number(min);
  const numericMax = Number(max);
  if (
    !Number.isFinite(numericValue) ||
    !Number.isFinite(numericMin) ||
    !Number.isFinite(numericMax) ||
    numericMax <= numericMin
  ) {
    return { "--settings-range-fill": "0%" };
  }

  const percent = ((numericValue - numericMin) / (numericMax - numericMin)) * 100;
  const clamped = Math.max(0, Math.min(100, percent));
  return { "--settings-range-fill": `${clamped}%` };
}

function findResolutionPreset(width, height) {
  return RESOLUTION_OPTIONS.find((option) => option.width === Number(width) && option.height === Number(height)) ?? null;
}

function nearestResolutionPreset(width, height) {
  const targetWidth = Number(width);
  const targetHeight = Number(height);
  if (!Number.isFinite(targetWidth) || !Number.isFinite(targetHeight)) {
    return RESOLUTION_OPTIONS[0];
  }

  return RESOLUTION_OPTIONS.reduce((best, option) => {
    const bestScore = Math.abs(best.width - targetWidth) + Math.abs(best.height - targetHeight);
    const optionScore = Math.abs(option.width - targetWidth) + Math.abs(option.height - targetHeight);
    return optionScore < bestScore ? option : best;
  });
}

const windowResolutionIsCustom = computed(() =>
  Boolean(store.settings.customWindowResolution) || !findResolutionPreset(store.settings.windowW, store.settings.windowH)
);

const fullscreenResolutionIsCustom = computed(() =>
  Boolean(store.settings.customFullscreenResolution) || !findResolutionPreset(store.settings.fullscreenW, store.settings.fullscreenH)
);

const selectedWindowResolution = computed(() =>
  findResolutionPreset(store.settings.windowW, store.settings.windowH)?.value
    ?? nearestResolutionPreset(store.settings.windowW, store.settings.windowH)?.value
    ?? RESOLUTION_OPTIONS[0].value
);

const selectedFullscreenResolution = computed(() =>
  findResolutionPreset(store.settings.fullscreenW, store.settings.fullscreenH)?.value
    ?? nearestResolutionPreset(store.settings.fullscreenW, store.settings.fullscreenH)?.value
    ?? RESOLUTION_OPTIONS[0].value
);

function setResolutionPreset(target, value) {
  const parsed = parseResolutionValue(value);
  if (!parsed) return;
  if (target === "window") {
    setSetting("windowW", parsed.width);
    setSetting("windowH", parsed.height);
    return;
  }
  setSetting("fullscreenW", parsed.width);
  setSetting("fullscreenH", parsed.height);
}

function setCustomResolutionMode(target, enabled) {
  const key = target === "window" ? "customWindowResolution" : "customFullscreenResolution";
  if (!enabled) {
    const width = target === "window" ? store.settings.windowW : store.settings.fullscreenW;
    const height = target === "window" ? store.settings.windowH : store.settings.fullscreenH;
    const preset = findResolutionPreset(width, height) ?? nearestResolutionPreset(width, height);
    if (preset) {
      setResolutionPreset(target, preset.value);
    }
  }
  setUiPref(key, Boolean(enabled));
}

function onGameCustomResolutionEditButton(target) {
  const enabled =
    target === "window" ? windowResolutionIsCustom.value : fullscreenResolutionIsCustom.value;
  setCustomResolutionMode(target, !enabled);
}

function onMaxCharDisplayRangeInput(event) {
  setSetting("maxCharDisplay", clampInt(event.target.value, 4, 100, 100));
}

function onMaxCharDisplayNumberInput(value) {
  setSetting("maxCharDisplay", clampInt(value, 4, 100, 100));
}

</script>

<template>
  <div class="overflow-auto h-full min-h-0 scrollbar pr-2 flex flex-col gap-3 overflow-x-hidden">
    <template v-if="showLauncher">
      <div key="settings-launcher-section" class="flex flex-col gap-5 text-[25px]">
        <SettingsItem :name="$t('style-label')" info-key="launcher-theme">
          <SettingsDropdown
            :model-value="storeMut.style"
            :options="launcherStyleOptions"
            info-key="launcher-theme"
            @update:model-value="storeMut.style = $event"
          />
        </SettingsItem>
        <SettingsItem :name="$t('font-style-label')" info-key="font-selection">
          <SettingsDropdown
            :model-value="store.settings.fontPreset"
            :options="fontPresetOptions"
            info-key="font-selection"
            @change="setUiPref('fontPreset', $event)"
          />
        </SettingsItem>
        <SettingsItem
          v-if="showLauncherResolutionSetting"
          :name="$t('launcher-resolution-label', 'Launcher Resolution')"
          info-key="launcher-resolution"
        >
          <div class="flex items-center gap-3 flex-wrap justify-center">
            <template v-if="launcherResolutionIsCustom">
              <div class="flex gap-1">
                <input
                  v-model="draft.launcherW"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                  @blur="saveLauncherResolution"
                  @keyup.enter="saveLauncherResolution"
                />
                x
                <input
                  v-model="draft.launcherH"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                  @blur="saveLauncherResolution"
                  @keyup.enter="saveLauncherResolution"
                />
              </div>
            </template>
            <SettingsDropdown
              v-else
              :model-value="launcherResolutionSelection"
              :options="launcherRecentResolutionOptions"
              width-class="settings-select-resolution"
              info-key="launcher-resolution"
              @change="onLauncherResolutionChange"
            />
            <button
              type="button"
              class="settings-resolution-custom-button"
              data-settings-info-key="launcher-custom-resolution"
              data-controller-clickable="true"
              data-controller-size="big"
              :data-settings-custom-active="launcherResolutionIsCustom ? 'true' : null"
              :aria-pressed="launcherResolutionIsCustom ? 'true' : 'false'"
              :aria-label="$t('custom-resolution-label', 'Custom Size')"
              :title="$t('custom-resolution-label', 'Custom Size')"
              @click="onLauncherCustomResolutionEditButton"
            >
              <span class="settings-game-folder-edit-icon" aria-hidden="true"></span>
            </button>
            <button
              type="button"
              class="settings-resolution-reset-button"
              data-settings-info-key="launcher-resolution-reset"
              :aria-label="$t('launcher-resolution-reset-label', 'Reset')"
              :title="$t('launcher-resolution-reset-label', 'Reset')"
              @click="resetLauncherResolution"
            >
              <span aria-hidden="true">↺</span>
            </button>
          </div>
        </SettingsItem>
      </div>

      <div class="flex flex-col gap-2 text-[20px]">
        <SettingsCheckbox
          :model-value="store.settings.launcherController"
          @update:model-value="setUiPref('launcherController', $event)"
          :name="$t('launcher-controller-label', 'Controller')"
          info-key="launcher-controller"
        />

        <SettingsCheckbox
          :model-value="store.settings.sfxEnabled"
          @update:model-value="setUiPref('sfxEnabled', $event)"
          :name="$t('launcher-sfx-label', 'Sound Effects')"
          info-key="launcher-sfx"
        />
        <SettingsItem
          v-if="store.settings.sfxEnabled"
          :name="$t('sfx-volume-label', 'UI sound volume')"
          info-key="launcher-sfx-volume"
        >
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="0"
              max="100"
              :value="store.settings.sfxVolume"
              class="range range-primary settings-range-accent w-[200px]"
              :style="rangeFillStyle(store.settings.sfxVolume, 0, 100)"
              @input="onSfxVolumeRangeInput"
            />
            <div class="flex items-center gap-1">
              <input
                type="number"
                min="0"
                max="100"
                :value="store.settings.sfxVolume"
                class="input input-sm input-primary settings-volume-input w-[72px] text-[1rem] text-right"
                data-controller-skip="true"
                @input="onSfxVolumeTextInput"
              />
              <span>%</span>
            </div>
          </div>
        </SettingsItem>
      </div>
    </template>

    <div v-if="showLauncher && (showVersion || showSettings || showGraphics || showAudio || showControls || showAdvanced)" class="divider my-0 py-0"></div>

    <template v-if="showVersion">
      <div key="settings-version-section" class="flex flex-col gap-2 text-[20px]">
        <SettingsItem
          :name="$t('game-branch-label', 'Branch')"
          info-key="game-branch"
          :locked="sessionSettingsLocked"
          :lock-tooltip="settingsLockTooltip"
        >
          <div class="flex flex-col gap-2 items-center w-full">
            <div class="flex flex-wrap gap-2 justify-center" data-settings-branch-group="true">
              <button
                v-for="branch in GAME_BRANCHES"
                :key="branch"
                class="settings-game-branch-button px-3 py-1 rounded border border-[#ffd67c] transition text-[18px] leading-tight"
                :class="[
                  selectedBranch === branch ? 'bg-[#ffd67c] text-black shadow-[0_0_12px_rgba(255,214,124,0.45)]' : 'bg-black/45 text-white hover:text-white opacity-80 hover:opacity-100',
                  sessionSettingsLocked ? 'settings-control-locked' : ''
                ]"
                :data-settings-info-key="`game-branch-${String(branch).toLowerCase()}`"
                :data-controller-node="sessionSettingsLocked ? null : branchNode(branch)"
                data-controller-up="__block__"
                :data-controller-left="previousBranchNode(branch)"
                :data-controller-right="nextBranchNode(branch)"
                :data-controller-down="selectedGameVersionNodeId"
                :disabled="sessionSettingsLocked"
                @click.prevent="onGameBranchClick(branch)"
                @mouseenter="sessionSettingsLocked ? null : playHover()"
              >
                <span>{{ $t(`branch-${branch.toLowerCase()}-label`, branch) }}</span>
              </button>
            </div>
          </div>
        </SettingsItem>

        <SettingsItem
          :name="$t('game-version-label', 'Version')"
          info-key="game-version"
          :locked="sessionSettingsLocked"
          :lock-tooltip="settingsLockTooltip"
        >
          <div class="flex flex-col gap-2 items-center w-full">
            <div class="flex flex-wrap gap-2 justify-center" data-settings-version-group="true">
                <button
                  v-for="version in visibleGameVersions"
                  :key="version"
                  class="settings-game-version-button px-3 py-1 rounded border border-[#ffd67c] transition text-[18px] leading-tight"
                  :class="[
                    store.settings.gameVersion === version ? 'bg-[#ffd67c] text-black shadow-[0_0_12px_rgba(255,214,124,0.45)]' : 'bg-black/45 text-[#444444] hover:text-black opacity-80 hover:opacity-100',
                    sessionSettingsLocked ? 'settings-control-locked' : ''
                  ]"
                  :data-settings-info-key="versionInfoKey(version)"
                  :data-controller-node="sessionSettingsLocked ? null : versionNode(version)"
                  :data-controller-left="previousGameVersionNode(version)"
                  :data-controller-right="nextGameVersionNode(version)"
                  :data-controller-up="selectedBranchNodeId"
                  data-controller-down="settings-friend-signature"
                  :disabled="sessionSettingsLocked"
                  @click.prevent="onGameVersionClick(version)"
                  @mouseenter="sessionSettingsLocked ? null : playHover()"
                >
                  {{ versionDisplayLabel(version) }}
                </button>
            </div>
          </div>
        </SettingsItem>
          <SettingsItem :name="$t('friend-signature-label', 'Signature')" info-key="friend-signature">
            <SettingsDropdown
              :model-value="store.settings.friendSignature"
              :options="friendSignatureOptions"
              info-key="friend-signature"
              controller-node="settings-friend-signature"
              :controller-up="selectedGameVersionNodeId"
              :controller-down="supportsHdVersion ? 'settings-hd-version' : 'settings-game-folder-browse'"
              @change="onFriendSignatureChange"
            />
          </SettingsItem>

        <SettingsCheckbox
          v-if="supportsHdVersion"
          :model-value="store.settings.hdVersion"
          @update:model-value="onHdVersionChange($event)"
          :name="$t('hd-version-label')"
          info-key="hd-version"
          controller-node="settings-hd-version"
          controller-up="settings-friend-signature"
          controller-down="settings-game-folder-browse"
        />

        <SettingsItem
          :name="$t('game-folder-label', 'Location')"
          info-key="game-folder"
          :locked="sessionSettingsLocked"
          :lock-tooltip="settingsLockTooltip"
        >
          <div class="settings-game-folder-action-row">
            <button
              type="button"
              class="settings-game-folder-browse-button"
              :class="{ 'settings-control-locked': sessionSettingsLocked }"
              data-settings-info-key="game-folder-browse"
              :data-controller-clickable="sessionSettingsLocked ? null : 'true'"
              data-controller-size="big"
              :data-controller-node="sessionSettingsLocked ? null : 'settings-game-folder-browse'"
              :data-controller-up="gameFolderControllerUpNode"
              data-controller-right="settings-game-folder-edit"
              :disabled="sessionSettingsLocked"
              @click="onChooseFolder"
            >
              {{ $t("browse-folder-label", "Browse Folder") }}
            </button>
          </div>
          <template #extended>
            <div class="settings-game-folder-path-row">
              <div class="settings-game-folder-path-shell" :class="{ 'settings-game-folder-path-shell-editing': isEditingGameFolder }">
                <input
                  v-if="isEditingGameFolder"
                  ref="gameFolderInputRef"
                  v-model="gameFolderDraft"
                  type="text"
                  class="settings-game-folder-input"
                  :placeholder="gameFolderPlaceholder"
                  data-controller-skip="true"
                  :disabled="sessionSettingsLocked"
                  @keyup.enter="commitGameFolderEdit"
                  @keyup.esc="cancelGameFolderEdit"
                />
                <div v-else class="settings-game-folder-path">
                  {{ gameFolderDisplayValue }}
                </div>
                <button
                  type="button"
                  class="settings-game-folder-inline-action-button"
                  :class="{ 'settings-control-locked': sessionSettingsLocked }"
                  data-settings-info-key="game-folder-edit"
                  :data-controller-clickable="sessionSettingsLocked ? null : 'true'"
                  data-controller-size="big"
                  :data-controller-node="sessionSettingsLocked ? null : 'settings-game-folder-edit'"
                  :data-controller-up="gameFolderControllerUpNode"
                  data-controller-left="settings-game-folder-browse"
                  :disabled="sessionSettingsLocked"
                  :aria-label="
                    isEditingGameFolder
                      ? $t('apply-folder-label', 'Apply')
                      : $t('edit-folder-label', 'Edit Path')
                  "
                  :title="
                    isEditingGameFolder
                      ? $t('apply-folder-label', 'Apply')
                      : $t('edit-folder-label', 'Edit Path')
                  "
                  @click="onGameFolderEditButton"
                >
                  <span
                    class="settings-game-folder-edit-icon"
                    :class="{ 'settings-game-folder-edit-icon-apply': isEditingGameFolder }"
                    aria-hidden="true"
                  ></span>
                </button>
              </div>
            </div>
            <div class="settings-game-folder-description">
              {{
                $t(
                  "game-folder-panel-description",
                  "Defaults to launcher directory unless set by the Hunter."
                )
              }}
            </div>
          </template>
        </SettingsItem>
      </div>
    </template>

    <div v-if="showVersion && (showSettings || showGraphics || showAudio || showControls || showAdvanced)" class="divider my-0 py-0"></div>

    <template v-if="showSettings">
      <div key="settings-screen-section" class="flex flex-col gap-2 text-[20px] pb-3">
        <div class="settings-brightness-setting" data-settings-info-key="brightness">
          <div class="settings-brightness-preview" aria-hidden="true">
            <img
              :src="assetUrl('/extra/Brightness.png')"
              class="settings-brightness-preview-image"
              :style="brightnessPreviewImageStyle"
              draggable="false"
              alt=""
            />
          </div>
          <h2 class="settings-brightness-label">
            {{ $t('brightness-label', 'Brightness') }}
          </h2>
          <input
            type="range"
            min="-35"
            max="35"
            step="1"
            :value="brightnessSliderValue"
            class="range range-primary settings-range-accent settings-brightness-range"
            :style="rangeFillStyle(brightnessSliderValue, -35, 35)"
            @input="onBrightnessRangeInput"
          />
          <span class="settings-brightness-value">{{ formatSignedNumber(brightnessSliderValue) }}</span>
        </div>

        <div
          class="settings-binary-toggle flex flex-wrap items-center justify-center gap-3 min-h-[45px] text-center cursor-pointer"
          data-settings-info-key="display-mode"
          data-controller-clickable="true"
          data-controller-size="big"
          :data-controller-toggle-state="store.settings.fullscreen ? 'on' : 'off'"
          tabindex="0"
          @click.stop.prevent="onFullscreenToggle"
        >
          <span
            class="transition-opacity"
            :class="store.settings.fullscreen ? 'opacity-45 text-white/70' : 'text-[var(--controller-active-color)] opacity-100'"
          >
            {{ $t('windowed-label', 'Windowed') }}
          </span>
          <label class="relative inline-flex items-center cursor-pointer" @click.stop.prevent="onFullscreenToggle">
            <input
              type="checkbox"
              class="sr-only peer"
              :checked="store.settings.fullscreen"
              @change.stop
            />
            <div class="w-12 h-7 rounded-full bg-black/50 border transition-colors" :style="{ borderColor: 'var(--controller-active-color)' }"></div>
            <div class="absolute left-[3px] top-[3px] w-5 h-5 rounded-full bg-[#f5f5f5] shadow transition-transform transition-colors peer-checked:translate-x-5" :style="store.settings.fullscreen ? { backgroundColor: 'var(--controller-active-color)' } : null"></div>
          </label>
          <span
            class="transition-opacity"
            :class="store.settings.fullscreen ? 'text-[var(--controller-active-color)] opacity-100' : 'opacity-45 text-white/70'"
          >
            {{ $t('fullscreen-label', 'Fullscreen') }}
          </span>
        </div>

        <SettingsItem v-if="!store.settings.fullscreen" :name="$t('window-resolution-label')" info-key="window-resolution">
          <div class="flex items-center gap-3">
            <template v-if="!windowResolutionIsCustom">
              <SettingsDropdown
                :model-value="selectedWindowResolution"
                :options="RESOLUTION_OPTIONS"
                width-class="settings-select-resolution"
                @change="setResolutionPreset('window', $event)"
              />
            </template>
            <template v-else>
              <div class="flex gap-1">
                <input
                  :value="draft.windowW"
                  @input="draft.windowW = $event.target.value"
                  @blur="commitPositiveDraft('windowW')"
                  @keyup.enter="commitPositiveDraft('windowW')"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                />
                x
                <input
                  :value="draft.windowH"
                  @input="draft.windowH = $event.target.value"
                  @blur="commitPositiveDraft('windowH')"
                  @keyup.enter="commitPositiveDraft('windowH')"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                />
              </div>
            </template>
            <button
              type="button"
              class="settings-resolution-custom-button"
              data-settings-info-key="window-custom-resolution"
              data-controller-clickable="true"
              data-controller-size="big"
              :data-settings-custom-active="windowResolutionIsCustom ? 'true' : null"
              :aria-pressed="windowResolutionIsCustom ? 'true' : 'false'"
              :aria-label="$t('custom-resolution-label', 'Custom Size')"
              :title="$t('custom-resolution-label', 'Custom Size')"
              @click="onGameCustomResolutionEditButton('window')"
            >
              <span class="settings-game-folder-edit-icon" aria-hidden="true"></span>
            </button>
          </div>
        </SettingsItem>

        <SettingsItem v-else :name="$t('fullscreen-resolution-label')" info-key="fullscreen-resolution">
          <div class="flex items-center gap-3" :class="store.settings.matchMonitorResolution ? 'opacity-50' : ''">
            <template v-if="!fullscreenResolutionIsCustom">
              <SettingsDropdown
                :model-value="selectedFullscreenResolution"
                :options="RESOLUTION_OPTIONS"
                width-class="settings-select-resolution"
                :disabled="store.settings.matchMonitorResolution"
                @change="setResolutionPreset('fullscreen', $event)"
              />
            </template>
            <template v-else>
              <div class="flex gap-1">
                <input
                  :value="draft.fullscreenW"
                  @input="draft.fullscreenW = $event.target.value"
                  @blur="commitPositiveDraft('fullscreenW')"
                  @keyup.enter="commitPositiveDraft('fullscreenW')"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                  :disabled="store.settings.matchMonitorResolution"
                />
                x
                <input
                  :value="draft.fullscreenH"
                  @input="draft.fullscreenH = $event.target.value"
                  @blur="commitPositiveDraft('fullscreenH')"
                  @keyup.enter="commitPositiveDraft('fullscreenH')"
                  inputmode="numeric"
                  pattern="[0-9]*"
                  class="input input-sm input-primary w-[90px] text-[20px]"
                  :disabled="store.settings.matchMonitorResolution"
                />
              </div>
            </template>
            <button
              type="button"
              class="settings-resolution-custom-button"
              data-settings-info-key="fullscreen-custom-resolution"
              data-controller-clickable="true"
              data-controller-size="big"
              :data-settings-custom-active="fullscreenResolutionIsCustom ? 'true' : null"
              :aria-pressed="fullscreenResolutionIsCustom ? 'true' : 'false'"
              :aria-label="$t('custom-resolution-label', 'Custom Size')"
              :title="$t('custom-resolution-label', 'Custom Size')"
              :disabled="store.settings.matchMonitorResolution"
              @click="onGameCustomResolutionEditButton('fullscreen')"
            >
              <span class="settings-game-folder-edit-icon" aria-hidden="true"></span>
            </button>
          </div>
        </SettingsItem>

        <SettingsCheckbox
          v-if="store.settings.fullscreen"
          :model-value="store.settings.matchMonitorResolution"
          @update:model-value="setSetting('matchMonitorResolution', $event)"
          :name="$t('match-monitor-resolution-label', 'Match monitor resolution')"
          info-key="match-monitor-resolution"
        />
      </div>
    </template>

    <div v-if="showSettings && (showGraphics || showAudio || showControls || showAdvanced)" class="divider my-0 py-0"></div>


    <div v-if="showGraphics" key="settings-graphics-section" class="flex flex-col gap-2 text-[20px] pb-3">
        <SettingsCheckbox
          :model-value="store.settings.textureCompression"
          @update:model-value="setSetting('textureCompression', $event)"
          :name="$t('texture-compression-label', 'Texture compression')"
          info-key="texture-compression"
        />

        <SettingsItem :name="$t('max-char-display-label', 'Max character display')" info-key="max-char-display">
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="4"
              max="100"
              step="1"
              :value="store.settings.maxCharDisplay"
              class="range range-primary settings-range-accent w-[220px]"
              :style="rangeFillStyle(store.settings.maxCharDisplay, 4, 100)"
              @input="onMaxCharDisplayRangeInput"
            />
            <input
              type="number"
              min="4"
              max="100"
              step="1"
              :value="store.settings.maxCharDisplay"
              class="input input-sm input-primary w-[90px] text-[18px] text-right"
              data-controller-skip="true"
              @input="onMaxCharDisplayNumberInput($event.target.value)"
            />
          </div>
        </SettingsItem>

        <template v-if="supportsHdGraphics">
          <h2
            class="text-2xl text-center"
            :class="hdGraphicsLocked ? 'text-white/50' : 'text-[#ffd67c]'"
          >
            {{ hdGraphicsLocked ? $t('hd-version-settings-disabled-label', 'HD Mode Settings (Disabled)') : $t('hd-version-settings-label', 'HD Mode Settings') }}
          </h2>

          <div :class="hdGraphicsLocked ? 'opacity-50 pointer-events-none' : ''">
            <SettingsCheckbox
              :model-value="showHdAdvancedSettings"
              @update:model-value="showHdAdvancedSettings = $event"
              :name="$t('advance-settings-label', 'Advance Settings')"
              info-key="hd-advanced-toggle"
            />

            <SettingsCheckbox
              v-for="setting in hdToggleSettings"
              :key="setting.key"
              :model-value="store.settings[setting.key]"
              @update:model-value="setSetting(setting.key, $event)"
              :name="$t(setting.labelKey, setting.fallback)"
              info-key="hd-graphics-toggle"
            />

            <template v-if="showHdAdvancedSettings">
              <SettingsItem
                v-for="setting in hdNumericSettings"
                :key="setting.key"
                :name="$t(setting.labelKey, setting.fallback)"
                info-key="hd-graphics-numeric"
              >
                <div class="flex items-center gap-3">
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    :value="store.settings[setting.key]"
                    class="range range-primary settings-range-accent w-[220px]"
                    :style="rangeFillStyle(store.settings[setting.key], 0, 100)"
                    @input="onHdNumericRangeInput(setting.key, $event)"
                  />
                  <input
                    type="number"
                    min="0"
                    max="100"
                    step="1"
                    :value="store.settings[setting.key]"
                    class="input input-sm input-primary w-[90px] text-[18px] text-right"
                    data-controller-skip="true"
                    @input="onHdNumericNumberInput(setting.key, $event.target.value)"
                  />
                </div>
              </SettingsItem>
            </template>
          </div>
        </template>
    </div>

    <div v-if="showGraphics && (showAudio || showControls || showAdvanced)" class="divider my-0 py-0"></div>

    <div v-if="showAudio" key="settings-audio-section" class="flex flex-col gap-2 text-[20px] pb-3">
        <SettingsCheckbox
          :model-value="store.settings.disableSoundOutput"
          @update:model-value="setSetting('disableSoundOutput', $event)"
          :name="$t('disable-sound-output-label', 'Disable sound output')"
          info-key="disable-sound-output"
        />

        <SettingsItem :name="$t('general-volume-label', 'General volume')" info-key="general-volume">
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="0"
              max="100"
              step="1"
              :value="iniVolumeToPercent(store.settings.sound)"
              class="range range-primary settings-range-accent w-[220px]"
              :style="rangeFillStyle(iniVolumeToPercent(store.settings.sound), 0, 100)"
              @input="onVolumePercentRangeInput('sound', $event)"
            />
            <div class="flex items-center gap-1">
              <input
                type="number"
                min="0"
                max="100"
                step="1"
                :value="iniVolumeToPercent(store.settings.sound)"
                class="input input-sm input-primary w-[72px] text-[18px] text-right"
                data-controller-skip="true"
                @input="onVolumePercentNumberInput('sound', $event.target.value)"
              />
              <span>%</span>
            </div>
          </div>
        </SettingsItem>

        <SettingsItem :name="$t('unfocused-volume-label', 'Unfocused volume')" info-key="unfocused-volume">
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="0"
              max="100"
              step="1"
              :value="iniVolumeToPercent(store.settings.soundUnfocused)"
              class="range range-primary settings-range-accent w-[220px]"
              :style="rangeFillStyle(iniVolumeToPercent(store.settings.soundUnfocused), 0, 100)"
              @input="onVolumePercentRangeInput('soundUnfocused', $event)"
            />
            <div class="flex items-center gap-1">
              <input
                type="number"
                min="0"
                max="100"
                step="1"
                :value="iniVolumeToPercent(store.settings.soundUnfocused)"
                class="input input-sm input-primary w-[72px] text-[18px] text-right"
                data-controller-skip="true"
                @input="onVolumePercentNumberInput('soundUnfocused', $event.target.value)"
              />
              <span>%</span>
            </div>
          </div>
        </SettingsItem>

        <SettingsItem :name="$t('minimized-volume-label', 'Minimized volume')" info-key="minimized-volume">
          <div class="flex items-center gap-3">
            <input
              type="range"
              min="0"
              max="100"
              step="1"
              :value="iniVolumeToPercent(store.settings.soundMinimized)"
              class="range range-primary settings-range-accent w-[220px]"
              :style="rangeFillStyle(iniVolumeToPercent(store.settings.soundMinimized), 0, 100)"
              @input="onVolumePercentRangeInput('soundMinimized', $event)"
            />
            <div class="flex items-center gap-1">
              <input
                type="number"
                min="0"
                max="100"
                step="1"
                :value="iniVolumeToPercent(store.settings.soundMinimized)"
                class="input input-sm input-primary w-[72px] text-[18px] text-right"
                data-controller-skip="true"
                @input="onVolumePercentNumberInput('soundMinimized', $event.target.value)"
              />
              <span>%</span>
            </div>
          </div>
        </SettingsItem>

        <SettingsItem :name="$t('game-bgm-volume-label', 'BGM volume')" info-key="game-bgm-volume">
          <SettingsDropdown
            :model-value="store.settings.gameBgmVolume"
            :options="soundLevelDropdownOptions"
            info-key="game-bgm-volume"
            @change="onDropdownNumberChange('gameBgmVolume', $event)"
          />
        </SettingsItem>

        <SettingsItem :name="$t('game-se-volume-label', 'SFX volume')" info-key="game-se-volume">
          <SettingsDropdown
            :model-value="store.settings.gameSeVolume"
            :options="soundLevelDropdownOptions"
            info-key="game-se-volume"
            @change="onDropdownNumberChange('gameSeVolume', $event)"
          />
        </SettingsItem>

        <SettingsItem :name="$t('sound-frequency-label', 'Sample rate')" info-key="sound-frequency">
          <SettingsDropdown
            :model-value="store.settings.soundFrequency"
            :options="sampleRateDropdownOptions"
            info-key="sound-frequency"
            @change="onDropdownNumberChange('soundFrequency', $event)"
          />
        </SettingsItem>

        <SettingsItem :name="$t('sound-buffer-num-label', 'Buffer size')" info-key="sound-buffer-num">
          <div class="flex items-center gap-2">
            <input
              :value="draft.soundBufferNum"
              @input="draft.soundBufferNum = $event.target.value"
              @blur="commitClampedDraft('soundBufferNum', 256, 16384)"
              @keyup.enter="commitClampedDraft('soundBufferNum', 256, 16384)"
              inputmode="numeric"
              pattern="[0-9]*"
              class="input input-sm input-primary w-[110px] text-[20px]"
            />
            <span class="text-[16px] opacity-70">(256-16384)</span>
          </div>
        </SettingsItem>
    </div>

    <div v-if="showAudio && (showControls || showAdvanced)" class="divider my-0 py-0"></div>

    <template v-if="showControls">
      <div key="settings-controls-section" class="flex flex-col gap-2 text-[20px] pb-3">
        <SettingsCheckbox
          :model-value="store.settings.controllerVibration"
          @update:model-value="setSetting('controllerVibration', $event)"
          :name="$t('controller-vibration-label', 'Controller vibration')"
          info-key="controller-vibration"
        />

        <SettingsCheckbox
          :model-value="store.settings.preloadControllerDlls"
          @update:model-value="onControllerDllToggle"
          :name="$t('disable-xinput-preload-label', 'R-Analog Patch')"
          info-key="controller-fix"
          :disabled="controllerDllsMissing"
          :locked="controllerDllsMissing"
          :lock-tooltip="controllerDllsMissingTooltip"
        />
      </div>
    </template>

    <div v-if="showControls && showAdvanced" class="divider my-0 py-0"></div>

    <template v-if="showAdvanced">
      <div key="settings-advanced-section" class="flex flex-col gap-2 text-[20px] pb-3">
        <SettingsItem
          v-if="showWinePrefixSettings"
          :name="$t('wine-prefix-label', 'Wine Prefix')"
          info-key="wine-prefix-mode"
        >
          <div class="flex flex-col items-start gap-3">
            <SettingsDropdown
              :model-value="store.settings.winePrefixMode"
              :options="winePrefixModeOptions"
              info-key="wine-prefix-mode"
              controller-node="settings-wine-prefix-mode"
              @change="onWinePrefixModeChange"
            />
          </div>
          <template #extended>
            <div
              v-if="showPortableWinePrefixStatus"
              class="settings-prefix-status-row"
            >
              <div
                class="settings-prefix-status-label"
                :class="portableWinePrefixStatusClass"
              >
                {{ portableWinePrefixStatusLabel }}
              </div>
              <button
                v-if="store.settings.winePrefixMode === 'portable'"
                type="button"
                class="settings-resolution-action-button settings-prefix-install-button"
                data-controller-clickable="true"
                data-controller-size="big"
                data-controller-node="settings-wine-prefix-install"
                :data-controller-up="winePrefixControllerUpNode"
                :disabled="store.linuxPrefixStatus.loading"
                @click="dialogOpenLinuxPrefixInstall"
              >
                {{ $t("install-button", "Install") }}
              </button>
            </div>
            <div
              v-if="showPortableWinePrefixStatus && store.linuxPrefixStatus.prefixPath"
              class="settings-game-folder-path-row settings-prefix-path-row"
            >
              <div class="settings-game-folder-path-shell settings-prefix-path-shell">
                <div class="settings-game-folder-path settings-prefix-path">
                  {{ store.linuxPrefixStatus.prefixPath }}
                </div>
              </div>
            </div>
          </template>
        </SettingsItem>

        <SettingsItem
          v-if="showCustomWinePrefixPath"
          :name="$t('wine-prefix-custom-path-label', 'Custom Prefix Path')"
          info-key="wine-prefix-custom-path"
        >
          <div class="settings-game-folder-action-row">
            <button
              type="button"
              class="settings-game-folder-browse-button"
              data-settings-info-key="wine-prefix-custom-browse"
              data-controller-clickable="true"
              data-controller-size="big"
              data-controller-node="settings-wine-prefix-browse"
              :data-controller-up="winePrefixControllerUpNode"
              data-controller-right="settings-wine-prefix-edit"
              @click="onChooseWinePrefixFolder"
            >
              {{ $t("wine-prefix-browse-label", "Browse") }}
            </button>
          </div>
          <template #extended>
            <div class="settings-game-folder-path-row">
              <div
                class="settings-game-folder-path-shell"
                :class="{ 'settings-game-folder-path-shell-editing': isEditingWinePrefixCustomPath }"
              >
                <input
                  v-if="isEditingWinePrefixCustomPath"
                  ref="winePrefixCustomInputRef"
                  v-model="winePrefixCustomDraft"
                  type="text"
                  class="settings-game-folder-input"
                  :placeholder="winePrefixCustomPlaceholder"
                  data-controller-skip="true"
                  @keyup.enter="commitWinePrefixCustomEdit"
                  @keyup.esc="cancelWinePrefixCustomEdit"
                />
                <div v-else class="settings-game-folder-path">
                  {{ winePrefixCustomDisplayValue }}
                </div>
                <button
                  type="button"
                  class="settings-game-folder-inline-action-button"
                  data-controller-clickable="true"
                  data-controller-size="big"
                  data-controller-node="settings-wine-prefix-edit"
                  :data-controller-up="winePrefixControllerUpNode"
                  data-controller-left="settings-wine-prefix-browse"
                  :aria-label="
                    isEditingWinePrefixCustomPath
                      ? $t('apply-folder-label', 'Apply')
                      : $t('edit-folder-label', 'Edit Path')
                  "
                  :title="
                    isEditingWinePrefixCustomPath
                      ? $t('apply-folder-label', 'Apply')
                      : $t('edit-folder-label', 'Edit Path')
                  "
                  @click="onWinePrefixCustomEditButton"
                >
                  <span
                    class="settings-game-folder-edit-icon"
                    :class="{ 'settings-game-folder-edit-icon-apply': isEditingWinePrefixCustomPath }"
                    aria-hidden="true"
                  ></span>
                </button>
              </div>
            </div>
            <div
              v-if="winePrefixCustomPathError"
              class="settings-game-folder-description settings-inline-error"
            >
              {{ winePrefixCustomPathError }}
            </div>
          </template>
        </SettingsItem>

        <SettingsCheckbox
          :model-value="store.settings.devMode"
          @update:model-value="setUiPref('devMode', $event)"
          :name="$t('dev-mode-label', 'Dev Mode')"
          info-key="dev-mode"
          :disabled="sessionSettingsLocked"
          :locked="sessionSettingsLocked"
          :lock-tooltip="settingsLockTooltip"
        />

        <SettingsCheckbox
          v-if="showLinuxHardwareAccelerationSetting"
          :model-value="store.settings.linuxHardwareAcceleration"
          @update:model-value="setUiPref('linuxHardwareAcceleration', $event)"
          :name="$t('launcher-hardware-acceleration-label', 'Hardware Acceleration')"
          info-key="launcher-hardware-acceleration"
        />

        <SettingsCheckbox
          :model-value="store.settings.offlineImages"
          @update:model-value="setUiPref('offlineImages', $event)"
          :name="$t('offline-images-label', 'Offline-Images')"
          info-key="offline-images"
        />

        <SettingsButton
          :label="$t('reset-patch-label')"
          :button-text="$t('reset-button-label')"
          :game-folder="storeMut.gameFolder || effectiveFolder"
          info-key="reset-patch"
          :disabled="sessionSettingsLocked"
          :locked="sessionSettingsLocked"
          :lock-tooltip="settingsLockTooltip"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.settings-control-locked {
  cursor: default !important;
  opacity: 1 !important;
  pointer-events: none;
}

.settings-brightness-preview {
  grid-column: 1 / -1;
  width: 100%;
  aspect-ratio: 508 / 33;
  overflow: hidden;
}

.settings-brightness-setting {
  display: inline-grid;
  grid-template-columns: max-content 220px 44px;
  width: fit-content;
  margin: 0 auto;
  align-items: center;
  column-gap: 0.75rem;
  row-gap: 0.35rem;
  min-height: 45px;
}

.settings-brightness-label {
  grid-column: 1;
  grid-row: 2;
  line-height: 1.25;
}

.settings-brightness-range {
  grid-column: 2;
  grid-row: 2;
  width: 220px;
}

.settings-brightness-value {
  grid-column: 3;
  grid-row: 2;
  min-width: 44px;
  text-align: right;
}

.settings-brightness-preview-image {
  display: block;
  width: calc(100% + 4px);
  max-width: none;
  height: auto;
  margin: -2px 0 0 -2px;
}

.settings-game-branch-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 72px;
  min-height: 42px;
  gap: 0;
  color: #444444;
}

.settings-game-branch-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.1rem;
  min-width: 72px;
}

.input.input-sm.input-primary {
  border-radius: 0.25rem;
  border-color: rgba(255, 255, 255, 0.2) !important;
  background: #00000099 !important;
  color: #ffffff !important;
  box-shadow: none !important;
  font-size: 18px !important;
  line-height: 1.25rem !important;
}

.input.input-sm.input-primary::placeholder {
  color: rgba(255, 255, 255, 0.4);
}

.input.input-sm.input-primary:focus,
.input.input-sm.input-primary:focus-visible {
  outline: none !important;
  border-color: var(--controller-focus-color) !important;
  box-shadow: 0 0 0 1px var(--controller-focus-color), var(--controller-focus-glow) !important;
}

.input.input-sm.input-primary:disabled {
  opacity: 0.55;
}

.settings-game-folder-browse-button.controller-nav-focused,
.settings-game-folder-browse-button:focus-visible {
  outline: none !important;
  border-color: var(--controller-focus-color) !important;
  box-shadow: var(--controller-focus-glow) !important;
}

.settings-game-folder-action-row {
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-game-folder-description {
  width: min(100%, 720px);
  margin: 0.45rem auto 0;
  font-size: 15px;
  color: rgba(255, 255, 255, 0.72);
  text-align: left;
}

.settings-game-folder-browse-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  border: 1px solid #ffd67c;
  color: #ffffff;
  background: rgba(0, 0, 0, 0.45);
  font-size: 18px;
  line-height: 1.25rem;
  opacity: 0.8;
  transition: opacity 0.16s ease, background-color 0.16s ease, color 0.16s ease,
    border-color 0.16s ease, box-shadow 0.16s ease;
}

.settings-game-folder-browse-button:hover {
  color: #ffffff;
  background: rgba(0, 0, 0, 0.45);
  border-color: #ffd67c;
  box-shadow: none;
  opacity: 1;
}

.settings-game-folder-edit-icon {
  width: 20px;
  height: 20px;
  border: 0;
  border-radius: 0;
  background: url("/extra/PencilB.png") center / contain no-repeat;
}

.settings-game-folder-edit-icon-apply {
  width: 20px;
  height: 20px;
  border: 0;
  border-radius: 0;
  background: url("/extra/PencilB.png") center / contain no-repeat;
  filter: drop-shadow(0 0 4px rgba(255, 214, 124, 0.45));
}

.settings-game-folder-path-row {
  width: min(100%, 720px);
  margin: 0.6rem auto 0;
}

.settings-game-folder-path-shell {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.25rem;
  padding: 0;
  border-radius: 0.25rem;
  border: 0;
  background: #00000099;
  box-shadow: none;
}

.settings-game-folder-path-shell-editing {
  box-shadow: 0 0 0 1px var(--controller-focus-color), var(--controller-focus-glow);
}

.settings-game-folder-input,
.settings-game-folder-path {
  width: 100%;
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  border: 0;
  background: transparent;
  color: rgba(255, 255, 255, 0.86);
  font-size: 18px;
  line-height: 1.25rem;
  text-align: left;
  word-break: break-all;
  font-family: "Consolas", "Courier New", monospace;
}

.settings-game-folder-input {
  outline: none;
}

.settings-game-folder-input::placeholder {
  color: rgba(255, 255, 255, 0.42);
}

.settings-game-folder-inline-action-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  margin-right: 0;
  padding: 0;
  border: 0;
  border-left: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 0;
  color: #ffffff;
  background: transparent;
  box-shadow: none !important;
  transition: background-color 0.16s ease;
}

.settings-game-folder-inline-action-button:hover {
  background: rgba(255, 214, 124, 0.1);
}

.settings-game-folder-inline-action-button.controller-nav-focused,
.settings-game-folder-inline-action-button:focus-visible {
  outline: none !important;
  background: rgba(255, 214, 124, 0.14);
  border-left-color: var(--controller-focus-color) !important;
  box-shadow: none !important;
}

.settings-resolution-reset-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  border: 1px solid rgba(255, 214, 124, 0.35);
  border-radius: 0.25rem;
  color: #ffd67c;
  background: rgba(0, 0, 0, 0.45);
  transition: background-color 0.16s ease, color 0.16s ease, border-color 0.16s ease;
}

.settings-resolution-custom-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  border: 1px solid rgba(255, 214, 124, 0.35);
  border-radius: 0.25rem;
  color: #ffd67c;
  background: rgba(0, 0, 0, 0.45);
  transition:
    background-color 0.16s ease,
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    opacity 0.16s ease;
}

.settings-resolution-action-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 4.5rem;
  height: 2.25rem;
  padding: 0 0.9rem;
  border-radius: 0.25rem;
  border: 1px solid #ffd67c;
  color: #ffffff;
  background: rgba(0, 0, 0, 0.45);
  font-size: 18px;
  line-height: 1.25rem;
  opacity: 0.85;
  transition: opacity 0.16s ease, background-color 0.16s ease, color 0.16s ease,
    border-color 0.16s ease, box-shadow 0.16s ease;
}

.settings-resolution-action-button:hover,
.settings-resolution-action-button:focus-visible {
  outline: none;
  opacity: 1;
  background: rgba(255, 214, 124, 0.14);
  color: #fff3c7;
}

.settings-resolution-action-button:disabled {
  opacity: 0.45;
}

.settings-resolution-custom-button:hover,
.settings-resolution-custom-button:focus-visible,
.settings-resolution-custom-button[data-settings-custom-active="true"] {
  outline: none;
  background: rgba(255, 214, 124, 0.14);
  border-color: #ffd67c;
}

.settings-resolution-custom-button:disabled {
  opacity: 0.45;
  pointer-events: none;
}

.settings-resolution-reset-button:hover,
.settings-resolution-reset-button:focus-visible {
  outline: none;
  background: rgba(255, 214, 124, 0.14);
  color: #fff3c7;
  border-color: #ffd67c;
}

.settings-prefix-status-row {
  width: min(100%, 720px);
  margin: 0.6rem auto 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.settings-prefix-install-button {
  flex: 0 0 auto;
}

.settings-prefix-status-label {
  flex: 1;
  min-height: 2.25rem;
  display: flex;
  align-items: center;
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  background: rgba(0, 0, 0, 0.45);
  font-size: 17px;
  line-height: 1.2;
  text-align: left;
}

.settings-prefix-status-ready {
  color: #d7f7bf;
}

.settings-prefix-status-warning {
  color: #ffd7a0;
}

.settings-inline-error {
  margin-top: 0.35rem;
  color: #ffb3b3;
}
</style>

