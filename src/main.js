import "./style.css";

import { createApp, nextTick, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { exists } from "@tauri-apps/api/fs";
import { platform } from "@tauri-apps/api/os";
import { join } from "@tauri-apps/api/path";
import { appWindow, PhysicalSize } from "@tauri-apps/api/window";

import Main from "./Main.vue";
import { fluentVue } from "./fluent";
import {
  assetUrl,
  backgroundUrl,
  classicButtonUrl,
  dialogUrl,
  effectiveFolder,
  launcherHeaderUrl,
  ps4ButtonUrl,
  refreshLinuxPrefixStatus,
  rememberLauncherResolution,
  serverPatchUrl,
  setUiPref,
  setDevFriendTestEntries,
  store,
  storeMut,
} from "./store.js";
import { setDevAltClientStatsOverride } from "./altClientStats";
import { preloadSfx, setLinuxAudioRuntimeStatus } from "./sfx";
import {
  CLASSIC_STYLE,
  PS4_STYLE,
  getLauncherResolutionPrefKeys,
  getLauncherWindowAspect,
  getLauncherWindowDefaults,
  isLauncherWindowResizable,
} from "./common";

const commonClassicLikeAssetNames = [
  "Settings.png",
  "SettingsHover.png",
  "minimize.png",
  "close.png",
  "msg-line-important.png",
  "msg-line-base.png",
  "new.gif",
  "bar.jpg",
  "bar_frame.png",
  "btn-blue.png",
  "btn-generic.png",
  "capcom.png",
  "checkbox.png",
  "cog.png",
  "icon-inquiry.png",
  "progress.png",
];

const classicAssetNames = [
  "btn-blue.png",
  "launcher-header.png",
];

const ps4AssetNames = [
  "Button.png",
  "Emblem.png",
];

const bannerAssets = [
  "/banners/BannerShow1.png",
  "/banners/BannerWelcome.png",
];

const flagAssets = [
  "/flags/en.svg",
  "/flags/jp.svg",
];

const sharedDialogAssets = [
  "/extra/dialog.png",
  "/extra/ServerPatch.png",
];

const DEFERRED_STARTUP_ASSETS = [
  ...bannerAssets,
  ...flagAssets,
];

const preloadedImages = new Set();

function preloadImages(assets) {
  for (const src of assets) {
    const resolved = assetUrl(src);
    if (preloadedImages.has(resolved)) continue;
    const image = new Image();
    image.src = resolved;
    preloadedImages.add(resolved);
  }
}

function deferPreload(assets) {
  const run = () => preloadImages(assets);
  if (typeof requestIdleCallback === "function") {
    requestIdleCallback(run);
  } else {
    setTimeout(run, 0);
  }
}

function classicLikeAssetFolder() {
  if (storeMut.style === PS4_STYLE) return "/ps4";
  return "/classic";
}

function styleAssetPath(fileName) {
  return `${classicLikeAssetFolder()}/${fileName}`;
}

function syncStyleAssetCssVars() {
  document.documentElement.style.setProperty(
    "--settings-btn-image",
    `url('${assetUrl(styleAssetPath("Settings.png"))}')`
  );
  document.documentElement.style.setProperty(
    "--settings-btn-hover-image",
    `url('${assetUrl(styleAssetPath("SettingsHover.png"))}')`
  );
}

function currentResolvedLauncherAssets() {
  const assets = [launcherHeaderUrl.value, dialogUrl.value, serverPatchUrl.value];
  if (storeMut.style === CLASSIC_STYLE) {
    assets.push(classicButtonUrl.value);
  }
  if (storeMut.style === PS4_STYLE) {
    assets.push(ps4ButtonUrl.value);
  }
  return assets.filter(Boolean);
}

function scheduleLauncherPreload({ includeBackground = true } = {}) {
  if (store.launcherSuspended) return;
  if (includeBackground) {
    preloadImages(backgroundUrl.value ? [backgroundUrl.value] : []);
  }
  preloadImages(getStyleAssets());
  preloadImages(currentResolvedLauncherAssets());
  preloadImages(sharedDialogAssets);
  deferPreload(DEFERRED_STARTUP_ASSETS);
}

function getStyleAssets() {
  if (storeMut.style === CLASSIC_STYLE) {
    return [...commonClassicLikeAssetNames, ...classicAssetNames].map((name) =>
      styleAssetPath(name)
    );
  }
  if (storeMut.style === PS4_STYLE) {
    return [...commonClassicLikeAssetNames, ...ps4AssetNames].map((name) =>
      styleAssetPath(name)
    );
  }
  return [];
}

let zenAntiqueFontStyle = null;
let msGothicFontStyle = null;
let customFontStyle = null;
let isLinuxDesktop = false;
let resizePersistTimer = null;
let resizeListenerAttached = false;
let resizeCorrectionInFlight = false;
let lastWindowSize = null;

// --- create & mount the app exactly as before ---
applyZenAntiqueFontFace();
setMsGothicFontFace();
applyCustomFontFace();

const app = createApp(Main).use(fluentVue);
app.mount("#app");

nextTick(() => {
  requestAnimationFrame(() => {
    appWindow.show().catch(() => undefined);
  });
});

const splash = document.getElementById("splash");
if (splash) splash.remove();

if (import.meta.env.DEV && typeof window !== "undefined") {
  window.__mezeportaFriendTest = function mezeportaFriendTest(options = {}) {
    const characters = Array.isArray(store.characters) ? store.characters : [];
    const fallbackCharacterId = Number(
      store.lastCharId ??
        characters.find((character) => Number(character?.id ?? 0) > 0)?.id ??
        1
    );
    const characterId = Number(options.characterId ?? fallbackCharacterId);
    const count = Math.max(1, Number(options.count ?? 50));
    const onlineIndexes = Array.isArray(options.online)
      ? options.online
      : [1, 4, 9, 14];
    const onlineIndexSet = new Set(
      onlineIndexes
        .map((value) => Number(value))
        .filter((value) => Number.isFinite(value) && value > 0)
    );

    const friends = Array.from({ length: count }, (_, index) => {
      const displayIndex = index + 1;
      return {
        cid: characterId,
        id: displayIndex,
        name: `Test Friend ${String(displayIndex).padStart(2, "0")}`,
      };
    });
    const onlineFriends = friends
      .filter((friend) => onlineIndexSet.has(friend.id))
      .map((friend, index) => ({
        ...friend,
        serverId: index + 1,
      }));

    setDevFriendTestEntries(friends);
    setDevAltClientStatsOverride({ onlineFriends });
    window.dispatchEvent(new CustomEvent("mezeporta:test-alt-stats-changed"));
    return {
      characterId,
      friends: friends.length,
      online: onlineFriends.map((friend) => friend.name),
    };
  };
}

// --- NEW: keep #app's wallpaper in sync with backgroundUrl ---
watch(
  backgroundUrl,
  (url) => {
    const value = url ? `url('${url}')` : "none";
    document.documentElement.style.setProperty("--launcher-bg-image", value);
  },
  { immediate: true }   // apply the current value on first load
);

watch(
  () => store.settings.fontPreset,
  (preset) => {
    const isCustomPreset = typeof preset === "string" && preset.startsWith("custom:");
    const family =
      preset === "classic"
        ? '"MS Gothic", "Zen Antique", serif'
        : isCustomPreset
        ? '"Custom Font", "Zen Antique", "MS Gothic", serif'
        : '"Zen Antique", "MS Gothic", serif';
    document.documentElement.style.setProperty("--launcher-font-family", family);

    if (preset === "classic" && document.fonts?.load) {
      document.fonts.load('16px "MS Gothic"');
    }
    if (isCustomPreset) {
      applyCustomFontFace();
      if (document.fonts?.load) {
        document.fonts.load('16px "Custom Font"');
      }
    }
  },
  { immediate: true }
);

function applyZenAntiqueFontFace() {
  const css = `@font-face { font-family: "Zen Antique"; src: url('${encodeURI(assetUrl("/fonts/ZenAntique-Regular.ttf"))}') format('truetype'); font-display: swap; }`;
  if (!zenAntiqueFontStyle) {
    zenAntiqueFontStyle = document.createElement("style");
    zenAntiqueFontStyle.id = "zen-antique-font-face";
    document.head.appendChild(zenAntiqueFontStyle);
  }
  if (zenAntiqueFontStyle.textContent !== css) {
    zenAntiqueFontStyle.textContent = css;
  }
  if (document.fonts?.load) {
    document.fonts.load('16px "Zen Antique"');
  }
}

function getCustomFontFileFromPreset(preset) {
  const value = String(preset ?? "").trim();
  if (!value.startsWith("custom:")) return null;

  const fileName = value.slice("custom:".length).trim();
  if (!fileName || /[\\/]/.test(fileName)) return null;
  if (!/\.(ttf|ttc|otf)$/i.test(fileName)) return null;
  return fileName;
}

async function getExternalCustomFontUrl() {
  const fileName = getCustomFontFileFromPreset(store.settings.fontPreset);
  if (!fileName) return null;

  const folder = effectiveFolder.value;
  if (!folder) return null;

  const candidatePaths = [
    await join(folder, "Mezeporta", "fonts", "Custom", fileName),
    await join(folder, "fonts", "Custom", fileName),
  ];

  for (const candidate of candidatePaths) {
    if (await exists(candidate)) {
      return `url('${convertFileSrc(candidate)}')`;
    }
  }

  return null;
}

async function applyCustomFontFace() {
  if (!customFontStyle) {
    customFontStyle = document.createElement("style");
    customFontStyle.id = "custom-font-face";
    document.head.appendChild(customFontStyle);
  }

  try {
    const customUrl = await getExternalCustomFontUrl();
    if (!customUrl) {
      customFontStyle.textContent = "";
      return;
    }

    const css = `@font-face { font-family: "Custom Font"; src: ${customUrl}; font-display: swap; }`;
    if (customFontStyle.textContent !== css) {
      customFontStyle.textContent = css;
    }
  } catch (_error) {
    customFontStyle.textContent = "";
  }
}
function getBundledMsGothicUrl() {
  return `url('${encodeURI(assetUrl("/fonts/MS Gothic.ttf"))}') format('truetype')`;
}

async function getExternalMsGothicUrl() {
  const folder = effectiveFolder.value;
  if (!folder) return null;

  const candidatePaths = [
    await join(folder, "Mezeporta", "fonts", "MS Gothic.ttf"),
    await join(folder, "fonts", "MS Gothic.ttf"),
  ];

  for (const candidate of candidatePaths) {
    if (await exists(candidate)) {
      return `url('${convertFileSrc(candidate)}') format('truetype')`;
    }
  }

  return null;
}

async function setMsGothicFontFace() {
  const sources = [];

  try {
    const externalUrl = await getExternalMsGothicUrl();
    if (externalUrl) {
      // Prefer user-managed font in Mezeporta/fonts over bundled fallback.
      sources.push(externalUrl);
    }
  } catch (_error) {
    // Keep bundled fallback when path probing fails.
  }

  sources.push(getBundledMsGothicUrl());

  const css = `@font-face { font-family: "MS Gothic"; src: ${sources.join(", ")}; font-display: swap; }`;

  if (!msGothicFontStyle) {
    msGothicFontStyle = document.createElement("style");
    msGothicFontStyle.id = "ms-gothic-font-face";
    document.head.appendChild(msGothicFontStyle);
  }
  if (msGothicFontStyle.textContent !== css) {
    msGothicFontStyle.textContent = css;
  }
  if (document.fonts?.load) {
    document.fonts.load('16px "MS Gothic"');
  }
}

watch(
  effectiveFolder,
  () => {
    setMsGothicFontFace();
    applyCustomFontFace();
  },
  { immediate: true }
);

watch(
  () => storeMut.style,
  () => {
    syncStyleAssetCssVars();
    scheduleLauncherPreload({ includeBackground: false });
  },
  { immediate: true }
);

watch(
  [classicButtonUrl, ps4ButtonUrl, launcherHeaderUrl, dialogUrl, serverPatchUrl],
  () => {
    scheduleLauncherPreload({ includeBackground: false });
  }
);

watch(
  () => store.settings.sfxEnabled,
  (enabled) => {
    if (enabled) preloadSfx();
  },
  { immediate: true }
);

watch(
  () => store.launcherSuspended,
  (suspended) => {
    if (suspended) {
      preloadedImages.clear();
      return;
    }
    scheduleLauncherPreload();
  }
);

function normalizeWindowSize(width, height, fallback) {
  const safeWidth = Number.parseInt(String(width ?? ""), 10);
  const safeHeight = Number.parseInt(String(height ?? ""), 10);
  return {
    width:
      Number.isFinite(safeWidth) && safeWidth > 0
        ? safeWidth
        : fallback.width,
    height:
      Number.isFinite(safeHeight) && safeHeight > 0
        ? safeHeight
        : fallback.height,
  };
}

function sameWindowSize(a, b) {
  return Boolean(a && b) && a.width === b.width && a.height === b.height;
}

function scheduleLauncherResolutionPersist(size) {
  if (!getLauncherResolutionPrefKeys(storeMut.style)) return;
  clearTimeout(resizePersistTimer);
  resizePersistTimer = setTimeout(() => {
    rememberLauncherResolution(storeMut.style, size);
  }, 120);
}

function correctedLauncherWindowSize(size) {
  const defaults = getLauncherWindowDefaults(storeMut.style);
  const normalized = normalizeWindowSize(size.width, size.height, defaults);
  if (!isLinuxDesktop || !isLauncherWindowResizable(storeMut.style)) {
    return normalized;
  }

  const aspect = getLauncherWindowAspect(storeMut.style);
  if (!aspect) return normalized;

  const previous = lastWindowSize ?? defaults;
  const widthDelta = Math.abs(normalized.width - previous.width);
  const heightDelta = Math.abs(normalized.height - previous.height);

  if (widthDelta >= heightDelta) {
    return {
      width: normalized.width,
      height: Math.max(
        1,
        Math.round((normalized.width * aspect.denominator) / aspect.numerator)
      ),
    };
  }

  return {
    width: Math.max(
      1,
      Math.round((normalized.height * aspect.numerator) / aspect.denominator)
    ),
    height: normalized.height,
  };
}

async function readWindowSizeFromEvent() {
  const size = await appWindow.innerSize();
  return {
    width: Math.round(size.width),
    height: Math.round(size.height),
  };
}

async function createPhysicalWindowSize(width, height) {
  return new PhysicalSize(width, height);
}

async function attachLauncherResizePersistence() {
  if (resizeListenerAttached) return;
  resizeListenerAttached = true;

  try {
    isLinuxDesktop = (await platform()) === "linux";
  } catch (_error) {
    isLinuxDesktop = false;
  }

  setLinuxAudioRuntimeStatus({
    platform: isLinuxDesktop ? "linux" : "other",
  });

  if (isLinuxDesktop) {
    await refreshLinuxPrefixStatus().catch(() => undefined);
  }

  try {
    const initialSize = await appWindow.innerSize();
    lastWindowSize = {
      width: Math.round(initialSize.width),
      height: Math.round(initialSize.height),
    };
  } catch (_error) {
    lastWindowSize = null;
  }

  appWindow.onResized(async (event) => {
    if (!isLauncherWindowResizable(storeMut.style)) {
      lastWindowSize = null;
      resizeCorrectionInFlight = false;
      return;
    }

    const liveSize = await readWindowSizeFromEvent();
    const corrected = correctedLauncherWindowSize(liveSize);

    if (resizeCorrectionInFlight) {
      resizeCorrectionInFlight = false;
      lastWindowSize = liveSize;
      scheduleLauncherResolutionPersist(liveSize);
      requestAnimationFrame(() => {
        window.dispatchEvent(new Event("resize"));
      });
      return;
    }

    if (!sameWindowSize(liveSize, corrected)) {
      resizeCorrectionInFlight = true;
      try {
        await appWindow.setSize(
          await createPhysicalWindowSize(corrected.width, corrected.height)
        );
        const settledSize = await readWindowSizeFromEvent();
        lastWindowSize = settledSize;
        scheduleLauncherResolutionPersist(settledSize);
        requestAnimationFrame(() => {
          window.dispatchEvent(new Event("resize"));
        });
      } catch (_error) {
        resizeCorrectionInFlight = false;
        lastWindowSize = liveSize;
        scheduleLauncherResolutionPersist(liveSize);
      }
      return;
    }

    lastWindowSize = liveSize;
    scheduleLauncherResolutionPersist(liveSize);
  });
}

attachLauncherResizePersistence();






