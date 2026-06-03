<script setup>
import { ref, watch, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

import "./style.css";

import {
  storeMut,
  store,
  initStore,
  initRemoteEndpoints,
  updateRemoteMessages,
  updatePatcher,
  logText,
  resumeLauncherResources,
  closeLauncher,
  backgroundUrl,
  assetUrl,
  launcherAssetsLoading,
} from "./store";
import ClassicLauncher from "./classic/Launcher.vue";
import Ps4Launcher from "./ps4/Launcher.vue";
import {
  CLASSIC_STYLE,
  PS4_STYLE,
} from "./common";
import { logMessage } from "./store";

const uiScale = ref(1);
const CLASSIC_SIZE = { width: 1124, height: 600 };
const PS4_SIZE = { width: 1280, height: 720 };
let dpiMediaQuery = null;

// Recalculate scale whenever window resizes or style changes
function updateScale() {
  const usesFreeScaling =
    storeMut.style === PS4_STYLE || storeMut.style === CLASSIC_STYLE;
  const design = storeMut.style === PS4_STYLE ? PS4_SIZE : CLASSIC_SIZE;
  const designW = design.width;
  const designH = design.height;
  const w = window.innerWidth;
  const h = window.innerHeight;
  const targetScale = Math.min(w / designW, h / designH);
  uiScale.value = usesFreeScaling ? targetScale : Math.min(targetScale, 1);
  const scaledW = designW * uiScale.value;
  const scaledH = designH * uiScale.value;
  const offsetX = Math.max((w - scaledW) / 2, 0);
  const offsetY = usesFreeScaling ? Math.max((h - scaledH) / 2, 0) : 0;
  document.documentElement.style.setProperty("--ui-width", `${designW}px`);
  document.documentElement.style.setProperty("--ui-height", `${designH}px`);
  document.documentElement.style.setProperty("--ui-offset-x", `${offsetX}px`);
  document.documentElement.style.setProperty("--ui-offset-y", `${offsetY}px`);
  document.documentElement.style.setProperty(
    "--ui-scale",
    uiScale.value.toString()
  );
}

onMounted(() => {
  updateScale();
  window.addEventListener("resize", updateScale);
  if (window.visualViewport) {
    window.visualViewport.addEventListener("resize", updateScale);
  }
  appWindow
    .onResized(() => {
      requestAnimationFrame(() => {
        updateScale();
        requestAnimationFrame(updateScale);
      });
    })
    .then((unlisten) => {
      eventUnlisteners.push(unlisten);
    })
    .catch(() => {
      // ignore resize listener registration errors in non-tauri contexts
    });
  watchDevicePixelRatio();
});

// Re-run scale when user flips between launcher styles.
watch(() => storeMut.style, () => {
  updateScale();
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateScale);
  if (window.visualViewport) {
    window.visualViewport.removeEventListener("resize", updateScale);
  }
  if (dpiMediaQuery) {
    dpiMediaQuery.removeEventListener("change", handleDpiChange);
  }
  while (eventUnlisteners.length) {
    const unlisten = eventUnlisteners.pop();
    try {
      unlisten?.();
    } catch (_error) {
      // ignore listener cleanup errors
    }
  }
});

function handleDpiChange() {
  updateScale();
  watchDevicePixelRatio();
}

function watchDevicePixelRatio() {
  if (!window.matchMedia) return;
  if (dpiMediaQuery) {
    dpiMediaQuery.removeEventListener("change", handleDpiChange);
  }
  dpiMediaQuery = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
  dpiMediaQuery.addEventListener("change", handleDpiChange);
}

