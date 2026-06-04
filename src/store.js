import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { readDir } from "@tauri-apps/api/fs";
import { join } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { computed, nextTick, reactive, readonly, ref, watch, watchEffect } from "vue";

import { getMessage, setFluentLocale } from "./fluent";
import {
  LOGIN_PAGE,
  CLASSIC_STYLE,
  PS4_STYLE,
  CHARACTERS_PAGE,
  SERVERS_DIALOG,
  DELETE_DIALOG,
  SETTINGS_PAGE,
  PATCHER_PAGE,
  PATCHER_DIALOG,
  SERVER_SWITCH_DIALOG,
  VERSION_SWITCH_DIALOG,
  EXTERNAL_LINK_DIALOG,
  RESET_PATCH_DIALOG,
  BAN_DIALOG,
  LINUX_PREFIX_DIALOG,
  CHECKING_PATCHER,
  DONE_PATCHER,
  ERROR_PATCHER,
  DOWNLOADING_PATCHER,
  RESTORING_PATCHER,
  PATCHING_PATCHER,
  GAME_VERSIONS,
  getLauncherResolutionPrefKeys,
  getLauncherRecentResolutionPrefKey,
  getLauncherCustomResolutionPrefKey,
  getLauncherWindowDefaults,
  formatResolutionValue,
  parseResolutionValue,
} from "./common";

import {
  applyCachedImage,
  getCachedImageUrl,
  refreshCachedImageUrl,
  clearImageCache,
} from "./imageCache";
import {
  releaseUiResources,
  resumeUiWindow,
  setLauncherPreference,
  setUiPreference,
  suspendUiWindow,
} from "./bridge";

const DEFAULT_LAUNCHER_TAG = "LilButter™";

function createOfflineImageOverrideSet() {
  return {
    background: null,
    capcom: null,
    cog: null,
    dialogue: null,
    header: null,
    serverPatch: null,
  };
}

const storePrivate = reactive({
  endpoints: [],
  remoteEndpoints: [],
  currentEndpoint: null,
  currentFolder: "",
  lastCharId: null,

  banners: [],
  links: [],
  launcherTag: DEFAULT_LAUNCHER_TAG,
  characters: [],
  friends: [],
  messages: [],
  remoteMessages: [],
  background: null,
  cog: null,
  capcom: null,
  classicButton: null,
  classicAddServerButton: null,
  dialogImage: null,
  serverPatchImage: null,
  classicHeaderOnline: null,
  classicHeaderForward: null,
  classicHeaderG: null,
  classicHeaderZ: null,
  classicHeaderZZ: null,
  ps4Background: null,
    ps4Button: null,
    ps4AddServerButton: null,
    ps4Capcom: null,
  ps4Cog: null,
  ps4Emblem: null,
  ps4HeaderOnline: null,
  ps4HeaderForward: null,
  ps4HeaderG: null,
  ps4HeaderZ: null,
  ps4HeaderZZ: null,
  offlineImageOverrides: {
    classic: createOfflineImageOverrideSet(),
    ps4: createOfflineImageOverrideSet(),
  },

  authLoading: false,
  characterLoading: false,
  gameLaunching: false,
  gameLaunchMessage: "",

  log: [],

  dialogOpen: false,
  dialogKind: 0,
  dialogLoading: false,
  dialogError: "",
  dialogMessage: "",
  versionSignatureChoices: [],
  resetPatchCompleted: false,
  resetPatchProgress: 0,
  linuxPrefixInstallCompleted: false,
  linuxPrefixInstallProgress: 0,
  linuxPrefixStatus: {
    loading: false,
    ready: false,
    missingTools: [],
    prefixPath: "",
    error: "",
    audioReady: true,
    audioMissing: [],
  },

  editEndpointNew: false,
  deleteCharacter: null,

  patcher: {
    total: 0,
    current: 0,
    state: DONE_PATCHER,
    queuePosition: 0,
    queueNoticePosition: 0,
  },

  settings: {
    sfxEnabled: false,
    sfxVolume: 30,
    fontPreset: "default",
    hdVersion: false,
    fullscreen: true,
    fullscreenW: 1920,
    fullscreenH: 1080,
    windowW: 1280,
    windowH: 720,
    customWindowResolution: false,
    customFullscreenResolution: false,
    brightness: -128,
    textureCompression: false,
    maxCharDisplay: 100,
    matchMonitorResolution: true,
    disableSoundOutput: false,
    sound: 0,
    soundUnfocused: 0,
    soundMinimized: 0,
    soundFrequency: 48000,
    soundBufferNum: 2048,
    gameBgmVolume: 5,
    gameSeVolume: 7,
    controllerVibration: false,
    hdGraphicShadowQuest: true,
    hdGraphicShadowLobby: false,
    hdGraphicDof: true,
    hdGraphicBloom: true,
    hdGraphicSsao: true,
    hdGraphicGodray: true,
    hdGraphicAntiAliasing: true,
    hdGraphicSoftParticle: true,
    hdGraphicDofFarBlurSize: 100,
    hdGraphicBloomDispersion: 100,
    hdGraphicBloomThreshold: 100,
    hdGraphicBloomColor: 100,
    hdGraphicGaussianBlurDispersion: 100,
    hdGraphicGaussianBlurBlendRate: 100,
    hdGraphicSsaoDensity: 100,
    hdGraphicShadowmapColor: 100,
    hdGraphicPlLightShadowAttenuation: 100,
    hdGraphicBgLightShadowAttenuation: 100,
    hdGraphicAntiAliasingWeightScale: 100,
    preloadControllerDlls: false,
    friendSignature: "none",
    winePrefixMode: "portable",
    winePrefixCustomPath: null,
    classicLauncherWidth: 1124,
    classicLauncherHeight: 600,
    classicLauncherCustomResolution: false,
    classicLauncherRecentResolutions: [],
    ps4LauncherWidth: 1280,
    ps4LauncherHeight: 720,
    ps4LauncherCustomResolution: false,
    ps4LauncherRecentResolutions: [],
    gameVersion: "ZZ",
      forceGameVersion: false,
      devMode: false,
      externalLinkPromptDisabled: false,
      serverVersionPromptDisabled: false,
      launcherController: true,
      linuxHardwareAcceleration: true,
      offlineImages: false,
    },
  launcherSuspended: false,
  launcherAssetsLoading: false,
  serverVersionInfo: null,
  unitCardLoadCycle: 0,
  characterRevealCycle: 0,
  skipCharacterLoadingForPatcher: false,
});
export const store = readonly(storePrivate);
export const launcherAssetsLoading = computed(() => storePrivate.launcherAssetsLoading);

export function setDevFriendTestEntries(entries) {
  if (!import.meta.env.DEV) return;
  storePrivate.friends = Array.isArray(entries)
    ? entries
        .map((entry) => ({
          cid: Number(entry?.cid ?? 0),
          id: Number(entry?.id ?? 0),
          name: String(entry?.name ?? "").trim(),
        }))
        .filter(
          (entry) =>
            Number.isFinite(entry.cid) &&
            entry.cid > 0 &&
            Number.isFinite(entry.id) &&
            entry.id > 0 &&
            entry.name
        )
    : [];
}

const fallbackBackground = "/backgroundALT7.jpg";

function classicLikeAssetFolder() {
  if (storeMut.style === PS4_STYLE) return "/ps4";
  return "/classic";
}

function styleAssetPath(fileName) {
  return `${classicLikeAssetFolder()}/${fileName}`;
}

function fallbackBackgroundPath() {
  return storeMut.style === PS4_STYLE ? "/ps4/BackgroundALT5.png" : fallbackBackground;
}

export function assetUrl(path) {
  return path;
}

export const storeMut = reactive({
  page: LOGIN_PAGE,
  settingsReturnPage: LOGIN_PAGE,
  style: CLASSIC_STYLE,
  locale: "",
  username: "",
  password: "",
  rememberMe: false,
  gameFolder: "",
  editEndpoint: null,
  serverlistUrl: "",
  messagelistUrl: "",
});

function normalizeLauncherStyle(style, _devMode) {
  return style === PS4_STYLE ? PS4_STYLE : CLASSIC_STYLE;
}

function normalizeWinePrefixMode(value) {
  const normalized = String(value ?? "").trim().toLowerCase();
  if (["system", "custom", "proton"].includes(normalized)) return normalized;
  return "portable";
}

function normalizeWinePrefixCustomPath(value) {
  if (value === null || value === undefined) return null;
  const normalized = String(value).trim();
  return normalized || null;
}

function normalizeLauncherDimension(value, fallback) {
  const parsed = Number.parseInt(String(value ?? "").trim(), 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(8192, Math.max(320, parsed));
}

function normalizeLauncherRecentResolutionList(values) {
  if (!Array.isArray(values)) return [];
  const normalized = [];
  for (const value of values) {
    const parsed = parseResolutionValue(value);
    if (!parsed) continue;
    if (normalized.includes(parsed.value)) continue;
    normalized.push(parsed.value);
    if (normalized.length >= 5) break;
  }
  return normalized;
}

function applyLauncherResolutionPrefs(prefs = {}) {
  for (const style of [CLASSIC_STYLE, PS4_STYLE]) {
    const keys = getLauncherResolutionPrefKeys(style);
    const recentKey = getLauncherRecentResolutionPrefKey(style);
    const customKey = getLauncherCustomResolutionPrefKey(style);
    const defaults = getLauncherWindowDefaults(style);
    if (!keys) continue;
    storePrivate.settings[keys.width] = normalizeLauncherDimension(
      prefs?.[keys.width] ?? storePrivate.settings[keys.width] ?? defaults.width,
      defaults.width
    );
    storePrivate.settings[keys.height] = normalizeLauncherDimension(
      prefs?.[keys.height] ?? storePrivate.settings[keys.height] ?? defaults.height,
      defaults.height
    );
    if (recentKey) {
      storePrivate.settings[recentKey] = normalizeLauncherRecentResolutionList(
        prefs?.[recentKey] ?? storePrivate.settings[recentKey] ?? []
      );
    }
    if (customKey) {
      storePrivate.settings[customKey] = Boolean(
        prefs?.[customKey] ?? storePrivate.settings[customKey] ?? false
      );
    }
  }
}

function getLauncherRecentResolutionsForStyle(style) {
  const recentKey = getLauncherRecentResolutionPrefKey(style);
  if (!recentKey) return [];
  return normalizeLauncherRecentResolutionList(storePrivate.settings[recentKey]);
}

function promoteLauncherRecentResolution(style, width, height) {
  const recentKey = getLauncherRecentResolutionPrefKey(style);
  if (!recentKey) return [];
  const nextValue = formatResolutionValue(width, height);
  const existing = getLauncherRecentResolutionsForStyle(style).filter(
    (entry) => entry !== nextValue
  );
  const nextList = [nextValue, ...existing].slice(0, 5);
  storePrivate.settings[recentKey] = nextList;
  return nextList;
}

export function logText(level, text) {
  storePrivate.log.push({ level, message: text });
}

export function logMessage(level, message, args) {
  storePrivate.log.push({
    level,
    message: getMessage(message, args),
  });
}

function isBlankConfiguredPort(value) {
  return (
    value === null ||
    value === undefined ||
    (typeof value === "string" && value.trim() === "")
  );
}

function normalizeConfiguredPort(value) {
  if (isBlankConfiguredPort(value)) return null;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return null;
  const port = Math.trunc(parsed);
  if (port <= 0 || port > 65535) return null;
  return port;
}

const DEFAULT_LAUNCHER_PORT = 8080;
const DEFAULT_GAME_PORT = 53310;

function normalizeDialogPort(value, fallback) {
  if (isBlankConfiguredPort(value)) return fallback;
  return normalizeConfiguredPort(value);
}

function defaultPortFromProtocol(protocol) {
  if (protocol === "https:") return 443;
  if (protocol === "http:") return 80;
  return null;
}

function resolveEndpointPort(endpoint, parsedUrl = null) {
  const configuredPort = normalizeConfiguredPort(endpoint?.launcherPort);
  if (configuredPort !== null) return configuredPort;
  if (!parsedUrl) return null;

  const parsedPort = normalizeConfiguredPort(parsedUrl.port);
  if (parsedPort !== null) return parsedPort;

  return defaultPortFromProtocol(parsedUrl.protocol);
}

function getEndpointServerKey(endpoint) {
  if (!endpoint || !endpoint.url || endpoint.url === "OFFLINEMODE") {
    return "";
  }
  const rawUrl = endpoint.url.includes("://")
    ? endpoint.url
    : `http://${endpoint.url}`;
  try {
    const parsed = new URL(rawUrl);
    const host = parsed.hostname || endpoint.url;
    const port = resolveEndpointPort(endpoint, parsed);
    return port === null ? host : `${host}:${port}`;
  } catch (_error) {
    const port = resolveEndpointPort(endpoint, null);
    return port === null ? endpoint.url : `${endpoint.url}:${port}`;
  }
}

function getServerLabel(serverKey) {
  if (!serverKey) return serverKey;
  const allEndpoints = [...(storePrivate.endpoints ?? []), ...(storePrivate.remoteEndpoints ?? [])];
  const matching = allEndpoints.find(
    (endpoint) => getEndpointServerKey(endpoint) === serverKey
  );
  return matching?.name ?? serverKey;
}

function endpointKey(endpoint) {
  if (!endpoint) return "";
  return [
    endpoint.name ?? "",
    endpoint.url ?? "",
    endpoint.launcherPort ?? "",
    endpoint.gamePort ?? "",
    endpoint.isRemote ? "1" : "0",
    endpoint.version ?? "",
  ].join("|");
}

export const recentLog = ref(null);
let lastPatcherQueueNotice = 0;
let recentLogTimeout = null;
let characterLaunchInputLocked = false;
watchEffect(() => {
  const lastLog = storePrivate.log[storePrivate.log.length - 1];
  if (!lastLog) return;
  clearTimeout(recentLogTimeout);
  recentLogTimeout = setTimeout(() => (recentLog.value = null), 5000);
  recentLog.value = lastLog;
});
export function dismissRecentLog() {
  recentLog.value = null;
}

export const bannerIndex = ref(0);
export const currentBanner = computed(() => {
  const banners = effectiveBanners.value;
  if (!banners.length) return null;
  return banners[bannerIndex.value] ?? banners[0] ?? null;
});

function resolveLocalAsset(url) {
  if (typeof url !== "string") return url;
  if (!url.startsWith("/")) return url;
  return assetUrl(url);
}

const bundledAssetPrefixes = [
  "/classic/",
  "/ps4/",
  "/extra/",
  "/headers/",
  "/banners/",
  "/flags/",
  "/background",
  "/weapons/",
  "/units/",
];

function isBundledAssetPath(path) {
  return bundledAssetPrefixes.some((prefix) => path.startsWith(prefix));
}

function normalizeEndpointBase(endpoint) {
  if (!endpoint?.url || endpoint.url === "OFFLINEMODE") return "";
  const rawUrl = endpoint.url.includes("://")
    ? endpoint.url
    : `http://${endpoint.url}`;
  try {
    const parsed = new URL(rawUrl);
    const resolvedPort = resolveEndpointPort(endpoint, parsed);
    if (resolvedPort !== null) {
      parsed.port = String(resolvedPort);
    }
    return parsed.origin;
  } catch (_error) {
    return rawUrl;
  }
}

function resolveEndpointAsset(path, endpoint) {
  if (typeof path !== "string" || !path) return path;
  if (/^(data:|blob:|https?:)/i.test(path)) return path;
  if (path.startsWith("/") && isBundledAssetPath(path)) {
    return resolveLocalAsset(path);
  }
  const base = normalizeEndpointBase(endpoint);
  if (!base) return path;
  if (path.startsWith("/")) return `${base}${path}`;
  return `${base}/${path}`;
}

function applyCachedImageList(
  items,
  key,
  updateList,
  resolver = resolveLocalAsset,
  options = {}
) {
  if (!Array.isArray(items)) return [];
  const prioritizeFirst = Boolean(options.prioritizeFirst);
  const refreshLimit = Number.isFinite(options.refreshLimit)
    ? Math.max(0, Math.trunc(options.refreshLimit))
    : Number.POSITIVE_INFINITY;
  return items.map((item, index) => {
    const url = resolver(item?.[key]);
    const cached = getCachedImageUrl(url) || url;
    if (url && index < refreshLimit) {
      const refresh = () => {
        refreshCachedImageUrl(url).then((nextUrl) => {
          if (!nextUrl || nextUrl === cached) return;
          updateList(index, nextUrl);
        });
      };
      if (prioritizeFirst && index > 0) {
        setTimeout(refresh, 0);
      } else {
        refresh();
      }
    }
    return { ...item, [key]: cached };
  });
}

function preloadImageWithTimeout(url, timeoutMs = 5000) {
  if (!url || typeof Image === "undefined") return Promise.resolve(false);
  return new Promise((resolve) => {
    let settled = false;
    const image = new Image();
    const complete = (ok) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutId);
      resolve(ok);
    };
    const timeoutId = setTimeout(() => complete(false), timeoutMs);
    image.onload = () => complete(true);
    image.onerror = () => complete(false);
    image.src = url;
  });
}

function uniqueImageUrls(urls) {
  return Array.from(new Set((urls ?? []).filter(Boolean)));
}

async function resolveFreshImageUrl(url) {
  if (!url) return null;
  try {
    return (await refreshCachedImageUrl(url)) || getCachedImageUrl(url) || url;
  } catch (_) {
    return getCachedImageUrl(url) || url;
  }
}

async function preloadFreshImage(url, timeoutMs = 5000) {
  const freshUrl = await resolveFreshImageUrl(url);
  await preloadImageWithTimeout(freshUrl, timeoutMs);
  return freshUrl;
}

async function preloadFreshImageList(urls, timeoutMs = 5000) {
  await Promise.allSettled(
    uniqueImageUrls(urls).map((url) => preloadFreshImage(url, timeoutMs))
  );
}

async function preloadImageListWithTimeout(urls, timeoutMs = 5000) {
  await Promise.allSettled(
    uniqueImageUrls(urls).map((url) => preloadImageWithTimeout(url, timeoutMs))
  );
}

function styleSettingsAssetUrls() {
  return [
    assetUrl(styleAssetPath("Settings.png")),
    assetUrl(styleAssetPath("SettingsHover.png")),
  ];
}

function updateBanner() {
  let value = bannerIndex.value;
  value++;
  if (value >= effectiveBanners.value.length) {
    value = 0;
  }
  bannerIndex.value = value;
}
let bannerInterval = setInterval(updateBanner, 7000);
export function setBannerIndex(index) {
  const maxIndex = effectiveBanners.value.length - 1;
  const parsedIndex = Number(index);
  const safeIndex = Number.isFinite(parsedIndex) ? parsedIndex : 0;
  if (maxIndex < 0) {
    bannerIndex.value = 0;
  } else {
    bannerIndex.value = Math.min(Math.max(safeIndex, 0), maxIndex);
  }
  clearTimeout(bannerInterval);
  bannerInterval = setInterval(updateBanner, 7000);
}

let prevPage = null;
export function onSettingsButton() {
  if (storeMut.page !== SETTINGS_PAGE) {
    prevPage = storeMut.page ?? LOGIN_PAGE;
    storeMut.settingsReturnPage = prevPage;
    storeMut.page = SETTINGS_PAGE;
  } else {
    storeMut.page = storeMut.settingsReturnPage ?? prevPage ?? LOGIN_PAGE;
  }
}

export function updateRemoteMessages(messages) {
  storePrivate.remoteMessages = messages;
}

export function updatePatcher(patcher) {
  const incomingQueuePosition = Number(patcher?.queuePosition ?? 0);
  const nextState = patcher?.state ?? storePrivate.patcher.state;
  const nextQueueNoticePosition =
    nextState === DONE_PATCHER || nextState === ERROR_PATCHER
      ? 0
      : incomingQueuePosition > 0
      ? incomingQueuePosition
      : Number(storePrivate.patcher.queueNoticePosition ?? 0);

  storePrivate.patcher = {
    ...storePrivate.patcher,
    ...patcher,
    queuePosition: incomingQueuePosition,
    queueNoticePosition: nextQueueNoticePosition,
  };

  if (
    incomingQueuePosition > 0 &&
    incomingQueuePosition !== lastPatcherQueueNotice
  ) {
    logMessage("info", "patcher-queue", {
      position: incomingQueuePosition,
    });
    lastPatcherQueueNotice = incomingQueuePosition;
  }

  if (nextState === DONE_PATCHER || nextState === ERROR_PATCHER) {
    lastPatcherQueueNotice = 0;
  }

  if (nextState === DONE_PATCHER) {
    const isOnPatcherPage = storeMut.page === PATCHER_PAGE;
    if (!isOnPatcherPage) {
      return;
    }
  } else if (nextState === ERROR_PATCHER) {
    cancelPatcher();
  }
}
export const patcherPercentage = computed(() => {
  switch (storePrivate.patcher.state) {
    case CHECKING_PATCHER:
      return 0;
    case DOWNLOADING_PATCHER:
      return storePrivate.patcher.current / (storePrivate.patcher.total || 1);
    case RESTORING_PATCHER:
      return 1;
    case PATCHING_PATCHER:
      return 1;
    case DONE_PATCHER:
      return 1;
    default:
      return 0;
  }
});
export const patcherLog = computed(() => {
  switch (storePrivate.patcher.state) {
    case CHECKING_PATCHER:
      if (
        (storePrivate.patcher.queuePosition ||
          storePrivate.patcher.queueNoticePosition ||
          0) > 0
      ) {
        return getMessage("patcher-queue", {
          position:
            storePrivate.patcher.queuePosition ||
            storePrivate.patcher.queueNoticePosition,
        });
      }
      return getMessage("patcher-checking");
    case DOWNLOADING_PATCHER:
      return getMessage("patcher-percentage", {
        percentage: Math.round(patcherPercentage.value * 100),
      });
    case RESTORING_PATCHER:
      return getMessage("patcher-restoring");
    case PATCHING_PATCHER:
      return getMessage("patcher-installing");
    case DONE_PATCHER:
      return storeMut.page === PATCHER_PAGE ? getMessage("patcher-done") : null;
    default:
      return null;
  }
});
async function handleInvoke(cmd, args, level) {
  try {
    return await invoke(cmd, args);
  } catch (error) {
    if (error !== "") {
      level = level || "error";
      logMessage(level, error);
    }
    throw error;
  }
}

const SETTING_WRITE_DELAY_MS = 150;
let settingWriteTimer = null;
const pendingSettingWrites = new Map();

function scheduleSettingWrite(setting, value) {
  pendingSettingWrites.set(setting, value);
  if (settingWriteTimer) clearTimeout(settingWriteTimer);
  settingWriteTimer = setTimeout(async () => {
    const entries = Array.from(pendingSettingWrites.entries());
    pendingSettingWrites.clear();
    settingWriteTimer = null;
    for (const [pendingSetting, pendingValue] of entries) {
      try {
        await handleInvoke("set_setting", {
          setting: pendingSetting,
          value: pendingValue,
        });
      } catch (_error) {
        // swallow errors to keep UI responsive
      }
    }
  }, SETTING_WRITE_DELAY_MS);
}

let launcherPrefWriteTimer = null;
let isHydratingStore = false;
let pendingLauncherPrefs = {};
let launcherPrefSyncPromise = Promise.resolve();
let uiPrefSyncPromise = Promise.resolve();
let gameLaunchOverlayStageTimer = null;

const GAME_LAUNCH_STAGE_DEFS = Object.freeze([
  {
    key: "game-launch-stage-gathering-supplies",
    fallback: "Gathering supplies",
  },
  {
    key: "game-launch-stage-sharpening-weapon",
    fallback: "Sharpening Weapon...",
  },
  {
    key: "game-launch-stage-cooking-rations",
    fallback: "Cooking rations...",
  },
  {
    key: "game-launch-stage-meow",
    fallback: "Meow Meow Meow",
  },
  {
    key: "game-launch-stage-goocoo",
    fallback: "Checking on Goocoo",
  },
]);
const GAME_LAUNCH_READY_FALLBACK = "Hunter is ready!";

function applyGameLaunchStage(index) {
  const stage = GAME_LAUNCH_STAGE_DEFS[Math.max(0, Math.min(index, GAME_LAUNCH_STAGE_DEFS.length - 1))];
  storePrivate.gameLaunching = true;
  const message = getMessage(stage.key, null);
  storePrivate.gameLaunchMessage = message === stage.key ? stage.fallback : message;
}

function stopGameLaunchOverlay() {
  if (gameLaunchOverlayStageTimer) {
    clearInterval(gameLaunchOverlayStageTimer);
    gameLaunchOverlayStageTimer = null;
  }
  storePrivate.gameLaunching = false;
  storePrivate.gameLaunchMessage = "";
}

async function completeGameLaunchOverlay() {
  if (gameLaunchOverlayStageTimer) {
    clearInterval(gameLaunchOverlayStageTimer);
    gameLaunchOverlayStageTimer = null;
  }
  if (!storePrivate.gameLaunching) return;
  const message = getMessage("game-launch-stage-ready", null);
  storePrivate.gameLaunchMessage =
    message === "game-launch-stage-ready" ? GAME_LAUNCH_READY_FALLBACK : message;
  await new Promise((resolve) => setTimeout(resolve, 420));
  stopGameLaunchOverlay();
}

export async function runCharacterLaunchAction(action, options = {}) {
  if (
    characterLaunchInputLocked ||
    storePrivate.characterLoading ||
    storePrivate.authLoading
  ) {
    return false;
  }

  const showLaunchOverlay = Boolean(options.showLaunchOverlay);
  characterLaunchInputLocked = true;
  try {
    if (showLaunchOverlay) {
      await startGameLaunchOverlay();
      await nextTick();
      if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
        await new Promise((resolve) => window.requestAnimationFrame(resolve));
      }
    }
    await action();
    if (showLaunchOverlay) {
      await completeGameLaunchOverlay();
    }
    return true;
  } finally {
    if (showLaunchOverlay && storePrivate.gameLaunching) {
      stopGameLaunchOverlay();
    }
    characterLaunchInputLocked = false;
  }
}

async function startGameLaunchOverlay() {
  stopGameLaunchOverlay();
  let stageIndex = 0;
  applyGameLaunchStage(stageIndex);
  gameLaunchOverlayStageTimer = setInterval(() => {
    stageIndex = (stageIndex + 1) % GAME_LAUNCH_STAGE_DEFS.length;
    applyGameLaunchStage(stageIndex);
  }, 850);
}

function resolveLauncherPrefs(prefs = {}) {
  const gameVersion = normalizeGameVersion(
    prefs.gameVersion ?? storePrivate.settings.gameVersion
  );
  const hdVersion = normalizeHdVersion(
    prefs.hdVersion ?? storePrivate.settings.hdVersion
  );
  return {
    preloadControllerDlls:
      prefs.preloadControllerDlls ?? storePrivate.settings.preloadControllerDlls ?? false,
    friendSignature: normalizeFriendSignature(
      gameVersion,
      prefs.friendSignature ?? storePrivate.settings.friendSignature ?? "none",
      hdVersion
    ),
    winePrefixMode: normalizeWinePrefixMode(
      prefs.winePrefixMode ?? storePrivate.settings.winePrefixMode
    ),
    winePrefixCustomPath: normalizeWinePrefixCustomPath(
      prefs.winePrefixCustomPath ?? storePrivate.settings.winePrefixCustomPath
    ),
  };
}

function formatLauncherPrefArgs(prefs) {
  if (typeof window !== "undefined" && window.__TAURI__?.invoke) {
    return { prefs };
  }
  return prefs;
}

function scheduleLauncherPrefWrite(prefs) {
  pendingLauncherPrefs = { ...pendingLauncherPrefs, ...prefs };
  if (launcherPrefWriteTimer) clearTimeout(launcherPrefWriteTimer);
  launcherPrefWriteTimer = setTimeout(async () => {
    const payload = resolveLauncherPrefs(pendingLauncherPrefs);
    pendingLauncherPrefs = {};
    launcherPrefWriteTimer = null;
    try {
      await handleInvoke("set_launcher_pref", formatLauncherPrefArgs(payload));
    } catch (_error) {
      // swallow errors to keep UI responsive
    }
  }, SETTING_WRITE_DELAY_MS);
}
watch(
  () => storeMut.style,
  async (style) => {
    if (isHydratingStore) return;
    saveUiPrefs();
    const shouldReloadEndpoint = Boolean(storePrivate.currentEndpoint);
    if (shouldReloadEndpoint) {
      storePrivate.launcherAssetsLoading = true;
    }
    storeMut.page = LOGIN_PAGE;
    await handleInvoke("set_style", { style });
    clearLauncherImageState();
    if (shouldReloadEndpoint) {
      lastEndpointKey = null;
      await setCurrentEndpoint({ ...storePrivate.currentEndpoint });
    }
  }
);
function normalizeLocaleValue(locale) {
  const raw = String(locale ?? "").toLowerCase();
  if (raw.startsWith("jp") || raw.startsWith("ja")) return "jp";
  if (raw.startsWith("en")) return "en";
  return "en";
}

watch(
  () => storeMut.locale,
  async (locale) => {
    const normalized = normalizeLocaleValue(locale);
    setFluentLocale(normalized);
    if (normalized !== locale) {
      storeMut.locale = normalized;
      return;
    }
    if (isHydratingStore) return;
    await handleInvoke("set_locale", { locale: normalized });
  }
);
watch(
  () => storeMut.gameFolder,
  async (gameFolder, oldGameFolder) => {
    if (isHydratingStore) return;
    try {
      await handleInvoke("set_game_folder", { gameFolder });
      if (storePrivate.currentEndpoint) {
        await setCurrentEndpoint({ ...storePrivate.currentEndpoint }, { showLoading: false });
      }
    } catch (error) {
      if (error === "") return;
      storeMut.gameFolder = oldGameFolder;
    }
  }
);
watch(
  () => storeMut.serverlistUrl,
  async (serverlistUrl) => {
    if (isHydratingStore) return;
    await handleInvoke("set_serverlist_url", { serverlistUrl });
  }
);
watch(
  () => storeMut.messagelistUrl,
  async (messagelistUrl) => {
    if (isHydratingStore) return;
    await handleInvoke("set_messagelist_url", { messagelistUrl });
  }
);
const ONLINE_BRANCH_VERSIONS = new Set(["S6", "S7K"]);
const FORWARD_BRANCH_VERSIONS = new Set(["F4", "F5"]);
const G_BRANCH_VERSIONS = new Set([
  "G1",
  "G2",
  "G3",
  "G3.1",
  "G3.2",
  "GG",
  "G5",
  "G5.1",
  "G5.2",
  "G6",
  "G7",
  "G9.1",
  "G10.1",
]);

function headerBranchForVersion(version) {
  const normalized = normalizeGameVersion(version);
  if (ONLINE_BRANCH_VERSIONS.has(normalized)) return "online";
  if (FORWARD_BRANCH_VERSIONS.has(normalized)) return "forward";
  if (G_BRANCH_VERSIONS.has(normalized)) return "g";
  return "z";
}

function headerVariantKeyForVersion(version) {
  const branch = headerBranchForVersion(version);
  if (branch === "online") return "online";
  if (branch === "forward") return "forward";
  if (branch === "g") return "g";
  if (branch === "z") {
    return normalizeGameVersion(version) === "ZZ" ? "zz" : "z";
  }
  return "online";
}

function classicOfflineHeaderPath(version) {
  return headerBranchForVersion(version) === "z"
    ? "/headers/Mezeporta-Z.png"
    : "/headers/Mezeporta.png";
}