const initialLoaded = ref(false);
const showLoadingOverlay = ref(true);
const overlayState = computed(() => {
  return {
    visible: showLoadingOverlay.value || launcherAssetsLoading.value,
    text: "Now loading",
    serverPhase: !showLoadingOverlay.value && launcherAssetsLoading.value,
  };
});
const fallbackBackgroundUrl = computed(() =>
  storeMut.style === PS4_STYLE
    ? assetUrl("/ps4/BackgroundALT5.png")
    : assetUrl("/backgroundALT7.jpg")
);
const loadingWindowControls = computed(() => {
  const ps4Style = storeMut.style === PS4_STYLE;
  return {
    ps4Style,
    minimize: assetUrl(ps4Style ? "/ps4/minimize.png" : "/classic/minimize.png"),
    maximize: assetUrl(ps4Style ? "/ps4/Maximize.png" : "/classic/Maximize.png"),
    close: assetUrl(ps4Style ? "/ps4/close.png" : "/classic/close.png"),
  };
});
const fetchedBackgroundVisible = ref(false);
const fetchedBackgroundUrl = ref("");
let backgroundLoadToken = 0;
let lastBackgroundPreloadUrl = "";
let lastBackgroundStyle = null;
const eventUnlisteners = [];

function preloadBackground(url) {
  if (!url) {
    return Promise.resolve(false);
  }
  if (url === lastBackgroundPreloadUrl) {
    return Promise.resolve(true);
  }
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => {
      lastBackgroundPreloadUrl = url;
      resolve(true);
    };
    image.onerror = () => {
      if (lastBackgroundPreloadUrl === url) {
        lastBackgroundPreloadUrl = "";
      }
      resolve(false);
    };
    image.src = url;
  });
}

watch(
  [backgroundUrl, fallbackBackgroundUrl, () => storeMut.style],
  async ([url, fallbackUrl, style]) => {
    const token = ++backgroundLoadToken;
    const targetUrl = typeof url === "string" ? url : "";
    const styleChanged = lastBackgroundStyle !== style;
    lastBackgroundStyle = style;

    if (styleChanged) {
      fetchedBackgroundVisible.value = false;
      fetchedBackgroundUrl.value = "";
      lastBackgroundPreloadUrl = "";
    } else if (fetchedBackgroundUrl.value && fetchedBackgroundUrl.value !== targetUrl) {
      fetchedBackgroundVisible.value = false;
    }

    if (!targetUrl || targetUrl === fallbackUrl) {
      fetchedBackgroundVisible.value = false;
      fetchedBackgroundUrl.value = "";
      lastBackgroundPreloadUrl = "";
      return;
    }

    const loaded = await preloadBackground(targetUrl);
    if (
      !loaded ||
      token !== backgroundLoadToken ||
      storeMut.style !== style ||
      backgroundUrl.value !== targetUrl ||
      fallbackBackgroundUrl.value !== fallbackUrl
    ) {
      return;
    }

    if (!fetchedBackgroundUrl.value || fetchedBackgroundUrl.value === targetUrl) {
      fetchedBackgroundUrl.value = targetUrl;
      requestAnimationFrame(() => {
        if (
          token === backgroundLoadToken &&
          storeMut.style === style &&
          backgroundUrl.value === targetUrl
        ) {
          fetchedBackgroundVisible.value = true;
        }
      });
      return;
    }

    fetchedBackgroundVisible.value = false;
    await nextTick();
    if (
      token !== backgroundLoadToken ||
      storeMut.style !== style ||
      backgroundUrl.value !== targetUrl ||
      fallbackBackgroundUrl.value !== fallbackUrl
    ) return;
    fetchedBackgroundUrl.value = targetUrl;
    requestAnimationFrame(() => {
      if (
        token === backgroundLoadToken &&
        storeMut.style === style &&
        backgroundUrl.value === targetUrl
      ) {
        fetchedBackgroundVisible.value = true;
      }
    });
  },
  { immediate: true }
);

function hideLoadingOverlay() {
  window.setTimeout(() => {
    showLoadingOverlay.value = false;
  }, 260);
}

function onLoadingOverlayClose() {
  closeLauncher();
}

async function onLoadingOverlayMaximize() {
  await appWindow.toggleMaximize();
}


function registerEventListener(eventName, handler) {
  listen(eventName, handler)
    .then((unlisten) => {
      eventUnlisteners.push(unlisten);
    })
    .catch(() => {
      // ignore listen registration errors in non-tauri contexts
    });
}

initStore()
  .then(() => {
    initialLoaded.value = true;
    hideLoadingOverlay();
  })
  .catch((e) => {
    // Keep the UI visible even if the backend fails to start.
    // This preserves the 1:1 Vue UI; it just avoids "background only".
    try {
      const msg = String(e?.message ?? e);
      logText("error", `initStore failed: ${msg}`);
      // Also print to console for DevTools.
      // eslint-disable-next-line no-console
      console.error("initStore failed:", e);
    } catch (_) {}
    initialLoaded.value = true;
    hideLoadingOverlay();
  });

// Surface backend startup failures in the existing launcher log.
registerEventListener("backend_error", ({ payload }) => {
  logText("error", String(payload));
});
registerEventListener("backend_stderr", ({ payload }) => {
  logText("error", String(payload));
});
registerEventListener("backend_exit", () => {
  storeMut.page = LOGIN_PAGE;
});

registerEventListener("userdata", ({ payload }) => {
  storeMut.username   = payload.userdata.username;
  storeMut.password   = payload.password;
  storeMut.rememberMe = payload.userdata.rememberMe;
});
registerEventListener("endpoints", ({ payload }) => {
  initRemoteEndpoints(payload);
});
registerEventListener("remote_messages", ({ payload }) => {
  updateRemoteMessages(payload);
});
registerEventListener("patcher", ({ payload }) => {
  updatePatcher(payload);
});
registerEventListener("log", ({ payload }) => {
  logMessage(payload.level, payload.message);
});
registerEventListener("game_launch_timeout", () => {
  resumeLauncherResources();
  storeMut.page = LOGIN_PAGE;
});
registerEventListener("game_launch_ready", () => {
  closeLauncher();
});
registerEventListener("game_exit", () => {
  resumeLauncherResources();
  storeMut.page = LOGIN_PAGE;
});
registerEventListener("game_launch", () => {
  closeLauncher();
});
</script>

<template>
  <div
    id="app-wrapper"
  >
    <div
      class="launcher-bg-layer launcher-bg-base"
      :style="{ backgroundImage: `url('${fallbackBackgroundUrl}')` }"
    ></div>
    <div
      class="launcher-bg-layer launcher-bg-fetched"
      :class="{ 'is-visible': fetchedBackgroundVisible }"
      :style="{ backgroundImage: fetchedBackgroundUrl ? `url('${fetchedBackgroundUrl}')` : 'none' }"
    ></div>

    <div class="launcher-content">
      <template v-if="initialLoaded">
        <ClassicLauncher
          v-if="storeMut.style === CLASSIC_STYLE"
        />
        <Ps4Launcher
          v-else
        />
      </template>
    </div>

    <div
      v-if="overlayState.visible"
      class="launcher-loading-window-bar"
      :class="{ 'launcher-loading-window-bar-ps4': loadingWindowControls.ps4Style }"
    >
      <div data-tauri-drag-region class="launcher-loading-window-drag"></div>
      <img
        :src="loadingWindowControls.minimize"
        class="launcher-loading-window-image state-img"
        draggable="false"
        @click="appWindow.minimize()"
      />
      <img
        :src="loadingWindowControls.maximize"
        class="launcher-loading-window-image state-img"
        draggable="false"
        @click="onLoadingOverlayMaximize"
      />
      <img
        :src="loadingWindowControls.close"
        class="launcher-loading-window-image state-img"
        draggable="false"
        @click="onLoadingOverlayClose"
      />
    </div>

    <transition name="loading-fade">
      <div
        v-if="overlayState.visible"
        class="launcher-loading"
        :class="{ 'launcher-loading-assets': overlayState.serverPhase }"
      >
        <img
          class="launcher-loading-title-image"
          :src="assetUrl('/headers/PS4/MezeportaPS4.png')"
          alt=""
          draggable="false"
        />
        <img
          class="launcher-loading-status-image"
          :src="assetUrl('/extra/Now-Loading.gif')"
          :alt="overlayState.text"
          draggable="false"
        />
      </div>
    </transition>
  </div>
</template>