function classicConnectedFallbackHeaderPath(version) {
  switch (headerVariantKeyForVersion(version)) {
    case "online":
      return "/headers/MezeportaOnline.png";
    case "forward":
      return "/headers/MezeportaForward.png";
    case "g":
      return "/headers/MezeportaG.png";
    case "z":
      return "/headers/MezeportaZ.png";
    case "zz":
      return "/headers/MezeportaZZ.png";
    default:
      return "/headers/MezeportaFrontier.png";
  }
}

function ps4ConnectedFallbackHeaderPath(version) {
  switch (headerVariantKeyForVersion(version)) {
    case "forward":
      return "/headers/PS4/Forward.png";
    case "g":
      return "/headers/PS4/G.png";
    case "z":
      return "/headers/PS4/Z.png";
    case "zz":
      return "/headers/PS4/ZZ.png";
    default:
      return "/headers/PS4/Online.png";
  }
}

function classicRemoteHeaderUrlForVersion(version) {
  switch (headerVariantKeyForVersion(version)) {
    case "forward":
      return storePrivate.classicHeaderForward;
    case "g":
      return storePrivate.classicHeaderG;
    case "z":
      return storePrivate.classicHeaderZ;
    case "zz":
      return storePrivate.classicHeaderZZ;
    default:
      return storePrivate.classicHeaderOnline;
  }
}

function ps4RemoteHeaderUrlForVersion(version) {
  switch (headerVariantKeyForVersion(version)) {
    case "forward":
      return storePrivate.ps4HeaderForward;
    case "g":
      return storePrivate.ps4HeaderG;
    case "z":
      return storePrivate.ps4HeaderZ;
    case "zz":
      return storePrivate.ps4HeaderZZ;
    default:
      return storePrivate.ps4HeaderOnline;
  }
}

function headerUrlForVariant(variant, sources) {
  switch (variant) {
    case "forward":
      return sources.forward;
    case "g":
      return sources.g;
    case "z":
      return sources.z;
    case "zz":
      return sources.zz;
    default:
      return sources.online;
  }
}

function setClassicHeaderUrlForVariant(variant, value) {
  switch (variant) {
    case "forward":
      storePrivate.classicHeaderForward = value;
      break;
    case "g":
      storePrivate.classicHeaderG = value;
      break;
    case "z":
      storePrivate.classicHeaderZ = value;
      break;
    case "zz":
      storePrivate.classicHeaderZZ = value;
      break;
    default:
      storePrivate.classicHeaderOnline = value;
      break;
  }
}

function setPs4HeaderUrlForVariant(variant, value) {
  switch (variant) {
    case "forward":
      storePrivate.ps4HeaderForward = value;
      break;
    case "g":
      storePrivate.ps4HeaderG = value;
      break;
    case "z":
      storePrivate.ps4HeaderZ = value;
      break;
    case "zz":
      storePrivate.ps4HeaderZZ = value;
      break;
    default:
      storePrivate.ps4HeaderOnline = value;
      break;
  }
}

function isOfflineEndpoint(endpoint = storePrivate.currentEndpoint) {
  return !endpoint?.url || endpoint.url === "OFFLINEMODE";
}

export const backgroundUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? (
              offlineOverrideBucketForStyle().background ??
              (storeMut.style === PS4_STYLE
                ? assetUrl("/ps4/BackgroundALT5.png")
                : assetUrl(fallbackBackgroundPath()))
            )
          : (
              storeMut.style === PS4_STYLE
                ? (storePrivate.ps4Background ?? assetUrl("/ps4/BackgroundALT5.png"))
                : (storePrivate.background ?? assetUrl(fallbackBackgroundPath()))
            )
      )
);
export const cogUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? (
              offlineOverrideBucketForStyle().cog ??
              (storeMut.style === PS4_STYLE
                ? assetUrl("/ps4/cog.png")
                : assetUrl(styleAssetPath("cog.png")))
            )
          : (
              storeMut.style === PS4_STYLE
                ? (storePrivate.ps4Cog ?? assetUrl("/ps4/cog.png"))
                : (storePrivate.cog ?? assetUrl(styleAssetPath("cog.png")))
            )
      )
);
export const capcomUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? (
              offlineOverrideBucketForStyle().capcom ??
              (storeMut.style === PS4_STYLE
                ? assetUrl("/ps4/capcomALT.png")
                : assetUrl(styleAssetPath("capcomALT.png")))
            )
          : (
              storeMut.style === PS4_STYLE
                ? (storePrivate.ps4Capcom ?? assetUrl("/ps4/capcomALT.png"))
                : (storePrivate.capcom ?? assetUrl(styleAssetPath("capcomALT.png")))
            )
      )
);
export const launcherHeaderUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (() => {
        const overrideHeader = shouldUseOfflineImages()
          ? offlineOverrideBucketForStyle().header
          : null;
        if (storeMut.style === PS4_STYLE) {
          if (overrideHeader) return overrideHeader;
          if (isOfflineEndpoint()) {
            return assetUrl("/headers/PS4/MezeportaPS4.png");
          }
          return (
            (shouldUseOfflineImages()
              ? null
              : ps4RemoteHeaderUrlForVersion(storePrivate.settings.gameVersion)) ??
            assetUrl(ps4ConnectedFallbackHeaderPath(storePrivate.settings.gameVersion))
          );
        }
        if (storeMut.style === CLASSIC_STYLE) {
          if (overrideHeader) return overrideHeader;
          if (isOfflineEndpoint()) {
            return assetUrl(classicOfflineHeaderPath(storePrivate.settings.gameVersion));
          }
          return (
            (shouldUseOfflineImages()
              ? null
              : classicRemoteHeaderUrlForVersion(storePrivate.settings.gameVersion)) ??
            assetUrl(classicConnectedFallbackHeaderPath(storePrivate.settings.gameVersion))
          );
        }
        return assetUrl(styleAssetPath("launcher-headerALT3.png"));
      })()
);
export const dialogUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? (offlineOverrideBucketForStyle().dialogue ?? assetUrl("/extra/dialog.png"))
          : (storePrivate.dialogImage ?? assetUrl("/extra/dialog.png"))
      )
);
export const serverPatchUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? (offlineOverrideBucketForStyle().serverPatch ?? assetUrl("/extra/ServerPatch.png"))
          : (storePrivate.serverPatchImage ?? assetUrl("/extra/ServerPatch.png"))
      )
);
export const ps4ButtonUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? assetUrl("/ps4/Button.png")
          : (storePrivate.ps4Button ?? assetUrl("/ps4/Button.png"))
      )
);
export const ps4AddServerButtonUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? assetUrl("/ps4/Button.png")
          : (storePrivate.ps4AddServerButton ?? assetUrl("/ps4/Button.png"))
      )
);
export const classicButtonUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? assetUrl("/classic/btn-blue.png")
          : (storePrivate.classicButton ?? assetUrl("/classic/btn-blue.png"))
      )
);
export const classicAddServerButtonUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? assetUrl("/classic/btn-blueALT4.png")
          : (storePrivate.classicAddServerButton ?? assetUrl("/classic/btn-blueALT4.png"))
      )
);
export const ps4EmblemUrl = computed(() =>
  storePrivate.launcherSuspended
    ? ""
    : (
        shouldUseOfflineImages()
          ? assetUrl("/ps4/Emblem.png")
          : (storePrivate.ps4Emblem ?? assetUrl("/ps4/Emblem.png"))
      )
);

export const effectiveBanners = computed(() =>
  storePrivate.launcherSuspended
    ? []
    : store.banners.length
    ? store.banners
    : [
        {
          src: assetUrl("/banners/BannerWelcome.png"),
          link: "https://github.com/LilButter/Mezeporta",
        },
        {
          src: assetUrl("/banners/BannerShow1.png"),
          link: "https://github.com/LilButter/Mezeporta",
        }
      ]
);

// ---------------- Built-in fallback messages ----------------
const FALLBACK_MESSAGES = [
  {
    message: "Welcome to Mezeporta!",
    date:    Date.now() / 1000,   // seconds since epoch
    link:    "https://github.com/LilButter/Mezeporta",
    kind:    1                    // show in Announcements column
  },
  {
    message: "Add a server or make your own with Erupe!",
    date:    Date.now() / 1000,
    link:    "https://github.com/Mezeporta/Erupe",
    kind:    0                    // show in News column
  }
];


function normalizeFolderPath(folder) {
  if (folder == null) return null;
  const normalized = String(folder).trim().replace(/[\/]+$/, "");
  return normalized.length ? normalized : null;
}

function clearLauncherImageState() {
  storePrivate.background = null;
  storePrivate.cog = null;
  storePrivate.capcom = null;
  storePrivate.classicButton = null;
  storePrivate.classicAddServerButton = null;
  storePrivate.dialogImage = null;
  storePrivate.serverPatchImage = null;
  storePrivate.classicHeaderOnline = null;
  storePrivate.classicHeaderForward = null;
  storePrivate.classicHeaderG = null;
  storePrivate.classicHeaderZ = null;
  storePrivate.classicHeaderZZ = null;
  storePrivate.ps4Background = null;
  storePrivate.ps4Button = null;
  storePrivate.ps4AddServerButton = null;
  storePrivate.ps4Capcom = null;
  storePrivate.ps4Cog = null;
  storePrivate.ps4Emblem = null;
  storePrivate.ps4HeaderOnline = null;
  storePrivate.ps4HeaderForward = null;
  storePrivate.ps4HeaderG = null;
  storePrivate.ps4HeaderZ = null;
  storePrivate.ps4HeaderZZ = null;
}

const OFFLINE_IMAGE_EXTENSION_PRIORITY = ["png", "webp", "jpg", "jpeg"];
const OFFLINE_IMAGE_LOOKUP = {
  background: ["background"],
  capcom: ["capcom"],
  cog: ["cog"],
  dialogue: ["dialogue", "dialog"],
  header: ["header"],
  serverPatch: ["server-patch", "server_patch", "server patch", "serverpatch"],
};

function resetOfflineImageOverrides() {
  storePrivate.offlineImageOverrides.classic = createOfflineImageOverrideSet();
  storePrivate.offlineImageOverrides.ps4 = createOfflineImageOverrideSet();
}

function shouldUseOfflineImages() {
  return Boolean(storePrivate.settings.offlineImages);
}

function offlineOverrideBucketForStyle(style = storeMut.style) {
  return style === PS4_STYLE
    ? storePrivate.offlineImageOverrides.ps4
    : storePrivate.offlineImageOverrides.classic;
}

async function loadOfflineImageOverrideFolder(folderPath) {
  let entries = [];
  try {
    entries = await readDir(folderPath);
  } catch (_error) {
    return createOfflineImageOverrideSet();
  }

  const entryMap = new Map(
    entries
      .filter((entry) => !entry?.children && typeof entry?.name === "string")
      .map((entry) => [entry.name.toLowerCase(), entry.path])
      .filter(([, path]) => typeof path === "string" && path.length > 0)
  );

  const resolveAsset = (names) => {
    for (const name of names) {
      for (const extension of OFFLINE_IMAGE_EXTENSION_PRIORITY) {
        const match = entryMap.get(`${name}.${extension}`);
        if (match) return convertFileSrc(match);
      }
    }
    return null;
  };

  return {
    background: resolveAsset(OFFLINE_IMAGE_LOOKUP.background),
    capcom: resolveAsset(OFFLINE_IMAGE_LOOKUP.capcom),
    cog: resolveAsset(OFFLINE_IMAGE_LOOKUP.cog),
    dialogue: resolveAsset(OFFLINE_IMAGE_LOOKUP.dialogue),
    header: resolveAsset(OFFLINE_IMAGE_LOOKUP.header),
    serverPatch: resolveAsset(OFFLINE_IMAGE_LOOKUP.serverPatch),
  };
}

async function refreshOfflineImageOverrides() {
  if (typeof window === "undefined" || !window.__TAURI__ || !shouldUseOfflineImages()) {
    resetOfflineImageOverrides();
    return;
  }

  const folder = normalizeFolderPath(storeMut.gameFolder || storePrivate.currentFolder);
  if (!folder) {
    resetOfflineImageOverrides();
    return;
  }

  const classicFolder = await join(folder, "Mezeporta", "Offline-Images", "Classic");
  const ps4Folder = await join(folder, "Mezeporta", "Offline-Images", "PS4");
  const [classic, ps4] = await Promise.all([
    loadOfflineImageOverrideFolder(classicFolder),
    loadOfflineImageOverrideFolder(ps4Folder),
  ]);

  storePrivate.offlineImageOverrides.classic = classic;
  storePrivate.offlineImageOverrides.ps4 = ps4;
}

function applyOfflineFallbackUi() {
  storePrivate.banners = [];
  storePrivate.messages = [];
  storePrivate.links = [];
  storePrivate.launcherTag = DEFAULT_LAUNCHER_TAG;
  storePrivate.remoteMessages = [];
  clearLauncherImageState();
  bannerIndex.value = 0;
}

function normalizeLauncherTag(value) {
  const tag = String(value ?? "").trim();
  return tag || DEFAULT_LAUNCHER_TAG;
}

function launcherTagFromPayload(data) {
  return (
    data?.serverTag ??
    data?.server_tag ??
    data?.ServerTag ??
    data?.tag ??
    data?.Tag
  );
}

const STORAGE_KEYS = {
  uiPrefs: "ui_prefs_v1",
};
const DEFAULT_GAME_VERSION = "ZZ";
const CUSTOM_FONT_PRESET_REGEX = /^custom:[^/\\]+\.(ttf|ttc|otf)$/i;

function isSupportedFontPreset(value) {
  const preset = String(value ?? "").trim();
  if (!preset) return false;
  return preset === "default"
    || preset === "classic"
    || CUSTOM_FONT_PRESET_REGEX.test(preset);
}
const EMPTY_SIGNATURES = [];
const EMPTY_HD_SIGNATURES = { sd: [], hd: [] };
const S6_SIGNATURES = ["v1.13.3246"];
const S7K_SIGNATURES = ["v7.0.14_2"];
const F4_SIGNATURES = ["v1.20_107869"];
const F5_SIGNATURES = ["v1.20_125635", "v1.20_133710"];
const G1_SIGNATURES = ["v1.22_153077", "v1.22_156129"];
const G2_SIGNATURES = ["v1.23_187828"];
const G3_SIGNATURES = ["v1.27_211402", "v1.27_212295"];
const G3_1_SIGNATURES = ["v1.27_213258", "v1.27_215335", "v1.27_217155"];
const G3_2_SIGNATURES = ["v1.27_222273", "v1.27_223087"];
const GG_SIGNATURES = ["v1.28_246880"];
const G5_1_SIGNATURES = ["v1.30.283838"];
const G5_2_SIGNATURES = ["v1.32_302094"];
const G6_SIGNATURES = {
  sd: ["v1.33_325336"],
  hd: ["v1.33_326088"],
};
const G7_SIGNATURES = {
  sd: ["v1.36.05_936940dd"],
  hd: ["v1.36.05_a924ce4d"],
};
const G9_1_SIGNATURES = {
  sd: ["v1.38.19_e8966870"],
  hd: ["v1.38.19_47c90390"],
};
const G10_1_SIGNATURES = {
  sd: ["v1.41.30_c730c673", "v1.41.32_8acc3715"],
  hd: ["v1.41.30_f5ed3a6a", "v1.41.32_5c06b547"],
};
const Z1_SIGNATURES = {
  sd: ["v1.44.45_15a73eb7"],
  hd: ["v1.44.45_dca95f5f"],
};
const ZZ_SIGNATURES = {
  sd: ["v1.52.79_04d16dc4"],
  hd: ["v1.52.79_73c49f52"],
};

const FRIEND_SIGNATURE_OPTIONS = {
  S6: S6_SIGNATURES,
  S7K: S7K_SIGNATURES,
  F4: F4_SIGNATURES,
  F5: F5_SIGNATURES,
  G1: G1_SIGNATURES,
  G2: G2_SIGNATURES,
  G3: G3_SIGNATURES,
  "G3.1": G3_1_SIGNATURES,
  "G3.2": G3_2_SIGNATURES,
  GG: GG_SIGNATURES,
  "G5.1": G5_1_SIGNATURES,
  "G5.2": G5_2_SIGNATURES,
  G6: G6_SIGNATURES,
  G7: G7_SIGNATURES,
  "G9.1": G9_1_SIGNATURES,
  "G10.1": G10_1_SIGNATURES,
  Z1: Z1_SIGNATURES,
  Z2T: EMPTY_HD_SIGNATURES,
  ZZ: ZZ_SIGNATURES,
};

const FRIEND_SIGNATURE_SUPPORTED = FRIEND_SIGNATURE_OPTIONS;

const FRIEND_SIGNATURE_VERSION_TAGS = {
  "v1.13.3246": "S6",
  "v7.0.14_2": "S7K",
  "v1.20_107869": "F4",
  "v1.20_125635": "F5",
  "v1.20_133710": "F5",
  "v1.22_153077": "G1",
  "v1.22_156129": "G1",
  "v1.23_187828": "G2",
  "v1.27_211402": "G3",
  "v1.27_212295": "G3",
  "v1.27_213258": "G3.1",
  "v1.27_215335": "G3.1",
  "v1.27_217155": "G3.1",
  "v1.27_222273": "G3.2",
  "v1.27_223087": "G3.2",
  "v1.28_246880": "GG",
  "v1.30.283838": "G5.1",
  "v1.32_302094": "G5.2",
  "v1.33_325336": "G6 SD",
  "v1.33_326088": "G6 HD",
  "v1.36.05_936940dd": "G7 SD",
  "v1.36.05_a924ce4d": "G7 HD",
  "v1.38.19_e8966870": "G9.1 SD",
  "v1.38.19_47c90390": "G9.1 HD",
  "v1.41.30_c730c673": "G10.1 SD",
  "v1.41.30_f5ed3a6a": "G10.1 HD",
  "v1.41.32_8acc3715": "G10.1 SD",
  "v1.41.32_5c06b547": "G10.1 HD",
  "v1.44.45_15a73eb7": "Z1 SD",
  "v1.44.45_dca95f5f": "Z1 HD",
  "v1.52.79_04d16dc4": "ZZ SD",
  "v1.52.79_73c49f52": "ZZ HD",
};

const GAME_VERSION_ALIASES = {
  S6: "S6",
  "SEASON6": "S6",
  "SEASON6.0": "S6",
  S7K: "S7K",
  "SEASON7": "S7K",
  "SEASON7.0": "S7K",
  F4: "F4",
  FW4: "F4",
  "FW.4": "F4",
  "FW_4": "F4",
  F5: "F5",
  FW5: "F5",
  "FW.5": "F5",
  "FW_5": "F5",
  G1: "G1",
  G2: "G2",
  G3: "G3",
  "G3.1": "G3.1",
  G3_1: "G3.1",
  "G3.2": "G3.2",
  G3_2: "G3.2",
  GG: "GG",
  G4: "GG",
  "G5.1": "G5.1",
  G5_1: "G5.1",
  "G5.2": "G5.2",
  G5_2: "G5.2",
  G5: "G5",
  G6: "G6",
  "G6.1": "G6",
  G7: "G7",
  G9: "G9.1",
  "G9.1": "G9.1",
  G9_1: "G9.1",
  G10: "G10.1",
  "G10.1": "G10.1",
  G10_1: "G10.1",
  Z1: "Z1",
  Z2T: "Z2T",
  Z2TW: "Z2T",
  "Z1.1": "Z1",
  "Z1.2": "Z1",
  Z2: "Z2",
  "Z2.1": "Z1",
  "Z2.2": "Z2T",
  "Z2.3": "Z1",
  ZZ: "ZZ",
  Z3: "ZZ",
  "Z3.1": "ZZ",
};

function normalizeHdVersion(value) {
  return value === true || value === 1 || value === "1";
}

function resolveFriendSignatureOptions(version, hdVersion = false) {
  const normalizedVersion = normalizeGameVersion(version);
  const options = FRIEND_SIGNATURE_OPTIONS[normalizedVersion];
  if (!options) return [];
  if (Array.isArray(options)) return options;
  return normalizeHdVersion(hdVersion) ? (options.hd ?? []) : (options.sd ?? []);
}

function resolveSupportedFriendSignatures(version, hdVersion = false) {
  const normalizedVersion = normalizeGameVersion(version);
  const options = FRIEND_SIGNATURE_SUPPORTED[normalizedVersion];
  if (!options) return [];
  if (Array.isArray(options)) return options;
  return normalizeHdVersion(hdVersion) ? (options.hd ?? []) : (options.sd ?? []);
}

export function friendSignatureOptionsForVersion(version, hdVersion = false) {
  return resolveFriendSignatureOptions(version, hdVersion);
}

export function friendSignatureEntriesForVersion(version, hdVersion = false) {
  const supported = new Set(resolveSupportedFriendSignatures(version, hdVersion));
  return resolveFriendSignatureOptions(version, hdVersion).map((signature) => ({
    signature,
    enabled: supported.has(signature),
  }));
}

export function friendSignatureDisplayLabel(signature) {
  const value = String(signature ?? "").trim();
  if (!value) return "";
  return value;
}

function normalizeFriendSignature(version, signature, hdVersion = false) {
  const trimmed = String(signature ?? "").trim();
  const lower = trimmed.toLowerCase();
  if (!trimmed || lower === "none") return "none";
  if (lower === "detect" || lower === "detect-beta") return "none";

  const options = resolveFriendSignatureOptions(version, hdVersion);
  if (!options.includes(trimmed)) {
    return "none";
  }

  const supported = new Set(resolveSupportedFriendSignatures(version, hdVersion));
  return supported.has(trimmed) ? trimmed : "none";
}

function normalizeGameVersion(version) {
  const normalized = String(version ?? "").trim();
  if (GAME_VERSIONS.includes(normalized)) return normalized;

  const compact = normalized.toUpperCase().replace(/\s+/g, "");
  return GAME_VERSION_ALIASES[compact] ?? DEFAULT_GAME_VERSION;
}

function toBackendEndpointVersion(version) {
  return normalizeGameVersion(version);
}

function normalizeClientModeToken(clientMode) {
  const value = String(clientMode ?? "")
    .replace(/\s*\(debug\s+only\)\s*$/i, "")
    .trim()
    .toUpperCase();
  if (!value) return "";
  return value.replace(/\s+/g, "").replace(/[()]/g, "");
}

function parseClientModeNumber(token, prefix) {
  const match = token.match(new RegExp(`^${prefix}(\\d+)(?:\\.(\\d+))?`));
  if (!match) return null;
  return {
    major: Number(match[1] ?? "0"),
    minor: Number(match[2] ?? "0"),
  };
}

function mapClientModeToGameVersion(clientMode) {
  const token = normalizeClientModeToken(clientMode);
  if (!token) return null;

  if (token === "ZZ") return "ZZ";
  if (token === "Z2") return "Z2";
  if (token === "Z2.2") return "Z2T";

  if (token.startsWith("Z")) {
    const parsed = parseClientModeNumber(token, "Z");
    if (!parsed) return null;
    if (parsed.major >= 3) return "ZZ";
    if (parsed.major === 2) {
      if (parsed.minor >= 2) return "Z2T";
      if (parsed.minor === 0) return "Z2";
      return "Z1";
    }
    if (parsed.major >= 1) return "Z1";
    return null;
  }

  if (token.startsWith("SEASON") || /^S\d/.test(token)) {
    const match = token.match(/^(?:SEASON|S)(\d+)(?:\.(\d+))?([A-Z]*)$/);
    if (!match) return null;
    const major = Number(match[1] ?? "0");
    const suffix = match[3] ?? "";
    if (major === 6) return "S6";
    if (major === 7 && (suffix === "" || suffix === "K")) return "S7K";
    return null;
  }

  if (token === "FORWARD" || token === "FW") return "F5";
  if (
    token.startsWith("FORWARD") ||
    token.startsWith("FW") ||
    /^F\d/.test(token)
  ) {
    const match = token.match(/(\d+(?:\.\d+)?)/);
    const parsed = Number(match?.[1]);
    if (!Number.isFinite(parsed)) return "F5";
    if (parsed >= 5) return "F5";
    if (parsed >= 4) return "F4";
    return null;
  }

  if (token === "GG" || token === "G4") return "GG";

  if (token.startsWith("G")) {
    const parsed = parseClientModeNumber(token, "G");
    if (!parsed) return null;
    if (parsed.major === 1) return "G1";
    if (parsed.major === 2) return "G2";
    if (parsed.major === 3) {
      if (parsed.minor >= 2) return "G3.2";
      if (parsed.minor >= 1) return "G3.1";
      return "G3";
    }
    if (parsed.major === 4) return "GG";
    if (parsed.major === 5) {
      if (parsed.minor >= 2) return "G5.2";
      if (parsed.minor >= 1) return "G5.1";
      return "G5";
    }
    if (parsed.major === 6) return "G6";
    if (parsed.major === 7) return "G7";
    if (parsed.major === 9) return "G9.1";
    if (parsed.major === 10) return "G10.1";
    return null;
  }

  return null;
}

function signatureFamilyKeyForVersion(version) {
  const normalizedVersion = normalizeGameVersion(version);
  if ([
    "S6",
    "S7K",
    "F4",
    "F5",
    "G1",
    "G2",
    "G3",
    "G3.1",
    "G3.2",
    "GG",
    "G5.1",
    "G5.2",
    "G6",
    "G7",
    "G9.1",
    "G10.1",
    "Z1",
    "ZZ",
  ].includes(normalizedVersion)) {
    return normalizedVersion;
  }
  return null;
}

function mapClientModeToSignatureKey(clientMode) {
  const version = mapClientModeToGameVersion(clientMode);
  if (!version) return null;
  if (Object.prototype.hasOwnProperty.call(CLIENT_MODE_SIGNATURES, version)) {
    return version;
  }
  return signatureFamilyKeyForVersion(version);
}

const CLIENT_MODE_SIGNATURES = {
  S6: { sd: "v1.13.3246", hd: "v1.13.3246" },
  S7K: { sd: "v7.0.14_2", hd: "v7.0.14_2" },
  F4: { sd: "v1.20_107869", hd: "v1.20_107869" },
  F5: { sd: "v1.20_133710", hd: "v1.20_133710" },
  G1: { sd: "v1.22_156129", hd: "v1.22_156129" },
  G2: { sd: "v1.23_187828", hd: "v1.23_187828" },
  G3: { sd: "v1.27_212295", hd: "v1.27_212295" },
  "G3.1": { sd: "v1.27_217155", hd: "v1.27_217155" },
  "G3.2": { sd: "v1.27_223087", hd: "v1.27_223087" },
  GG: { sd: "v1.28_246880", hd: "v1.28_246880" },
  "G5.1": { sd: "v1.30.283838", hd: "v1.30.283838" },
  "G5.2": { sd: "v1.32_302094", hd: "v1.32_302094" },
  G6: { sd: "v1.33_325336", hd: "v1.33_326088" },
  G7: { sd: "v1.36.05_936940dd", hd: "v1.36.05_a924ce4d" },
  "G9.1": { sd: "v1.38.19_e8966870", hd: "v1.38.19_47c90390" },
  "G10.1": { sd: "v1.41.30_c730c673", hd: "v1.41.30_f5ed3a6a" },
  Z1: { sd: "v1.44.45_15a73eb7", hd: "v1.44.45_dca95f5f" },
  ZZ: { sd: "v1.52.79_04d16dc4", hd: "v1.52.79_73c49f52" },
};
function resolveSignatureFromClientMode(gameVersion, clientMode, hdVersion = false) {
  const key = mapClientModeToSignatureKey(clientMode)
    ?? signatureFamilyKeyForVersion(gameVersion);
  if (!key) return "none";
  const mapping = CLIENT_MODE_SIGNATURES[key];
  if (!mapping) return "none";
  const preferred = normalizeHdVersion(hdVersion) ? mapping.hd ?? mapping.sd : mapping.sd;
  return normalizeFriendSignature(gameVersion, preferred, hdVersion);
}

async function fetchEndpointVersionInfo(endpoint) {
  if (!endpoint) return null;
  if (!endpoint.url || endpoint.url === "OFFLINEMODE") return null;

  const base = normalizeEndpointBase(endpoint);
  if (!base) return null;

  const normalizeVersionPayload = (payload) => {
    if (!payload || typeof payload !== "object") {
      return null;
    }

    const readField = (keys) => {
      for (const key of keys) {
        const value = payload[key];
        if (value === null || value === undefined) continue;
        const text = String(value).trim();
        if (text) return text;
      }
      return "";
    };

    const clientMode = readField([
      "clientMode",
      "client_mode",
      "clientModeId",
      "clientModeID",
      "client_mode_id",
      "ClientModeId",
      "ClientModeID",
      "version",
      "Version",
    ]);

    if (!clientMode) {
      return null;
    }

    return {
      endpointKey: endpointKey(endpoint),
      name: readField(["name", "Name"]),
      clientMode,
      mappedGameVersion: mapClientModeToGameVersion(clientMode),
    };
  };

  const backendEndpoint = normalizeEndpointForBackend(endpoint);

  try {
    const payload = await handleInvoke("get_server_version_info", {
      endpoint: backendEndpoint,
    });
    const info = normalizeVersionPayload(payload);
    if (info) {
      return info;
    }
  } catch (_error) {
    // Ignore backend failures here; login can continue without strict version check.
  }

  const urls = [`${base}/v2/version`, `${base}/version`];

  for (const url of urls) {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 3500);

    try {
      const response = await fetch(url, {
        method: "GET",
        headers: { Accept: "application/json" },
        cache: "no-store",
        signal: controller.signal,
      });
      if (!response.ok) {
        continue;
      }

      const payload = await response.json().catch(() => null);
      const info = normalizeVersionPayload(payload);
      if (info) {
        return info;
      }
    } catch (_error) {
      // Ignore failures; login can continue without strict version check.
    } finally {
      clearTimeout(timeoutId);
    }
  }

  return null;
}

async function ensureServerVersionInfoForLogin(endpoint) {
  if (!endpoint) return null;
  if (!endpoint.url || endpoint.url === "OFFLINEMODE") return null;

  const targetKey = endpointKey(endpoint);
  const cached = storePrivate.serverVersionInfo;
  const fetched = await fetchEndpointVersionInfo(endpoint);
  if (
    fetched &&
    endpointKey(storePrivate.currentEndpoint) === targetKey
  ) {
    storePrivate.serverVersionInfo = fetched;
    return fetched;
  }
  if (cached?.endpointKey === targetKey) {
    return cached;
  }
  return fetched;
}

function versionMismatchPrompt(versionInfo, selectedVersion) {
  const targetVersion = String(versionInfo?.mappedGameVersion ?? "").trim();
  const label = targetVersion || String(versionInfo?.clientMode ?? "").trim();
  if (!label || !selectedVersion) return "";
  return getMessage("version-switch-message", { version: label });
}

function normalizeEndpointForUi(endpoint) {
  if (!endpoint || typeof endpoint !== "object") return endpoint;
  const { legacy, ...rest } = endpoint;
  return { ...rest };
}

function normalizeEndpointCollectionForUi(endpoints) {
  if (!Array.isArray(endpoints)) return [];
  return endpoints.map((endpoint) => normalizeEndpointForUi(endpoint));
}

function normalizeEndpointForBackend(endpoint) {
  const { legacy, ...rest } = endpoint;
  const normalized = {
    ...rest,
    launcherPort: endpoint.launcherPort || null,
    gamePort: endpoint.gamePort || null,
    version: toBackendEndpointVersion(endpoint.version ?? storePrivate.settings.gameVersion),
  };
  if (!normalized.name || !String(normalized.name).trim()) {
    normalized.name = getEndpointServerKey(normalized) || "Server";
  }
  return normalized;
}


function applyUiPrefs(prefs) {
  if (!prefs || typeof prefs !== "object") return;
  const nextDevMode =
    typeof prefs.devMode === "boolean"
      ? prefs.devMode
      : storePrivate.settings.devMode;
  if (typeof prefs.devMode === "boolean") {
    storePrivate.settings.devMode = prefs.devMode;
  }
  if (typeof prefs.style === "string") {
    storeMut.style = normalizeLauncherStyle(prefs.style, nextDevMode);
  }
  if (typeof prefs.sfxEnabled === "boolean") storePrivate.settings.sfxEnabled = prefs.sfxEnabled;
  if (typeof prefs.sfxVolume === "number") storePrivate.settings.sfxVolume = prefs.sfxVolume;
  if (isSupportedFontPreset(prefs.fontPreset)) {
    storePrivate.settings.fontPreset = prefs.fontPreset;
  }
  if (typeof prefs.gameVersion === "string") {
    storePrivate.settings.gameVersion = normalizeGameVersion(prefs.gameVersion);
  }
  if (typeof prefs.forceGameVersion === "boolean") {
    storePrivate.settings.forceGameVersion = prefs.forceGameVersion;
  }
  if (typeof prefs.externalLinkPromptDisabled === "boolean") {
    storePrivate.settings.externalLinkPromptDisabled = prefs.externalLinkPromptDisabled;
  }
  if (typeof prefs.serverVersionPromptDisabled === "boolean") {
    storePrivate.settings.serverVersionPromptDisabled = prefs.serverVersionPromptDisabled;
  }
  if (typeof prefs.launcherController === "boolean") {
    storePrivate.settings.launcherController = prefs.launcherController;
  }
  if (typeof prefs.linuxHardwareAcceleration === "boolean") {
    storePrivate.settings.linuxHardwareAcceleration = prefs.linuxHardwareAcceleration;
  }
  if (typeof prefs.offlineImages === "boolean") {
    storePrivate.settings.offlineImages = prefs.offlineImages;
  }
  applyLauncherResolutionPrefs(prefs);
}

function readLocalUiPrefs() {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEYS.uiPrefs) || "{}");
  } catch (_) {
    return null;
  }
}

function loadUiPrefs() {
  const localPrefs = readLocalUiPrefs();
  if (localPrefs) applyUiPrefs(localPrefs);
}

function primeLocalLauncherStyle() {
  if (typeof window === "undefined") return;
  const localPrefs = readLocalUiPrefs();
  if (!localPrefs || typeof localPrefs !== "object") return;

  const wasHydratingStore = isHydratingStore;
  isHydratingStore = true;
  try {
    const localDevMode =
      typeof localPrefs.devMode === "boolean"
        ? localPrefs.devMode
        : storePrivate.settings.devMode;
    if (typeof localPrefs.devMode === "boolean") {
      storePrivate.settings.devMode = localDevMode;
    }
    if (typeof localPrefs.style === "string") {
      storeMut.style = normalizeLauncherStyle(localPrefs.style, localDevMode);
    }
  } finally {
    isHydratingStore = wasHydratingStore;
  }
}

primeLocalLauncherStyle();

function saveUiPrefs() {
  const classicKeys = getLauncherResolutionPrefKeys(CLASSIC_STYLE);
  const ps4Keys = getLauncherResolutionPrefKeys(PS4_STYLE);
  const classicRecentKey = getLauncherRecentResolutionPrefKey(CLASSIC_STYLE);
  const ps4RecentKey = getLauncherRecentResolutionPrefKey(PS4_STYLE);
  const classicCustomKey = getLauncherCustomResolutionPrefKey(CLASSIC_STYLE);
  const ps4CustomKey = getLauncherCustomResolutionPrefKey(PS4_STYLE);
  localStorage.setItem(
    STORAGE_KEYS.uiPrefs,
    JSON.stringify({
      style: storeMut.style,
      sfxEnabled: storePrivate.settings.sfxEnabled,
      sfxVolume: storePrivate.settings.sfxVolume,
      fontPreset: storePrivate.settings.fontPreset,
      gameVersion: normalizeGameVersion(storePrivate.settings.gameVersion),
        forceGameVersion: Boolean(storePrivate.settings.forceGameVersion),
      devMode: Boolean(storePrivate.settings.devMode),
      externalLinkPromptDisabled: Boolean(storePrivate.settings.externalLinkPromptDisabled),
      serverVersionPromptDisabled: Boolean(storePrivate.settings.serverVersionPromptDisabled),
      launcherController: Boolean(storePrivate.settings.launcherController),
      linuxHardwareAcceleration: Boolean(storePrivate.settings.linuxHardwareAcceleration),
      offlineImages: Boolean(storePrivate.settings.offlineImages),
      [classicKeys.width]: storePrivate.settings[classicKeys.width],
      [classicKeys.height]: storePrivate.settings[classicKeys.height],
      [classicCustomKey]: Boolean(storePrivate.settings[classicCustomKey]),
      [classicRecentKey]: normalizeLauncherRecentResolutionList(
        storePrivate.settings[classicRecentKey]
      ),
      [ps4Keys.width]: storePrivate.settings[ps4Keys.width],
      [ps4Keys.height]: storePrivate.settings[ps4Keys.height],
      [ps4CustomKey]: Boolean(storePrivate.settings[ps4CustomKey]),
      [ps4RecentKey]: normalizeLauncherRecentResolutionList(
        storePrivate.settings[ps4RecentKey]
      ),
      })
    );
}


function syncUiPrefsToBackend(previousPrefs = null) {
  if (typeof window === "undefined") return;
  const gameVersion = normalizeGameVersion(storePrivate.settings.gameVersion);
  const forceGameVersion = Boolean(storePrivate.settings.forceGameVersion);
  const serverVersionPromptDisabled = Boolean(storePrivate.settings.serverVersionPromptDisabled);
  const offlineImages = Boolean(storePrivate.settings.offlineImages);

  const previousGameVersion = normalizeGameVersion(previousPrefs?.gameVersion);
  const previousForceGameVersion = Boolean(previousPrefs?.forceGameVersion);
  const previousServerVersionPromptDisabled = Boolean(previousPrefs?.serverVersionPromptDisabled);
  const previousOfflineImages = Boolean(previousPrefs?.offlineImages);

  if (previousPrefs && previousGameVersion === gameVersion) {
    if (
      previousForceGameVersion === forceGameVersion &&
      previousServerVersionPromptDisabled === serverVersionPromptDisabled &&
      previousOfflineImages === offlineImages
    ) {
      return;
    }
  }

  setUiPreference("gameVersion", gameVersion);
  setUiPreference("forceGameVersion", forceGameVersion);
  setUiPreference("serverVersionPromptDisabled", serverVersionPromptDisabled);
  setUiPreference("offlineImages", offlineImages);
}

// Expose to components
export function setUiPref(name, value) {
  storePrivate.settings[name] = value;
  saveUiPrefs();
  if (typeof window !== "undefined") {
    uiPrefSyncPromise = uiPrefSyncPromise
      .catch(() => undefined)
      .then(() => setUiPreference(name, value))
      .catch(() => {
        saveUiPrefs();
      });
  }
  return uiPrefSyncPromise;
}

export function setGameVersion(version) {
  const nextVersion = normalizeGameVersion(version);
  if (storePrivate.settings.gameVersion !== nextVersion) {
    setUiPref("gameVersion", nextVersion);
  }

  const nextSignature = normalizeFriendSignature(
    nextVersion,
    storePrivate.settings.friendSignature,
    storePrivate.settings.hdVersion
  );
  setLauncherPrefs({
    friendSignature: nextSignature,
    gameVersion: nextVersion,
    hdVersion: storePrivate.settings.hdVersion,
  });

  if (!storePrivate.settings.forceGameVersion) {
    setUiPref("forceGameVersion", true);
  }
  if (storePrivate.currentEndpoint) {
    storePrivate.currentEndpoint = {
      ...storePrivate.currentEndpoint,
      version: toBackendEndpointVersion(nextVersion),
    };
  }
}

export function setForceGameVersion(enabled) {
  const next = Boolean(enabled);
  if (storePrivate.settings.forceGameVersion !== next) {
    setUiPref("forceGameVersion", next);
  }
}


export function setRange(name, e) {
  const v = Number(e.target.value);
  if (!Number.isFinite(v)) return;
  setUiPref(name, v);
}

export const effectiveMessages = computed(
  () => store.messages.length ? store.messages : FALLBACK_MESSAGES
);

export const effectiveFolder = computed(
  () => storeMut.gameFolder || storePrivate.currentFolder
);

watch(
  effectiveFolder,
  () => {
    void refreshOfflineImageOverrides();
  },
  { immediate: true }
);

watch(
  () => storePrivate.settings.offlineImages,
  async () => {
    if (isHydratingStore) {
      await refreshOfflineImageOverrides();
      return;
    }
    await refreshOfflineImageOverrides();
    if (!storePrivate.currentEndpoint) return;
    lastEndpointKey = null;
    await setCurrentEndpoint({ ...storePrivate.currentEndpoint }, { showLoading: false });
  }
);

watch(
  () => storePrivate.settings.gameVersion,
  async () => {
    if (isHydratingStore) return;
    if (!storePrivate.currentEndpoint?.url || storePrivate.currentEndpoint.url === "OFFLINEMODE") {
      return;
    }
    lastEndpointKey = null;
    await setCurrentEndpoint({ ...storePrivate.currentEndpoint }, { showLoading: false });
  }
);

export async function initStore() {
  const data = await handleInvoke("initial_data");
  isHydratingStore = true;

  try {

  // regular launcher state
  storeMut.style         = data.style;
  storeMut.locale        = data.locale;
  storeMut.username      = data.username;
  storeMut.password      = data.password;
  storeMut.rememberMe    = data.rememberMe;
  storeMut.gameFolder    = data.gameFolder;
  storeMut.serverlistUrl = data.serverlistUrl;
  storeMut.messagelistUrl= data.messagelistUrl;

  storePrivate.endpoints        = normalizeEndpointCollectionForUi(data.endpoints);
  storePrivate.remoteEndpoints  = normalizeEndpointCollectionForUi(data.remoteEndpoints);
  storePrivate.currentEndpoint  = normalizeEndpointForUi(data.currentEndpoint);
  storePrivate.currentFolder    = data.currentFolder;
  storePrivate.remoteMessages   = data.remoteMessages;
  storePrivate.lastCharId       = data.lastCharId;
  storePrivate.launcherTag = normalizeLauncherTag(launcherTagFromPayload(data));

  // Ensure currentEndpoint is never null. Several components assume it exists.
  if (!storePrivate.currentEndpoint) {
    const firstRemote = Array.isArray(storePrivate.remoteEndpoints) && storePrivate.remoteEndpoints.length
      ? storePrivate.remoteEndpoints[0]
      : null;
    const firstLocal = Array.isArray(storePrivate.endpoints) && storePrivate.endpoints.length
      ? storePrivate.endpoints[0]
      : null;

    storePrivate.currentEndpoint = firstRemote || firstLocal || { name: "Offline-Mode", url: "OFFLINEMODE" };
  }


  // IMPORTANT: merge, don't clobber
  if (data.settings) {
    Object.assign(storePrivate.settings, data.settings);
  }
  if (data.uiPrefs) {
    applyUiPrefs(data.uiPrefs);
  }
  const persistedFriendSignature =
    data.launcherPrefs && typeof data.launcherPrefs.friendSignature === "string"
      ? data.launcherPrefs.friendSignature
      : (storePrivate.settings.friendSignature ?? "none");
  const persistedWinePrefixMode = normalizeWinePrefixMode(
    data.launcherPrefs?.winePrefixMode ?? storePrivate.settings.winePrefixMode
  );
  const persistedWinePrefixCustomPath = normalizeWinePrefixCustomPath(
    data.launcherPrefs?.winePrefixCustomPath ?? storePrivate.settings.winePrefixCustomPath
  );

  if (data.launcherPrefs && typeof data.launcherPrefs.preloadControllerDlls === "boolean") {
    storePrivate.settings.preloadControllerDlls = data.launcherPrefs.preloadControllerDlls;
  }
  if (typeof persistedFriendSignature === "string") {
    storePrivate.settings.friendSignature = persistedFriendSignature;
  }
  storePrivate.settings.winePrefixMode = persistedWinePrefixMode;
  storePrivate.settings.winePrefixCustomPath = persistedWinePrefixCustomPath;

  // ensure defaults exist (in case backend does not know them)
  if (storePrivate.settings.sfxEnabled === undefined) storePrivate.settings.sfxEnabled = false;
  if (storePrivate.settings.sfxVolume  === undefined) storePrivate.settings.sfxVolume  = 30;
  if (!isSupportedFontPreset(storePrivate.settings.fontPreset)) {
    storePrivate.settings.fontPreset = "default";
  }
  if (storePrivate.settings.preloadControllerDlls === undefined) {
    storePrivate.settings.preloadControllerDlls = false;
  }
  if (storePrivate.settings.friendSignature === undefined) {
    storePrivate.settings.friendSignature = "none";
  }
  if (storePrivate.settings.winePrefixMode === undefined) {
    storePrivate.settings.winePrefixMode = "portable";
  }
  storePrivate.settings.winePrefixMode = normalizeWinePrefixMode(
    storePrivate.settings.winePrefixMode
  );
  storePrivate.settings.winePrefixCustomPath = normalizeWinePrefixCustomPath(
    storePrivate.settings.winePrefixCustomPath
  );
  storePrivate.settings.gameVersion = normalizeGameVersion(storePrivate.settings.gameVersion);
  if (storePrivate.settings.forceGameVersion === undefined) {
    storePrivate.settings.forceGameVersion = false;
  }
  if (storePrivate.settings.devMode === undefined) {
    storePrivate.settings.devMode = false;
  }
  if (storePrivate.settings.externalLinkPromptDisabled === undefined) {
    storePrivate.settings.externalLinkPromptDisabled = false;
  }
  if (storePrivate.settings.serverVersionPromptDisabled === undefined) {
    storePrivate.settings.serverVersionPromptDisabled = false;
  }
  if (storePrivate.settings.launcherController === undefined) {
    storePrivate.settings.launcherController = true;
  }
  if (storePrivate.settings.linuxHardwareAcceleration === undefined) {
    storePrivate.settings.linuxHardwareAcceleration = true;
  }
  if (storePrivate.settings.offlineImages === undefined) {
    storePrivate.settings.offlineImages = false;
  }

  const normalizedStyle = normalizeLauncherStyle(
    storeMut.style,
    storePrivate.settings.devMode
  );
  storeMut.style = normalizedStyle;
  await handleInvoke("set_style", { style: normalizedStyle }).catch(() => {});

  applyOfflineFallbackUi();
  // Force remote launcher assets to be refreshed on init/resume.
  // Without this, setCurrentEndpoint can no-op on same endpoint key and keep offline placeholders.
  lastEndpointKey = null;

  let refreshedEndpointAssets = false;
  let endpointAssetRefreshFailed = false;

  // Apply local UI prefs after backend prefs so last user choice is never lost on fast close/reopen.
  loadUiPrefs();
  syncUiPrefsToBackend(data.uiPrefs ?? null);
  saveUiPrefs();
  storePrivate.settings.friendSignature = normalizeFriendSignature(
    storePrivate.settings.gameVersion,
    persistedFriendSignature,
    storePrivate.settings.hdVersion
  );
  setLauncherPreference({
    preloadControllerDlls: storePrivate.settings.preloadControllerDlls,
    friendSignature: storePrivate.settings.friendSignature,
    winePrefixMode: storePrivate.settings.winePrefixMode,
    winePrefixCustomPath: storePrivate.settings.winePrefixCustomPath,
  });
  if (storePrivate.currentEndpoint) {
    storePrivate.currentEndpoint = {
      ...storePrivate.currentEndpoint,
      version: toBackendEndpointVersion(storePrivate.settings.gameVersion),
    };
  }

  if (
    storePrivate.currentEndpoint?.url &&
    storePrivate.currentEndpoint.url !== "OFFLINEMODE"
  ) {
    try {
      await setCurrentEndpoint(storePrivate.currentEndpoint);
      refreshedEndpointAssets = true;
    } catch (_error) {
      endpointAssetRefreshFailed = true;
      // Keep fallback assets without noisy startup warning spam.
    }
  }

  if (!refreshedEndpointAssets && !endpointAssetRefreshFailed) {
    const resolveInitialAsset = (path) =>
      resolveEndpointAsset(path, storePrivate.currentEndpoint);
    if (shouldUseOfflineImages()) {
      clearLauncherImageState();
    } else {
      const activeHeaderVariant = headerVariantKeyForVersion(storePrivate.settings.gameVersion);
      applyCachedImage(resolveInitialAsset(data.dialog ?? data.dialogue), (value) => {
        storePrivate.dialogImage = value;
      });
      applyCachedImage(resolveInitialAsset(data.serverPatch ?? data.server_patch), (value) => {
        storePrivate.serverPatchImage = value;
      });
      if (storeMut.style === PS4_STYLE) {
        applyCachedImage(resolveInitialAsset(data.ps4?.background), (value) => {
          storePrivate.ps4Background = value;
        });
        applyCachedImage(resolveInitialAsset(data.ps4?.button), (value) => {
          storePrivate.ps4Button = value;
        });
        applyCachedImage(resolveInitialAsset(data.ps4?.addServerButton), (value) => {
          storePrivate.ps4AddServerButton = value;
        });
        applyCachedImage(resolveInitialAsset(data.ps4?.capcom), (value) => {
          storePrivate.ps4Capcom = value;
        });
        applyCachedImage(resolveInitialAsset(data.ps4?.cog), (value) => {
          storePrivate.ps4Cog = value;
        });
        applyCachedImage(resolveInitialAsset(data.ps4?.emblem), (value) => {
          storePrivate.ps4Emblem = value;
        });
        const activePs4HeaderImageUrl = headerUrlForVariant(activeHeaderVariant, {
          online: resolveInitialAsset(data.ps4?.headers?.online),
          forward: resolveInitialAsset(data.ps4?.headers?.forward),
          g: resolveInitialAsset(data.ps4?.headers?.g),
          z: resolveInitialAsset(data.ps4?.headers?.z),
          zz: resolveInitialAsset(data.ps4?.headers?.zz),
        });
        if (activePs4HeaderImageUrl) {
          applyCachedImage(activePs4HeaderImageUrl, (value) => {
            setPs4HeaderUrlForVariant(activeHeaderVariant, value);
          });
        }
      } else {
        applyCachedImage(resolveInitialAsset(data.background), (value) => {
          storePrivate.background = value;
        });
        applyCachedImage(resolveInitialAsset(data.cog), (value) => {
          storePrivate.cog = value;
        });
        applyCachedImage(resolveInitialAsset(data.capcom), (value) => {
          storePrivate.capcom = value;
        });
        applyCachedImage(resolveInitialAsset(data.button), (value) => {
          storePrivate.classicButton = value;
        });
        applyCachedImage(resolveInitialAsset(data.classicAddServerButton), (value) => {
          storePrivate.classicAddServerButton = value;
        });
        const activeClassicHeaderImageUrl = headerUrlForVariant(activeHeaderVariant, {
          online: resolveInitialAsset(data.headers?.online),
          forward: resolveInitialAsset(data.headers?.forward),
          g: resolveInitialAsset(data.headers?.g),
          z: resolveInitialAsset(data.headers?.z),
          zz: resolveInitialAsset(data.headers?.zz),
        });
        if (activeClassicHeaderImageUrl) {
          applyCachedImage(activeClassicHeaderImageUrl, (value) => {
            setClassicHeaderUrlForVariant(activeHeaderVariant, value);
          });
        }
      }
    }
  }
  await preloadImageListWithTimeout([
    backgroundUrl.value,
    launcherHeaderUrl.value,
    capcomUrl.value,
    cogUrl.value,
    storeMut.style === PS4_STYLE ? ps4ButtonUrl.value : classicButtonUrl.value,
    storeMut.style === PS4_STYLE
      ? ps4AddServerButtonUrl.value
      : classicAddServerButtonUrl.value,
    currentBanner.value?.src,
    dialogUrl.value,
    serverPatchUrl.value,
    ...styleSettingsAssetUrls(),
  ]);
  } finally {
    isHydratingStore = false;
  }
}

watch(
  () => storePrivate.settings.devMode,
  (enabled) => {
    if (!enabled && storeMut.style !== CLASSIC_STYLE && storeMut.style !== PS4_STYLE) {
      storeMut.style = CLASSIC_STYLE;
    }
  }
);

export function suspendLauncherResources() {
  if (storePrivate.launcherSuspended) return;
  storePrivate.launcherSuspended = true;
  storePrivate.banners = [];
  storePrivate.links = [];
  storePrivate.characters = [];
  storePrivate.friends = [];
  storePrivate.messages = [];
  storePrivate.remoteMessages = [];
  storePrivate.background = null;
  storePrivate.cog = null;
  storePrivate.capcom = null;
  storePrivate.classicButton = null;
  storePrivate.classicAddServerButton = null;
  storePrivate.dialogImage = null;
  storePrivate.serverPatchImage = null;
  storePrivate.classicHeaderOnline = null;
  storePrivate.classicHeaderForward = null;
  storePrivate.classicHeaderG = null;
  storePrivate.classicHeaderZ = null;
  storePrivate.classicHeaderZZ = null;
  storePrivate.ps4Background = null;
    storePrivate.ps4Button = null;
    storePrivate.ps4AddServerButton = null;
    storePrivate.ps4Capcom = null;
  storePrivate.ps4Cog = null;
  storePrivate.ps4Emblem = null;
  storePrivate.ps4HeaderOnline = null;
  storePrivate.ps4HeaderForward = null;
  storePrivate.ps4HeaderG = null;
  storePrivate.ps4HeaderZ = null;
  storePrivate.ps4HeaderZZ = null;
  bannerIndex.value = 0;
  clearImageCache();
  releaseUiResources();
  suspendUiWindow();
}

export async function resumeLauncherResources() {
  if (!storePrivate.launcherSuspended) return;
  storePrivate.launcherSuspended = false;
  await resumeUiWindow();
  await initStore();
}

export async function initRemoteEndpoints({ endpoints, remoteEndpoints }) {
  if (endpoints !== null) {
    storePrivate.endpoints = normalizeEndpointCollectionForUi(endpoints);
  }
  if (remoteEndpoints !== null) {
    storePrivate.remoteEndpoints = normalizeEndpointCollectionForUi(remoteEndpoints);
  }
}

export function closeDialog() {
  if (storePrivate.dialogKind === VERSION_SWITCH_DIALOG) {
    if (pendingVersionSignaturePrompt) {
      resolveVersionSignaturePrompt("none");
    } else {
      resolveVersionSwitchPrompt("stay");
    }
  }
  if (storePrivate.dialogKind === EXTERNAL_LINK_DIALOG) {
    resolveExternalLinkPrompt(false);
  }
  stopResetProgressAnimation();
  stopLinuxPrefixProgressAnimation();
  storePrivate.dialogOpen = false;
  storePrivate.dialogLoading = false;
  storePrivate.dialogError = "";
  storePrivate.dialogMessage = "";
  storePrivate.versionSignatureChoices = [];
  storePrivate.resetPatchCompleted = false;
  storePrivate.resetPatchProgress = 0;
  storePrivate.linuxPrefixInstallCompleted = false;
  storePrivate.linuxPrefixInstallProgress = 0;
  pendingServerSwap = null;
  pendingResetGameFolder = null;
}

async function hanldeDialogClose(cb) {
  storePrivate.dialogLoading = true;
  try {
    await cb();
    storePrivate.dialogOpen = false;
    storePrivate.dialogError = "";
    storePrivate.dialogMessage = "";
    storePrivate.dialogLoading = false;
  } catch (error) {
    if (error === "") return;
    storePrivate.dialogError = error;
    storePrivate.dialogLoading = false;
    throw error;
  }
}

let pendingServerSwap = null;
let pendingResetGameFolder = null;
let pendingExternalLink = null;
let pendingVersionSwitchPrompt = null;
let pendingVersionSignaturePrompt = null;
let lastEndpointKey = null;
let endpointAssetRequestId = 0;
let resetPatchProgressTimer = null;
let linuxPrefixProgressTimer = null;

function stopResetProgressAnimation() {
  if (resetPatchProgressTimer) {
    clearInterval(resetPatchProgressTimer);
    resetPatchProgressTimer = null;
  }
}

function stopLinuxPrefixProgressAnimation() {
  if (linuxPrefixProgressTimer) {
    clearInterval(linuxPrefixProgressTimer);
    linuxPrefixProgressTimer = null;
  }
}

function startResetProgressAnimation() {
  stopResetProgressAnimation();
  storePrivate.resetPatchProgress = 0.08;
  resetPatchProgressTimer = setInterval(() => {
    if (storePrivate.resetPatchProgress >= 0.92) return;
    storePrivate.resetPatchProgress = Math.min(
      0.92,
      storePrivate.resetPatchProgress + 0.07
    );
  }, 160);
}

function finishResetProgressAnimation() {
  stopResetProgressAnimation();
  const start = Math.max(0, Math.min(1, Number(storePrivate.resetPatchProgress) || 0));
  const steps = 8;
  let tick = 0;

  return new Promise((resolve) => {
    resetPatchProgressTimer = setInterval(() => {
      tick += 1;
      const eased = 1 - Math.pow(1 - tick / steps, 2);
      storePrivate.resetPatchProgress = Math.min(1, start + (1 - start) * eased);
      if (tick >= steps) {
        stopResetProgressAnimation();
        storePrivate.resetPatchProgress = 1;
        setTimeout(resolve, 220);
      }
    }, 42);
  });
}

function startLinuxPrefixProgressAnimation() {
  stopLinuxPrefixProgressAnimation();
  storePrivate.linuxPrefixInstallProgress = 0.08;
  linuxPrefixProgressTimer = setInterval(() => {
    if (storePrivate.linuxPrefixInstallProgress >= 0.92) return;
    storePrivate.linuxPrefixInstallProgress = Math.min(
      0.92,
      storePrivate.linuxPrefixInstallProgress + 0.07
    );
  }, 160);
}


function resolveExternalLinkPrompt(allowOpen) {
  if (!pendingExternalLink) return;
  pendingExternalLink.resolve(Boolean(allowOpen));
  pendingExternalLink = null;
}

function resolveVersionSwitchPrompt(choice) {
  if (!pendingVersionSwitchPrompt) return;
  pendingVersionSwitchPrompt.resolve(choice);
  pendingVersionSwitchPrompt = null;
}

function resolveVersionSignaturePrompt(signature) {
  if (!pendingVersionSignaturePrompt) return;
  pendingVersionSignaturePrompt.resolve(signature);
  pendingVersionSignaturePrompt = null;
}

function openVersionSwitchDialog(message) {
  resolveVersionSwitchPrompt("stay");
  resolveVersionSignaturePrompt("none");
  return new Promise((resolve) => {
    pendingVersionSwitchPrompt = { resolve };
    storePrivate.dialogError = "";
    storePrivate.dialogMessage = String(message ?? "");
    storePrivate.versionSignatureChoices = [];
    storePrivate.dialogKind = VERSION_SWITCH_DIALOG;
    storePrivate.dialogOpen = true;
  });
}

function versionSignaturePromptHtml(version) {
  return getMessage(
    "version-signature-selection-message",
    { version },
    `Select the client data signature for ${version}. This is used to populate launcher data like friends list/data. If you are not sure, choose <strong>I don't know</strong> to leave it disabled.`
  );
}

function openVersionSignatureDialog(version, signatures) {
  resolveVersionSignaturePrompt("none");
  const choices = signatures.map((signature) => ({
    value: signature,
    label: friendSignatureDisplayLabel(signature),
  }));

  return new Promise((resolve) => {
    pendingVersionSignaturePrompt = { resolve };
    storePrivate.dialogError = "";
    storePrivate.dialogMessage = versionSignaturePromptHtml(version);
    storePrivate.versionSignatureChoices = choices;
    storePrivate.dialogKind = VERSION_SWITCH_DIALOG;
    storePrivate.dialogOpen = true;
  });
}

async function resolveVersionSwitchSignature(targetVersion, versionInfo) {
  const hdVersion = storePrivate.settings.hdVersion;
  const signatureOptions = friendSignatureOptionsForVersion(targetVersion, hdVersion);
  if (signatureOptions.length > 1) {
    const choice = await openVersionSignatureDialog(targetVersion, signatureOptions).catch(
      () => "none"
    );
    return normalizeFriendSignature(targetVersion, choice, hdVersion);
  }
  if (signatureOptions.length === 1) {
    return normalizeFriendSignature(targetVersion, signatureOptions[0], hdVersion);
  }
  return resolveSignatureFromClientMode(targetVersion, versionInfo?.clientMode, hdVersion);
}

export function confirmExternalLinkOpen(url) {
  if (!url) return Promise.resolve(false);
  if (storePrivate.settings.externalLinkPromptDisabled) {
    return Promise.resolve(true);
  }
  resolveExternalLinkPrompt(false);
  return new Promise((resolve) => {
    pendingExternalLink = {
      url,
      resolve,
    };
    storePrivate.dialogError = "";
    storePrivate.dialogMessage = String(url);
    storePrivate.dialogKind = EXTERNAL_LINK_DIALOG;
    storePrivate.dialogOpen = true;
  });
}

export function dialogCancelExternalLink() {
  resolveExternalLinkPrompt(false);
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
}

export function dialogConfirmExternalLink() {
  resolveExternalLinkPrompt(true);
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
}

export function dialogConfirmExternalLinkDontShowAgain() {
  storePrivate.settings.externalLinkPromptDisabled = true;
  saveUiPrefs();
  resolveExternalLinkPrompt(true);
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
}

export function dialogVersionSwitchStay() {
  resolveVersionSwitchPrompt("stay");
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
  storePrivate.versionSignatureChoices = [];
}

export function dialogVersionSwitchYes() {
  resolveVersionSwitchPrompt("yes");
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
  storePrivate.versionSignatureChoices = [];
}

export function dialogVersionSwitchDontAskAgain() {
  setUiPref("serverVersionPromptDisabled", true);
  resolveVersionSwitchPrompt("stay");
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
  storePrivate.versionSignatureChoices = [];
}

export function dialogVersionSignatureSelect(signature) {
  const normalized = String(signature ?? "").trim() || "none";
  resolveVersionSignaturePrompt(normalized);
  storePrivate.dialogOpen = false;
  storePrivate.dialogMessage = "";
  storePrivate.versionSignatureChoices = [];
}

function linuxPrefixInstallSummaryHtml() {
  return getMessage(
    "linux-prefix-install-confirmation",
    null,
    "Set up the portable Mezeporta Wine prefix for this install?<br><br>This will verify wine, wineserver, and winetricks, create or reuse <strong>Mezeporta/WinePrefix</strong>, run <strong>wineboot -u</strong>, install <strong>d3dcompiler_47</strong>, <strong>dxvk</strong>, and <strong>vcrun2022</strong>, and apply the Linux controller DLL overrides if R-Analog Patch is enabled."
  );
}

function linuxPrefixInstallProgressHtml() {
  return getMessage(
    "linux-prefix-install-progress",
    null,
    "Installing the portable Mezeporta Wine prefix for this game folder..."
  );
}

function normalizeLinuxPrefixInstallError(error) {
  const text = String(error ?? "");
  const missingToolsPrefix = "linux-prefix-missing-tools:";
  if (!text.startsWith(missingToolsPrefix)) {
    return text;
  }

  const tools = text
    .slice(missingToolsPrefix.length)
    .split(",")
    .map((entry) => String(entry).trim())
    .filter(Boolean)
    .join(", ");

  return getMessage(
    "linux-prefix-install-missing-tools",
    { tools },
    `Missing Linux runtime tools: ${tools}. Run the bundled mezeporta-setup-ubuntu.sh or mezeporta-setup-arch.sh script first.`
  );
}

export async function refreshLinuxPrefixStatus() {
  storePrivate.linuxPrefixStatus.loading = true;
  storePrivate.linuxPrefixStatus.error = "";
  try {
    const status = await handleInvoke("get_linux_prefix_status");
    storePrivate.linuxPrefixStatus.ready = Boolean(status?.ready);
    storePrivate.linuxPrefixStatus.missingTools = Array.isArray(status?.missingTools)
      ? status.missingTools.map((entry) => String(entry))
      : [];
    storePrivate.linuxPrefixStatus.prefixPath = String(status?.prefixPath ?? "");
    storePrivate.linuxPrefixStatus.error = String(status?.error ?? "");
    storePrivate.linuxPrefixStatus.audioReady =
      status?.audioReady === undefined ? true : Boolean(status?.audioReady);
    storePrivate.linuxPrefixStatus.audioMissing = Array.isArray(status?.audioMissing)
      ? status.audioMissing.map((entry) => String(entry))
      : [];
  } catch (_error) {
    storePrivate.linuxPrefixStatus.ready = false;
    storePrivate.linuxPrefixStatus.missingTools = [];
    storePrivate.linuxPrefixStatus.prefixPath = "";
    storePrivate.linuxPrefixStatus.error = "";
    storePrivate.linuxPrefixStatus.audioReady = true;
    storePrivate.linuxPrefixStatus.audioMissing = [];
  } finally {
    storePrivate.linuxPrefixStatus.loading = false;
  }
}

export function dialogOpenLinuxPrefixInstall() {
  stopLinuxPrefixProgressAnimation();
  storePrivate.dialogError = "";
  storePrivate.dialogLoading = false;
  storePrivate.dialogMessage = linuxPrefixInstallSummaryHtml();
  storePrivate.linuxPrefixInstallCompleted = false;
  storePrivate.linuxPrefixInstallProgress = 0;
  storePrivate.dialogKind = LINUX_PREFIX_DIALOG;
  storePrivate.dialogOpen = true;
}

export async function dialogConfirmLinuxPrefixInstall() {
  if (storePrivate.linuxPrefixInstallCompleted) {
    closeDialog();
    return;
  }

  storePrivate.dialogLoading = true;
  storePrivate.dialogError = "";
  storePrivate.linuxPrefixInstallProgress = 0;
  storePrivate.dialogMessage = linuxPrefixInstallProgressHtml();
  startLinuxPrefixProgressAnimation();
  try {
    const status = await handleInvoke("install_linux_portable_prefix");
    stopLinuxPrefixProgressAnimation();
    storePrivate.linuxPrefixStatus.ready = Boolean(status?.ready);
    storePrivate.linuxPrefixStatus.missingTools = Array.isArray(status?.missingTools)
      ? status.missingTools.map((entry) => String(entry))
      : [];
    storePrivate.linuxPrefixStatus.prefixPath = String(status?.prefixPath ?? "");
    storePrivate.linuxPrefixStatus.error = String(status?.error ?? "");
    storePrivate.linuxPrefixStatus.audioReady =
      status?.audioReady === undefined ? true : Boolean(status?.audioReady);
    storePrivate.linuxPrefixStatus.audioMissing = Array.isArray(status?.audioMissing)
      ? status.audioMissing.map((entry) => String(entry))
      : [];
    storePrivate.linuxPrefixInstallCompleted = true;
    storePrivate.linuxPrefixInstallProgress = 1;
    storePrivate.dialogLoading = false;
    storePrivate.dialogMessage = getMessage(
      "linux-prefix-install-success",
      null,
      "Portable Mezeporta Wine prefix is ready."
    );
  } catch (error) {
    if (error === "") return;
    stopLinuxPrefixProgressAnimation();
    storePrivate.linuxPrefixInstallProgress = 0;
    storePrivate.dialogError = normalizeLinuxPrefixInstallError(error);
    storePrivate.dialogLoading = false;
  }
}

const dialogCallbackMap = {
  [DELETE_DIALOG]: dialogDeleteCharacterConfirm,
  [SERVERS_DIALOG]: dialogSaveEndpoint,
  [PATCHER_DIALOG]: dialogStartPatcher,
  [SERVER_SWITCH_DIALOG]: dialogConfirmServerSwitch,
  [EXTERNAL_LINK_DIALOG]: dialogConfirmExternalLink,
  [RESET_PATCH_DIALOG]: dialogConfirmResetPatch,
  [LINUX_PREFIX_DIALOG]: dialogConfirmLinuxPrefixInstall,
  [BAN_DIALOG]: closeDialog,
};
export function dialogCallback() {
  dialogCallbackMap[storePrivate.dialogKind]();
}


// Dialog server edit/add
let editEndpointIndex = 0;
let editEndpointRemote = false;
export function dialogAddEndpoint() {
  editEndpointIndex = store.endpoints.length;
  editEndpointRemote = false;
  storeMut.editEndpoint = {
    name: "",
    host: "",
    launcherPort: null,
    gamePort: null,
    gamePath: null,
    version: DEFAULT_GAME_VERSION,
  };
  storePrivate.editEndpointNew = true;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export function dialogEditEndpoint(index, remote) {
  editEndpointIndex = index;
  editEndpointRemote = remote;
  let endpoints = remote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  storeMut.editEndpoint = {
    ...endpoints[index],
  };
  storePrivate.editEndpointNew = false;
  storePrivate.dialogKind = SERVERS_DIALOG;
  storePrivate.dialogOpen = true;
}
export async function dialogRemoveEndpoint() {
  let endpoints = editEndpointRemote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  endpoints = [...endpoints];
  endpoints.splice(editEndpointIndex, 1);
  setEndpoints(endpoints, editEndpointRemote);
  storePrivate.dialogError = "";
  storePrivate.dialogOpen = false;
}
export async function dialogSaveEndpoint() {
  if (storePrivate.dialogLoading) return;
  let endpoints = editEndpointRemote
    ? storePrivate.remoteEndpoints
    : storePrivate.endpoints;
  endpoints = [...endpoints];
  const newEndpoint = { ...storeMut.editEndpoint };
  if (!newEndpoint.isRemote) {
    newEndpoint.launcherPort = normalizeDialogPort(
      newEndpoint.launcherPort,
      DEFAULT_LAUNCHER_PORT
    );
    newEndpoint.gamePort = normalizeDialogPort(
      newEndpoint.gamePort,
      DEFAULT_GAME_PORT
    );
  }
  endpoints[editEndpointIndex] = newEndpoint;
  await hanldeDialogClose(
    async () => {
      await setEndpoints(endpoints, editEndpointRemote);
      if (!editEndpointRemote && editEndpointIndex === endpoints.length - 1) {
        await setCurrentEndpoint(newEndpoint);
      } else if (
        editEndpointRemote &&
        editEndpointIndex === storePrivate.remoteEndpoints.length - 1
      ) {
        await setCurrentEndpoint(newEndpoint);
      }
    }
  );
}

export function dialogDeleteCharacter(character) {
  storePrivate.deleteCharacter = character;
  storePrivate.dialogKind = DELETE_DIALOG;
  storePrivate.dialogOpen = true;
}
export async function dialogDeleteCharacterConfirm() {
  await hanldeDialogClose(
    async () => await doDeleteCharacter(storePrivate.deleteCharacter.id)
  );
}

export async function dialogStartPatcher() {
  storePrivate.authLoading = true;
  await hanldeDialogClose(async () => {
    await handleInvoke("patcher_start");
    storeMut.page = PATCHER_PAGE;
  });
}
export async function completePatcher() {
  storePrivate.authLoading = false;
  if (storePrivate.skipCharacterLoadingForPatcher) {
    storePrivate.skipCharacterLoadingForPatcher = false;
    storePrivate.unitCardLoadCycle = 0;
    storePrivate.characterRevealCycle += 1;
  }
  storeMut.page = CHARACTERS_PAGE;
}
export async function cancelPatcher() {
  await handleInvoke("patcher_stop");
  storePrivate.authLoading = false;
  storeMut.page = LOGIN_PAGE;
}

export async function dialogConfirmServerSwitch() {
  const swapTarget = pendingServerSwap?.targetServer;
  const swapToCached = pendingServerSwap?.swapToCached;
  pendingServerSwap = null;
  await hanldeDialogClose(async () => {
    if (!swapTarget) return;
    if (swapToCached) {
      await handleInvoke("patcher_swap_to_cached", {
        targetServer: swapTarget,
      });
    } else {
      await handleInvoke("reset_game_files", {
        gameFolder: effectiveFolder.value,
      });
    }
    await doAuth("login");
  });
}


export function dialogOpenResetPatch(gameFolder) {
  pendingResetGameFolder = gameFolder || effectiveFolder.value;
  stopResetProgressAnimation();
  storePrivate.dialogError = "";
  storePrivate.dialogMessage = getMessage(
    "reset-patch-confirmation",
    null,
    "Restore all patched files back to original for this game folder?"
  );
  storePrivate.resetPatchCompleted = false;
  storePrivate.resetPatchProgress = 0;
  storePrivate.dialogLoading = false;
  storePrivate.dialogKind = RESET_PATCH_DIALOG;
  storePrivate.dialogOpen = true;
}

export async function dialogConfirmResetPatch() {
  if (storePrivate.resetPatchCompleted) {
    closeDialog();
    return;
  }

  const gameFolder = pendingResetGameFolder || effectiveFolder.value;
  if (!gameFolder) {
    storePrivate.dialogError = "path-folder-error";
    return;
  }

  storePrivate.dialogLoading = true;
  storePrivate.dialogError = "";
  startResetProgressAnimation();

  try {
    await handleInvoke("reset_game_files", {
      gameFolder,
    });
    await finishResetProgressAnimation();
    storePrivate.dialogLoading = false;
    storePrivate.resetPatchCompleted = true;
    storePrivate.dialogMessage = getMessage(
      "reset-patch-success-confirmation",
      null,
      "Patched files were reset successfully."
    );
  } catch (error) {
    if (error === "") return;
    stopResetProgressAnimation();
    storePrivate.dialogLoading = false;
    storePrivate.resetPatchProgress = 0;
    storePrivate.dialogError = error;
  }
}

export function setSetting(setting, value) {
  storePrivate.settings[setting] = value;
  scheduleSettingWrite(setting, value);
}

export function setLauncherPrefs(prefs) {
  const payload = resolveLauncherPrefs({
    ...prefs,
    gameVersion: prefs?.gameVersion ?? storePrivate.settings.gameVersion,
    hdVersion: prefs?.hdVersion ?? storePrivate.settings.hdVersion,
  });
  if (prefs.preloadControllerDlls !== undefined) {
    storePrivate.settings.preloadControllerDlls = prefs.preloadControllerDlls;
  }
  if (prefs.friendSignature !== undefined) {
    storePrivate.settings.friendSignature = payload.friendSignature;
  }
  if (prefs.winePrefixMode !== undefined) {
    storePrivate.settings.winePrefixMode = payload.winePrefixMode;
  }
  if (prefs.winePrefixCustomPath !== undefined) {
    storePrivate.settings.winePrefixCustomPath = payload.winePrefixCustomPath;
  }
  launcherPrefSyncPromise = Promise.resolve(setLauncherPreference(payload)).catch(() => undefined);
  scheduleLauncherPrefWrite(payload);
  return launcherPrefSyncPromise;
}

export function getStoredLauncherRecentResolutions(style = storeMut.style) {
  return getLauncherRecentResolutionsForStyle(style);
}

export function rememberLauncherResolution(style, size) {
  const keys = getLauncherResolutionPrefKeys(style);
  const recentKey = getLauncherRecentResolutionPrefKey(style);
  if (!keys || !recentKey) return [];

  const width = normalizeLauncherDimension(
    size?.width,
    getLauncherWindowDefaults(style).width
  );
  const height = normalizeLauncherDimension(
    size?.height,
    getLauncherWindowDefaults(style).height
  );

  storePrivate.settings[keys.width] = width;
  storePrivate.settings[keys.height] = height;
  const recentList = promoteLauncherRecentResolution(style, width, height);
  saveUiPrefs();
  setUiPref(keys.width, width);
  setUiPref(keys.height, height);
  setUiPref(recentKey, recentList);
  return recentList;
}

export function getStoredLauncherResolution(style = storeMut.style) {
  const keys = getLauncherResolutionPrefKeys(style);
  const defaults = getLauncherWindowDefaults(style);
  if (!keys) {
    return { width: defaults.width, height: defaults.height };
  }
  return {
    width: normalizeLauncherDimension(
      storePrivate.settings[keys.width],
      defaults.width
    ),
    height: normalizeLauncherDimension(
      storePrivate.settings[keys.height],
      defaults.height
    ),
  };
}

export async function setEndpoints(endpoints, remote) {
  endpoints = endpoints.map((endpoint) => normalizeEndpointForBackend(endpoint));
  let currentEndpoint;
  if (remote) {
    currentEndpoint = await handleInvoke("set_remote_endpoints", {
      endpoints,
    });
    storePrivate.remoteEndpoints = endpoints;
  } else {
    currentEndpoint = await handleInvoke("set_endpoints", {
      endpoints,
    });
    storePrivate.endpoints = endpoints;
  }
  if (currentEndpoint !== storePrivate.currentEndpoint) {
    setCurrentEndpoint(currentEndpoint);
  }
}
export async function setCurrentEndpoint(currentEndpoint, options = {}) {
  return setCurrentEndpointWithOptions(currentEndpoint, options);
}

async function setCurrentEndpointWithOptions(currentEndpoint, options = {}) {
  const showLoading = options.showLoading !== false;
  const endpointWithVersion = normalizeEndpointForBackend({
    ...currentEndpoint,
    version: storePrivate.settings.gameVersion,
  });
  const activeHeaderVariant = headerVariantKeyForVersion(storePrivate.settings.gameVersion);
  const nextEndpointKey = [
    getEndpointServerKey(endpointWithVersion),
    storeMut.style,
    activeHeaderVariant,
  ].join("|");
  if (
    endpointKey(endpointWithVersion) === endpointKey(storePrivate.currentEndpoint) &&
    nextEndpointKey === lastEndpointKey
  ) {
    return;
  }

  storePrivate.currentEndpoint = endpointWithVersion;
  setBannerIndex(0);

  const requestId = ++endpointAssetRequestId;
  const isCurrentRequest = () => requestId === endpointAssetRequestId;

  storePrivate.serverVersionInfo = null;

  if (!endpointWithVersion?.url || endpointWithVersion.url === "OFFLINEMODE") {
    if (showLoading) {
      storePrivate.launcherAssetsLoading = true;
    }
    applyOfflineFallbackUi();
    lastEndpointKey = nextEndpointKey;
    await preloadImageListWithTimeout([
      backgroundUrl.value,
      launcherHeaderUrl.value,
      capcomUrl.value,
      cogUrl.value,
      storeMut.style === PS4_STYLE ? ps4ButtonUrl.value : classicButtonUrl.value,
      storeMut.style === PS4_STYLE
        ? ps4AddServerButtonUrl.value
        : classicAddServerButtonUrl.value,
      dialogUrl.value,
      serverPatchUrl.value,
      ...styleSettingsAssetUrls(),
    ]);
    if (isCurrentRequest() && showLoading) {
      storePrivate.launcherAssetsLoading = false;
    }
    return;
  }

  if (showLoading) {
    storePrivate.launcherAssetsLoading = true;
  }
  try {
    let data = await handleInvoke(
      "set_current_endpoint",
      { currentEndpoint: endpointWithVersion },
      "warning"
    );
    if (!isCurrentRequest()) return;

    lastEndpointKey = nextEndpointKey;
    const resolveAsset = (path) => resolveEndpointAsset(path, endpointWithVersion);
    const applyEndpointImage = (url, assign) => {
      if (!url) {
        if (!isCurrentRequest()) return;
        assign(null);
        return;
      }
      applyCachedImage(url, (value) => {
        if (!isCurrentRequest()) return;
        assign(value);
      });
    };
    const backgroundImageUrl = resolveAsset(data.background);
    const cogImageUrl = resolveAsset(data.cog);
    const capcomImageUrl = resolveAsset(data.capcom);
    const classicButtonImageUrl = resolveAsset(data.button);
    const classicAddServerButtonImageUrl = resolveAsset(data.classicAddServerButton);
    const dialogImageUrl = resolveAsset(data.dialog ?? data.dialogue);
    const serverPatchImageUrl = resolveAsset(data.serverPatch ?? data.server_patch);
    const classicHeaderOnlineImageUrl = resolveAsset(data.headers?.online);
    const classicHeaderForwardImageUrl = resolveAsset(data.headers?.forward);
    const classicHeaderGImageUrl = resolveAsset(data.headers?.g);
    const classicHeaderZImageUrl = resolveAsset(data.headers?.z);
    const classicHeaderZZImageUrl = resolveAsset(data.headers?.zz);
    const ps4BackgroundImageUrl = resolveAsset(data.ps4?.background);
    const ps4ButtonImageUrl = resolveAsset(data.ps4?.button);
    const ps4AddServerButtonImageUrl = resolveAsset(data.ps4?.addServerButton);
    const ps4CapcomImageUrl = resolveAsset(data.ps4?.capcom);
    const ps4CogImageUrl = resolveAsset(data.ps4?.cog);
    const ps4EmblemImageUrl = resolveAsset(data.ps4?.emblem);
    const ps4HeaderOnlineImageUrl = resolveAsset(data.ps4?.headers?.online);
    const ps4HeaderForwardImageUrl = resolveAsset(data.ps4?.headers?.forward);
    const ps4HeaderGImageUrl = resolveAsset(data.ps4?.headers?.g);
    const ps4HeaderZImageUrl = resolveAsset(data.ps4?.headers?.z);
    const ps4HeaderZZImageUrl = resolveAsset(data.ps4?.headers?.zz);
    const activeClassicHeaderImageUrl = headerUrlForVariant(activeHeaderVariant, {
      online: classicHeaderOnlineImageUrl,
      forward: classicHeaderForwardImageUrl,
      g: classicHeaderGImageUrl,
      z: classicHeaderZImageUrl,
      zz: classicHeaderZZImageUrl,
    });
    const activePs4HeaderImageUrl = headerUrlForVariant(activeHeaderVariant, {
      online: ps4HeaderOnlineImageUrl,
      forward: ps4HeaderForwardImageUrl,
      g: ps4HeaderGImageUrl,
      z: ps4HeaderZImageUrl,
      zz: ps4HeaderZZImageUrl,
    });
    const allowRemoteLauncherImages = !shouldUseOfflineImages();

    storePrivate.banners = applyCachedImageList(data.banners, "src", (index, value) => {
      if (!isCurrentRequest()) return;
      storePrivate.banners = storePrivate.banners.map((item, i) =>
        i === index ? { ...item, src: value } : item
      );
    }, resolveAsset, { refreshLimit: 1 });
    storePrivate.messages = data.messages;
    storePrivate.launcherTag = normalizeLauncherTag(launcherTagFromPayload(data));
    storePrivate.links = applyCachedImageList(data.links, "icon", (index, value) => {
      if (!isCurrentRequest()) return;
      storePrivate.links = storePrivate.links.map((item, i) =>
        i === index ? { ...item, icon: value } : item
      );
    }, resolveAsset);
    if (allowRemoteLauncherImages) {
      clearLauncherImageState();
      applyEndpointImage(dialogImageUrl, (value) => {
        storePrivate.dialogImage = value;
      });
      applyEndpointImage(serverPatchImageUrl, (value) => {
        storePrivate.serverPatchImage = value;
      });
      if (storeMut.style === PS4_STYLE) {
        applyEndpointImage(ps4BackgroundImageUrl, (value) => {
          storePrivate.ps4Background = value;
        });
        applyEndpointImage(ps4ButtonImageUrl, (value) => {
          storePrivate.ps4Button = value;
        });
        applyEndpointImage(ps4AddServerButtonImageUrl, (value) => {
          storePrivate.ps4AddServerButton = value;
        });
        applyEndpointImage(ps4CapcomImageUrl, (value) => {
          storePrivate.ps4Capcom = value;
        });
        applyEndpointImage(ps4CogImageUrl, (value) => {
          storePrivate.ps4Cog = value;
        });
        applyEndpointImage(ps4EmblemImageUrl, (value) => {
          storePrivate.ps4Emblem = value;
        });
        if (activePs4HeaderImageUrl) {
          applyEndpointImage(activePs4HeaderImageUrl, (value) => {
            setPs4HeaderUrlForVariant(activeHeaderVariant, value);
          });
        }
      } else {
        applyEndpointImage(backgroundImageUrl, (value) => {
          storePrivate.background = value;
        });
        applyEndpointImage(cogImageUrl, (value) => {
          storePrivate.cog = value;
        });
        applyEndpointImage(capcomImageUrl, (value) => {
          storePrivate.capcom = value;
        });
        applyEndpointImage(classicButtonImageUrl, (value) => {
          storePrivate.classicButton = value;
        });
        applyEndpointImage(classicAddServerButtonImageUrl, (value) => {
          storePrivate.classicAddServerButton = value;
        });
        if (activeClassicHeaderImageUrl) {
          applyEndpointImage(activeClassicHeaderImageUrl, (value) => {
            setClassicHeaderUrlForVariant(activeHeaderVariant, value);
          });
        }
      }
    } else {
      clearLauncherImageState();
    }

    const firstBannerRawUrl = resolveAsset(data.banners?.[0]?.src);
    if (firstBannerRawUrl) {
      const freshFirstBannerUrl = await preloadFreshImage(firstBannerRawUrl);
      if (!isCurrentRequest()) return;
      if (freshFirstBannerUrl) {
        storePrivate.banners = storePrivate.banners.map((item, index) =>
          index === 0 ? { ...item, src: freshFirstBannerUrl } : item
        );
      }
    } else if (storePrivate.banners[0]?.src) {
      await preloadImageWithTimeout(storePrivate.banners[0].src);
      if (!isCurrentRequest()) return;
    }

    if (allowRemoteLauncherImages) {
      const essentialAssets =
        storeMut.style === PS4_STYLE
          ? [
              ps4BackgroundImageUrl,
              ps4CapcomImageUrl,
              ps4CogImageUrl,
              ps4EmblemImageUrl,
              activePs4HeaderImageUrl,
              ps4ButtonImageUrl,
              ps4AddServerButtonImageUrl,
              dialogImageUrl,
              serverPatchImageUrl,
            ].filter(Boolean)
          : storeMut.style === CLASSIC_STYLE
          ? [
              backgroundImageUrl,
              cogImageUrl,
              capcomImageUrl,
              classicButtonImageUrl,
              classicAddServerButtonImageUrl,
              activeClassicHeaderImageUrl,
              dialogImageUrl,
              serverPatchImageUrl,
            ].filter(Boolean)
          : [backgroundImageUrl, cogImageUrl, capcomImageUrl, dialogImageUrl, serverPatchImageUrl].filter(Boolean);
      await preloadFreshImageList(essentialAssets);
      if (!isCurrentRequest()) return;
    }

    await preloadImageListWithTimeout([
      backgroundUrl.value,
      launcherHeaderUrl.value,
      capcomUrl.value,
      cogUrl.value,
      storeMut.style === PS4_STYLE ? ps4ButtonUrl.value : classicButtonUrl.value,
      storeMut.style === PS4_STYLE
        ? ps4AddServerButtonUrl.value
        : classicAddServerButtonUrl.value,
      storePrivate.banners[0]?.src,
      dialogUrl.value,
      serverPatchUrl.value,
      ...styleSettingsAssetUrls(),
    ]);
    if (!isCurrentRequest()) return;

    void fetchEndpointVersionInfo(endpointWithVersion).then((versionInfo) => {
      if (!isCurrentRequest()) return;
      storePrivate.serverVersionInfo = versionInfo;
    });
  } catch (_error) {
    if (!isCurrentRequest()) return;
    if (showLoading) {
      applyOfflineFallbackUi();
    }
    lastEndpointKey = null;
  } finally {
    if (isCurrentRequest() && showLoading) {
      storePrivate.launcherAssetsLoading = false;
    }
  }
}

export async function closeLauncher() {
  const forceClose = () => appWindow.close().catch(() => {});
  let shutdownHandled = false;
  try {
    await Promise.race([
      handleInvoke("shutdown_launcher").then(() => {
        shutdownHandled = true;
      }),
      new Promise((resolve) => setTimeout(resolve, 1500)),
    ]);
  } catch (_error) {
    // ignore and fall through to force close
  }
  if (!shutdownHandled) {
    await forceClose();
  }
}

export function addPlaceholderCharacter() {
  const hasPlaceholder = storePrivate.characters.some(
    (c) => c.id === null || c.placeholder
  );
  if (!hasPlaceholder) {
    storePrivate.characters.push({
      id: null,
      name: "",
      isFemale: false,
      weapon: 0,
      hr: 0,
      gr: 0,
      lastLogin: 0,
      placeholder: true,
    });
  }
}

// Invoke actions
function isBanMessage(error) {
  if (typeof error !== "string") return false;
  return /\bbanned\b/i.test(error);
}

function openBanDialog(message) {
  storePrivate.dialogError = "";
  storePrivate.dialogMessage = message;
  storePrivate.dialogKind = BAN_DIALOG;
  storePrivate.dialogOpen = true;
}

function authCredentialsSnapshot() {
  return {
    username: String(storeMut.username ?? ""),
    password: String(storeMut.password ?? ""),
    rememberMe: Boolean(storeMut.rememberMe),
  };
}

async function doAuth(kind, message, credentials = authCredentialsSnapshot()) {
  if (storePrivate.authLoading || storePrivate.characterLoading) return;
  storePrivate.authLoading = true;
  try {
    const { response, hasPatch } = await handleInvoke(
      kind,
      {
        username: credentials.username,
        password: credentials.password,
        rememberMe: credentials.rememberMe,
      },
      message
    );
    if (kind === "login" && !hasPatch) {
      storePrivate.unitCardLoadCycle += 1;
    }
    // If the backend returns no characters, or a single dummy entry with an empty name,
    // inject a placeholder object with weapon 0 and id null.
    const chars = Array.isArray(response.characters)
      ? response.characters
      : [];
    let normalised;
    if (
      chars.length === 0 ||
      (chars.length === 1 && (!chars[0] || !chars[0].name))
    ) {
      normalised = [
        {
          id: null,
          name: "",
          isFemale: false,
          weapon: 0,
          hr: 0,
          gr: 0,
          lastLogin: 0,
          placeholder: true,
        },
      ];
    } else {
      normalised = chars;
    }
    storePrivate.characters = normalised;
    storePrivate.friends = Array.isArray(response.friends)
      ? response.friends
          .map((friend) => ({
            cid: Number(friend?.cid ?? 0),
            id: Number(friend?.id ?? 0),
            name: String(friend?.name ?? "").trim(),
          }))
          .filter(
            (friend) =>
              Number.isFinite(friend.cid) &&
              friend.cid > 0 &&
              Number.isFinite(friend.id) &&
              friend.id > 0
          )
      : [];
    if (hasPatch) {
      if (kind === "login") {
        storePrivate.skipCharacterLoadingForPatcher = true;
      }
      storePrivate.dialogKind = PATCHER_DIALOG;
      storePrivate.dialogOpen = true;
    } else {
      storePrivate.skipCharacterLoadingForPatcher = false;
      storeMut.page = CHARACTERS_PAGE;
    }
  } catch (error) {
    if (isBanMessage(error)) {
      openBanDialog(error);
      return;
    }
    throw error;
  } finally {
    storePrivate.authLoading = false;
  }
}
export async function doLogin() {
  if (storePrivate.authLoading || storePrivate.characterLoading) return;
  const credentials = authCredentialsSnapshot();
  storePrivate.authLoading = true;
  try {
    await Promise.all([launcherPrefSyncPromise, uiPrefSyncPromise]);

    const endpoint = storePrivate.currentEndpoint;
    const canCheckVersion =
      endpoint &&
      endpoint.url &&
      endpoint.url !== "OFFLINEMODE";

    if (canCheckVersion) {
      const versionInfo = await ensureServerVersionInfoForLogin(endpoint);
      const targetVersion = versionInfo?.mappedGameVersion;
      const selectedVersion = normalizeGameVersion(storePrivate.settings.gameVersion);
      const ignoreServerVersionForSelected = selectedVersion === "Z2T";

      if (
        targetVersion &&
        targetVersion !== selectedVersion &&
          !ignoreServerVersionForSelected
        ) {
          const prompt = versionMismatchPrompt(versionInfo, selectedVersion);
          if (prompt && !storePrivate.settings.serverVersionPromptDisabled) {
            const action = await openVersionSwitchDialog(prompt).catch(() => "stay");
            if (action === "yes") {
              setGameVersion(targetVersion);

              const nextSignature = await resolveVersionSwitchSignature(
                targetVersion,
                versionInfo
              );
              setLauncherPrefs({
                friendSignature: nextSignature,
                gameVersion: targetVersion,
                hdVersion: storePrivate.settings.hdVersion,
              });

              await Promise.all([launcherPrefSyncPromise, uiPrefSyncPromise]);
            }
          }
        }
      }
  } finally {
    storePrivate.authLoading = false;
  }

  await doAuth("login", undefined, credentials);
}
export async function doRegister() {
  const credentials = authCredentialsSnapshot();
  await doAuth("register", undefined, credentials);
}
export async function doCreateCharacter() {
  if (storePrivate.characterLoading || storePrivate.authLoading) return;
  storePrivate.characterLoading = true;
  try {
    await Promise.all([launcherPrefSyncPromise, uiPrefSyncPromise]);
    await handleInvoke("create_character");
  } catch (error) {
    if (error !== "") {
      logText("error", String(error));
    }
  } finally {
    storePrivate.characterLoading = false;
  }
}
function getSavedataVersionTokenForLaunch() {
  const selectedVersion = normalizeGameVersion(storePrivate.settings.gameVersion);
  if (["G5", "Z2", "Z2T"].includes(selectedVersion)) {
    return selectedVersion;
  }
  const token = String(storePrivate.serverVersionInfo?.clientMode ?? "").trim();
  return token;
}
export function currentSavedataVersionToken() {
  return getSavedataVersionTokenForLaunch();
}
export async function doSelectCharacter(characterId) {
  if (storePrivate.characterLoading || storePrivate.authLoading) return;
  storePrivate.characterLoading = true;
  try {
    await Promise.all([launcherPrefSyncPromise, uiPrefSyncPromise]);
    await handleInvoke("select_character", {
      characterId,
      friendSignature: storePrivate.settings.friendSignature,
      savedataVersion: getSavedataVersionTokenForLaunch(),
    });
  } catch (error) {
    if (error !== "") {
      logText("error", String(error));
    }
  } finally {
    storePrivate.characterLoading = false;
  }
}
export async function doDeleteCharacter(characterId) {
  storePrivate.characterLoading = true;
  try {
    await handleInvoke("delete_character", { characterId });
    storePrivate.characters = storePrivate.characters.filter(
      (c) => c.id !== characterId
    );
  } finally {
    storePrivate.characterLoading = false;
  }
}
export async function doExportCharacter(characterId) {
  storePrivate.characterLoading = true;
  try {
    const location = await handleInvoke("export_character", { characterId });
    logMessage("info", "export-character-success", { location });
  } finally {
    storePrivate.characterLoading = false;
  }
}











































































