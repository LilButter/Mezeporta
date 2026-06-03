<script setup>
import { onBeforeUnmount, ref, watch, computed, reactive } from "vue";

import Character from "./Character.vue";
import { CHARACTERS_PAGE, copyCid, getCid, openPicker } from "../common";
import {
  store,
  storeMut,
  doCreateCharacter,
  doSelectCharacter,
  addPlaceholderCharacter,
  dialogDeleteCharacter,
  runCharacterLaunchAction,
  assetUrl,
  backgroundUrl,
  capcomUrl,
  cogUrl,
  dialogUrl,
  effectiveBanners,
  launcherHeaderUrl,
  ps4ButtonUrl,
  ps4EmblemUrl,
  serverPatchUrl,
  currentSavedataVersionToken,
} from "../store";

import { playHover, playSelect, playConfirm, playStartAndWait, playLogin, preloadLoginSfx } from "../sfx";
import { getMessage } from "../fluent";
import { hasAltCharacterSavedataVersion, prefetchAltCharacterSavedata } from "../altClientSavedata";

const rootRef = ref(null);
const props = defineProps({
  showAltButtons: {
    type: Boolean,
    default: false,
  },
});
const emit = defineEmits(["loading-log", "active-character", "toggle-gear", "toggle-box"]);

const lastCharIndex = store.characters.findIndex((c) => c.id === store.lastCharId);
const characterIndex = ref(lastCharIndex >= 0 ? lastCharIndex : 0);

const animationClass = "character-animation";
const animationClassReverse = "character-animation-reverse";
let characterTimeout = null;

const characterSettingsPicker = ref(false);

const adjacentVisible = ref(false);
let adjacentTimer = null;
const LAST_STARTED_LOGIN_CYCLE_KEY = "__mezeportaLastStartedLoginCycle";
function readLastStartedLoginCycle() {
  if (typeof window === "undefined") return 0;
  return Number(window[LAST_STARTED_LOGIN_CYCLE_KEY] ?? 0) || 0;
}
function writeLastStartedLoginCycle(cycle) {
  if (typeof window !== "undefined") {
    window[LAST_STARTED_LOGIN_CYCLE_KEY] = Number(cycle) || 0;
  }
}
let lastStartedLoginCycle = readLastStartedLoginCycle();

const FRAME_HEIGHT = 91;
const LOADING_FRAMES = [0, 1, 2, 3];
const COMPLETE_FRAMES = [4, 5, 6, 7];
const LOADING_FRAME_DURATIONS = [100, 200, 100, 200];
const COMPLETE_FRAME_DURATION = 100;
const FINAL_FRAME_HOLD_TICKS = 2;
const LOGIN_SFX_TO_SO_TASTY_MS = 1960;
const CHARACTER_PAGE_PRELOAD_TIMEOUT_MS = 3500;

const loaderStage = ref("idle");
const showLoginProgress = computed(() => loaderStage.value !== "idle");
const actionsReady = ref(true);
const altSavedataReady = ref(true);
const progressFrame = ref(LOADING_FRAMES[0]);
const progressFrameStyle = computed(() => ({
  objectPosition: `0 -${progressFrame.value * FRAME_HEIGHT}px`,
}));
const cycleReady = ref(false);

let loadingFrameIndex = 0;
let completeFrameIndex = 0;
let completeFinalHoldTicks = 0;
let pendingCompleteAfterLoop = false;
let frameTimer = null;
let loginCuePlayed = false;
let loginCueStartedAt = 0;
let loadingLoopCount = 0;

const showFlavorText = ref(false);
const flavorText = ref("");
const sizzlePulses = ref([]);
let nextSizzlePulseId = 1;
let characterPagePreloadRequest = 0;
const characterPageAssetsReady = ref(true);


const hasRealCharacter = computed(() =>
  store.characters.some((value) => Number(value?.id) > 0 && !value?.placeholder)
);
const isCharactersPageActive = computed(() => storeMut.page === CHARACTERS_PAGE);

const loadLogState = reactive({
  login: false,
  connected: false,
  fetching: false,
  finished: false,
});

function formatHunterList(names) {
  if (!Array.isArray(names) || names.length === 0) return "";
  if (names.length === 1) return names[0];
  if (names.length === 2) return `${names[0]} and ${names[1]}`;
  return `${names.slice(0, -1).join(", ")}, and ${names[names.length - 1]}`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function formatNameForLog(name) {
  return `<span style="font-weight:700;color:#ffd67c;">${escapeHtml(name)}</span>`;
}

function formatHunterListForLog(names) {
  const formatted = names.map(formatNameForLog);
  if (formatted.length === 0) return "";
  if (formatted.length === 1) return formatted[0];
  if (formatted.length === 2) return `${formatted[0]} and ${formatted[1]}`;
  return `${formatted.slice(0, -1).join(", ")}, and ${formatted[formatted.length - 1]}`;
}

const hunterNames = computed(() =>
  store.characters
    .map((character) => String(character?.name ?? "").trim())
    .filter((name) => name.length > 0)
);

const notifyingHuntersLine = computed(() => {
  if (hunterNames.value.length === 0) return `Gathering supplies...<br>----------------------------------------`;
  return `Preparing the Caravan...`;
});

const readyToDepartLine = computed(() => {
  if (hunterNames.value.length === 1) {
    return `~${formatNameForLog(hunterNames.value[0])} is ready to depart!`;
  }
  if (hunterNames.value.length === 2) {
    return `~${formatNameForLog(hunterNames.value[0])} and ${formatNameForLog(hunterNames.value[1])} are both ready to depart!`;
  }
  if (hunterNames.value.length === 3) {
    return `~${formatNameForLog(hunterNames.value[0])}, ${formatNameForLog(hunterNames.value[1])}, and ${formatNameForLog(hunterNames.value[2])} are all ready to depart!`;
  }
  if (hunterNames.value.length === 4) {
    return `~${formatNameForLog(hunterNames.value[0])}, ${formatNameForLog(hunterNames.value[1])}, ${formatNameForLog(hunterNames.value[2])}, and ${formatNameForLog(hunterNames.value[3])} are all ready to depart!`;
  }
  if (hunterNames.value.length >= 5) {
    return `~All Hunters are ready to depart!`;
  }
  return `~Preparations complete. The Caravan awaits.`;
});

const loadLogLines = computed(() => {
  const lines = [];
  if (loadLogState.login) {
    lines.push(`Logging in as <span style="font-weight:700;">${escapeHtml(storeMut.username || "Hunter")}</span>...`);
  }
  if (loadLogState.connected) lines.push(`~Guild Link Established!`);
  if (loadLogState.fetching) lines.push(notifyingHuntersLine.value);
  if (loadLogState.finished) lines.push(readyToDepartLine.value);
  return lines;
});

const showLoadingLog = computed(
  () => showLoginProgress.value && loadLogLines.value.length > 0
);
const patcherRevealVisible = ref(true);
const characterCardsVisible = computed(
  () => !showLoginProgress.value && patcherRevealVisible.value
);
const actionsVisible = computed(
  () => actionsReady.value && !showLoginProgress.value && patcherRevealVisible.value
);
let patcherRevealTimer = null;


watch(
  [showLoginProgress, loadLogLines],
  ([active, lines]) => {
    emit("loading-log", {
      active: Boolean(active),
      lines: Array.isArray(lines) ? [...lines] : [],
    });
  },
  { immediate: true }
);

let logStepTimer = null;
let fetchStepTimer = null;
let sizzleStartTimer = null;
let sizzleRepeatTimer = null;
let sizzleHideTimer = null;
let revealActionsTimer = null;

function clearRevealTimer() {
  if (revealActionsTimer) {
    clearTimeout(revealActionsTimer);
    revealActionsTimer = null;
  }
}

function clearPatcherRevealTimer() {
  if (patcherRevealTimer) {
    clearTimeout(patcherRevealTimer);
    patcherRevealTimer = null;
  }
}

function triggerPatcherReveal() {
  clearPatcherRevealTimer();
  patcherRevealVisible.value = false;
  patcherRevealTimer = setTimeout(() => {
    patcherRevealVisible.value = true;
    patcherRevealTimer = null;
  }, 24);
}

function clearLoadingFeedbackTimers() {
  if (logStepTimer) {
    clearTimeout(logStepTimer);
    logStepTimer = null;
  }
  if (fetchStepTimer) {
    clearTimeout(fetchStepTimer);
    fetchStepTimer = null;
  }
  if (sizzleStartTimer) {
    clearTimeout(sizzleStartTimer);
    sizzleStartTimer = null;
  }
  if (sizzleRepeatTimer) {
    clearInterval(sizzleRepeatTimer);
    sizzleRepeatTimer = null;
  }
  if (sizzleHideTimer) {
    clearTimeout(sizzleHideTimer);
    sizzleHideTimer = null;
  }
}

function showFlavor(text, visibleMs = 900) {
  if (sizzleHideTimer) {
    clearTimeout(sizzleHideTimer);
    sizzleHideTimer = null;
  }
  flavorText.value = text;
  showFlavorText.value = true;
  sizzleHideTimer = setTimeout(() => {
    showFlavorText.value = false;
    sizzleHideTimer = null;
  }, visibleMs);
}

function removeSizzlePulse(id) {
  sizzlePulses.value = sizzlePulses.value.filter((pulse) => pulse.id !== id);
}

function triggerSizzle() {
  if (loaderStage.value !== "loading") return;
  if (sizzlePulses.value.length > 0) return;
  sizzlePulses.value = [
    {
      id: nextSizzlePulseId++,
      text: getMessage("sizzle-label"),
    },
  ];
}

function resolvedLauncherOrigin() {
  if (!store.currentEndpoint || !store.currentEndpoint.url) return "";
  if (store.currentEndpoint.url === "OFFLINEMODE") return "";

  const rawUrl = store.currentEndpoint.url.includes("://")
    ? store.currentEndpoint.url
    : `http://${store.currentEndpoint.url}`;

  try {
    const parsed = new URL(rawUrl);
    if (store.currentEndpoint.launcherPort) {
      parsed.port = String(store.currentEndpoint.launcherPort);
    }
    return parsed.origin;
  } catch (_error) {
    return "";
  }
}

function portraitUrlsForCharacter(character) {
  const cacheBust = character?.lastLogin || Date.now();
  const characterId = Number(character?.id);
  const base = resolvedLauncherOrigin();
  if (!base || !Number.isFinite(characterId) || characterId <= 0) {
    return [];
  }

  const numericId = String(Math.trunc(characterId));
  const ids = [getCid(Math.trunc(characterId)), numericId]
    .map((value) => String(value ?? "").trim())
    .filter(Boolean);

  return [...new Set(ids)].map(
    (id) => `${base}/ClientImages/launcher/units/${encodeURIComponent(id)}.png?v=${cacheBust}`
  );
}

function preloadCharacterImage(url) {
  if (!url || typeof Image === "undefined") return Promise.resolve(false);
  return new Promise((resolve) => {
    const image = new Image();
    let settled = false;
    const timeout = setTimeout(() => finish(false), CHARACTER_PAGE_PRELOAD_TIMEOUT_MS);

    function finish(ok) {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      resolve(ok);
    }

    image.onload = () => finish(true);
    image.onerror = () => finish(false);
    image.src = url;
  });
}

function characterPagePreloadUrls() {
  const characterUnitUrls = store.characters.flatMap((character) => {
    if (!character || character.id === null || character.placeholder) {
      return [assetUrl("/units/unitbg.png")];
    }
    const fallback =
      typeof character.weapon === "number"
        ? assetUrl(`/units/${character.weapon}.png`)
        : assetUrl("/units/unitbg.png");
    return [fallback, ...portraitUrlsForCharacter(character)];
  });

  return [
    backgroundUrl.value,
    launcherHeaderUrl.value,
    capcomUrl.value,
    cogUrl.value,
    dialogUrl.value,
    serverPatchUrl.value,
    effectiveBanners.value?.[0]?.src,
    ps4ButtonUrl.value,
    ps4EmblemUrl.value,
    assetUrl("/units/unitbg.png"),
    assetUrl("/extra/Character.png"),
    assetUrl("/extra/Storage.png"),
    assetUrl("/extra/book.png"),
    assetUrl("/extra/bookOpen.png"),
    assetUrl("/extra/Face.png"),
    assetUrl("/extra/FaceHighlight.png"),
    assetUrl("/extra/MailClosed.png"),
    assetUrl("/extra/MailOpen.png"),
    assetUrl("/extra/ChestClosed.png"),
    assetUrl("/extra/ChestOpen.png"),
    ...characterUnitUrls,
  ].filter(Boolean);
}

async function preloadCharacterPageImages(blockProgress = false) {
  const request = ++characterPagePreloadRequest;
  if (blockProgress) {
    characterPageAssetsReady.value = false;
  }

  const urls = [...new Set(characterPagePreloadUrls())];
  await Promise.allSettled(urls.map(preloadCharacterImage));

  if (request !== characterPagePreloadRequest) return;
  characterPageAssetsReady.value = true;
  if (blockProgress) {
    maybeFinishLoginProgress();
  }
}

function startLoadingFeedback() {
  clearLoadingFeedbackTimers();
  loadLogState.login = true;
  loadLogState.connected = false;
  loadLogState.fetching = false;
  loadLogState.finished = false;
  showFlavorText.value = false;
  flavorText.value = "";
  sizzlePulses.value = [];
  playLoginCue();

  logStepTimer = setTimeout(() => {
    if (!showLoginProgress.value) return;
    loadLogState.connected = true;
  }, 420);

  fetchStepTimer = setTimeout(() => {
    if (!showLoginProgress.value) return;
    loadLogState.fetching = true;
    maybeFinishLoginProgress();
  }, 840);

}
function finishLoadingFeedback() {
  loadLogState.connected = true;
  loadLogState.fetching = true;
  loadLogState.finished = true;

  if (sizzleRepeatTimer) {
    clearInterval(sizzleRepeatTimer);
    sizzleRepeatTimer = null;
  }
  if (sizzleStartTimer) {
    clearTimeout(sizzleStartTimer);
    sizzleStartTimer = null;
  }
  if (sizzleHideTimer) {
    clearTimeout(sizzleHideTimer);
    sizzleHideTimer = null;
  }

  flavorText.value = getMessage("so-tasty-label");
  showFlavorText.value = true;
  sizzlePulses.value = [];
}

function clearProgressTimers() {
  if (frameTimer) {
    clearTimeout(frameTimer);
    frameTimer = null;
  }
}

function loadingFrameDuration(index) {
  return (
    LOADING_FRAME_DURATIONS[index] ??
    LOADING_FRAME_DURATIONS[LOADING_FRAME_DURATIONS.length - 1] ??
    COMPLETE_FRAME_DURATION
  );
}

function scheduleNextProgressFrame(delayMs = COMPLETE_FRAME_DURATION) {
  clearProgressTimers();
  frameTimer = setTimeout(stepProgressFrame, delayMs);
}

function resetProgressAnimation() {
  clearProgressTimers();
  loaderStage.value = "loading";
  progressFrame.value = LOADING_FRAMES[0];
  loadingFrameIndex = 0;
  completeFrameIndex = 0;
  completeFinalHoldTicks = 0;
  pendingCompleteAfterLoop = false;
  loginCuePlayed = false;
  loginCueStartedAt = 0;
  loadingLoopCount = 0;
}

function startLoginProgress() {
  if (!isCharactersPageActive.value) {
    stopLoginProgress();
    return;
  }
  clearRevealTimer();
  clearPatcherRevealTimer();
  patcherRevealVisible.value = true;
  cycleReady.value = false;
  actionsReady.value = false;
  resetProgressAnimation();
  preloadLoginSfx();
  playLoginCue();
  startLoadingFeedback();
  void preloadCharacterPageImages(true);

  if (!hasRealCharacter.value) {
    altSavedataReady.value = true;
  } else {
    altSavedataReady.value = false;
    void prefetchSelectedCharacterSavedata(true);
  }

  scheduleNextProgressFrame(loadingFrameDuration(loadingFrameIndex));
}

function stopLoginProgress() {
  clearProgressTimers();
  clearLoadingFeedbackTimers();
  clearRevealTimer();
  cycleReady.value = false;
  actionsReady.value = true;
  altSavedataReady.value = true;
  loaderStage.value = "idle";
  progressFrame.value = LOADING_FRAMES[0];
  loadingFrameIndex = 0;
  completeFrameIndex = 0;
  completeFinalHoldTicks = 0;
  pendingCompleteAfterLoop = false;
  loginCuePlayed = false;
  loginCueStartedAt = 0;
  loadingLoopCount = 0;
  characterPagePreloadRequest += 1;
  characterPageAssetsReady.value = true;
  showFlavorText.value = false;
  flavorText.value = "";
  sizzlePulses.value = [];
  loadLogState.login = false;
  loadLogState.connected = false;
  loadLogState.fetching = false;
  loadLogState.finished = false;
}
function beginActionReveal() {
  if (actionsReady.value) return;
  actionsReady.value = true;
  clearRevealTimer();
  revealActionsTimer = setTimeout(() => {
    revealActionsTimer = null;
    stopLoginProgress();
  }, 320);
}

function playLoginCue() {
  if (loginCuePlayed) return;
  loginCuePlayed = true;
  const cueRequestedAt = performance.now();
  loginCueStartedAt = cueRequestedAt;
  if (!store.settings?.sfxEnabled) return;
  const playResult = playLogin();
  if (playResult && typeof playResult.then === "function") {
    playResult.then((played) => {
      if (played && loginCueStartedAt === cueRequestedAt) {
        loginCueStartedAt = performance.now();
      }
    });
  }
}

function remainingLoginCueMs() {
  if (!store.settings?.sfxEnabled || !loginCueStartedAt) return 0;
  return Math.max(0, LOGIN_SFX_TO_SO_TASTY_MS - (performance.now() - loginCueStartedAt));
}

function stepProgressFrame() {
  if (loaderStage.value === "loading") {
    if (pendingCompleteAfterLoop && remainingLoginCueMs() <= 0) {
      loaderStage.value = "complete";
      completeFrameIndex = 0;
      completeFinalHoldTicks = 0;
      finishLoadingFeedback();
      progressFrame.value = COMPLETE_FRAMES[completeFrameIndex];
      scheduleNextProgressFrame(COMPLETE_FRAME_DURATION);
      return;
    }

    loadingFrameIndex = (loadingFrameIndex + 1) % LOADING_FRAMES.length;
    progressFrame.value = LOADING_FRAMES[loadingFrameIndex];
    if (loadingFrameIndex === LOADING_FRAMES.length - 1) {
      loadingLoopCount += 1;
      if (loadingLoopCount === 1 || (loadingLoopCount - 1) % 3 === 0) {
        triggerSizzle();
      }
    }
    const nextDelay = loadingFrameDuration(loadingFrameIndex);
    const cueDelay = remainingLoginCueMs();
    scheduleNextProgressFrame(
      pendingCompleteAfterLoop && cueDelay > 0
        ? Math.max(16, Math.min(nextDelay, cueDelay))
        : nextDelay
    );
    return;
  }

  if (loaderStage.value === "complete") {
    if (completeFrameIndex >= COMPLETE_FRAMES.length - 1) {
      completeFinalHoldTicks += 1;
      if (completeFinalHoldTicks < FINAL_FRAME_HOLD_TICKS) {
        scheduleNextProgressFrame(COMPLETE_FRAME_DURATION);
        return;
      }
      beginActionReveal();
      return;
    }

    completeFrameIndex += 1;
    progressFrame.value = COMPLETE_FRAMES[completeFrameIndex];
    scheduleNextProgressFrame(COMPLETE_FRAME_DURATION);
  }
}

function transitionToCompleteAfterLoop() {
  if (loaderStage.value !== "loading") return;
  if (pendingCompleteAfterLoop) return;
  pendingCompleteAfterLoop = true;
  playLoginCue();
  if (!frameTimer) {
    const nextDelay = loadingFrameDuration(loadingFrameIndex);
    const cueDelay = remainingLoginCueMs();
    scheduleNextProgressFrame(
      cueDelay > 0 ? Math.max(16, Math.min(nextDelay, cueDelay)) : nextDelay
    );
  }
}

async function prefetchSelectedCharacterSavedata(blockProgress = false) {
  if (!hasRealCharacter.value) {
    if (blockProgress) {
      altSavedataReady.value = true;
      maybeFinishLoginProgress();
    }
    return false;
  }

  const characterId = Number(store.characters?.[characterIndex.value]?.id ?? 0);
  if (!Number.isFinite(characterId) || characterId <= 0) {
    if (blockProgress) {
      altSavedataReady.value = true;
      maybeFinishLoginProgress();
    }
    return false;
  }

  if (blockProgress) {
    altSavedataReady.value = false;
  }

  const savedataVersion = currentSavedataVersionToken();
  const hasSavedataVersion = await hasAltCharacterSavedataVersion(
    characterId,
    savedataVersion
  );
  if (!hasSavedataVersion) {
    if (blockProgress) {
      altSavedataReady.value = true;
      maybeFinishLoginProgress();
    }
    return false;
  }

  const prefetched = await prefetchAltCharacterSavedata(
    characterId,
    savedataVersion
  );

  if (blockProgress) {
    altSavedataReady.value = true;
    maybeFinishLoginProgress();
  }

  return prefetched;
}

function maybeFinishLoginProgress() {
  if (!isCharactersPageActive.value) return;
  if (!showLoginProgress.value) return;
  if (!cycleReady.value) return;
  if (!characterPageAssetsReady.value) return;
  if (!loadLogState.fetching) return;
  if (!altSavedataReady.value) return;
  transitionToCompleteAfterLoop();
}

function onCurrentPortraitReady(cycle) {
  if (!isCharactersPageActive.value) return;
  if (cycle !== store.unitCardLoadCycle) return;
  cycleReady.value = true;
  maybeFinishLoginProgress();
}

function revealAdjacentDelayed() {
  if (adjacentTimer) clearTimeout(adjacentTimer);
  adjacentVisible.value = false;
  adjacentTimer = setTimeout(() => {
    adjacentVisible.value = true;
  }, 550);
}

watch(
  () => store.characters.length,
  (next, prev) => {
    if ((prev ?? 0) === 0 && next > 0) {
      revealAdjacentDelayed();
    }
  },
  { immediate: true }
);

watch(
  () => store.currentEndpoint?.url,
  (url) => {
    if (url === "OFFLINEMODE") {
      revealAdjacentDelayed();
    }
  }
);

watch(
  () => store.characterRevealCycle,
  (cycle, previous) => {
    if (cycle <= 0 || cycle <= (previous ?? 0)) return;
    stopLoginProgress();
    triggerPatcherReveal();
  }
);

onBeforeUnmount(() => {
  if (adjacentTimer) clearTimeout(adjacentTimer);
  clearPatcherRevealTimer();
  stopLoginProgress();
});

watch(
  () => [store.unitCardLoadCycle, storeMut.page],
  ([cycle, page]) => {
    if (page !== CHARACTERS_PAGE || cycle <= 0) {
      stopLoginProgress();
      if (cycle <= 0) {
        lastStartedLoginCycle = 0;
        writeLastStartedLoginCycle(0);
      }
      return;
    }
    if (cycle === lastStartedLoginCycle) return;
    lastStartedLoginCycle = cycle;
    writeLastStartedLoginCycle(cycle);
    startLoginProgress();
  },
  { immediate: true }
);

function clearAnimationClass() {
  for (const el of rootRef.value.querySelectorAll(".character")) {
    el.classList.remove(animationClass);
    el.classList.remove(animationClassReverse);
  }
}

function addAnimationClass(aclass) {
  for (const el of rootRef.value.querySelectorAll(".character")) {
    el.classList.add(aclass);
  }
}

watch(characterIndex, (newIndex, oldIndex) => {
  clearTimeout(characterTimeout);
  clearAnimationClass();
  const aclass = newIndex > oldIndex ? animationClass : animationClassReverse;
  setTimeout(() => {
    addAnimationClass(aclass);
    characterTimeout = setTimeout(clearAnimationClass, 300);
  }, 0);
  if (!showLoginProgress.value) {
    void prefetchSelectedCharacterSavedata(false);
  }
});

watch(
  () => store.characters,
  () => {
    if (store.characters.length <= characterIndex.value) {
      characterIndex.value = 0;
    }
  }
);

const character = computed(() => store.characters[characterIndex.value]);
const nextCharacter = computed(() => store.characters[characterIndex.value + 1]);
const nextNextCharacter = computed(() => store.characters[characterIndex.value + 2]);
const prevCharacter = computed(() => store.characters[characterIndex.value - 1]);
const prevPrevCharacter = computed(() => store.characters[characterIndex.value - 2]);

function emitActiveCharacterPayload() {
  const selected = store.characters?.[characterIndex.value];
  const id = Number(selected?.id ?? 0);
  const selectedIsReal =
    Number.isFinite(id) && id > 0 && !selected?.placeholder;
  emit("active-character", {
    id: selectedIsReal ? Math.trunc(id) : null,
    hasRealCharacter: selectedIsReal,
    placeholder: Boolean(selected?.placeholder),
  });
}

watch(
  () => [
    characterIndex.value,
    character.value?.id ?? null,
    Boolean(character.value?.placeholder),
    hasRealCharacter.value,
  ],
  () => {
    emitActiveCharacterPayload();
  },
  { immediate: true }
);
function onPrevClick() {
  playSelect();
  characterIndex.value--;
}

function onNextClick() {
  playSelect();
  characterIndex.value++;
}

function onCreateClick() {
  playSelect();
  addPlaceholderCharacter();
  characterIndex.value = store.characters.length - 1;
}

async function onStartClick() {
  if (!character.value) return;
  await runCharacterLaunchAction(async () => {
    await playStartAndWait();
    if (character.value.id === null || character.value.placeholder) {
      await doCreateCharacter();
    } else {
      await doSelectCharacter(character.value.id);
    }
  }, { showLaunchOverlay: true });
}

function onCharOptionsToggle() {
  playSelect();
  openPicker(characterSettingsPicker);
}

function onCopyCidClick() {
  playConfirm();
  copyCid(character.value.id);
  characterSettingsPicker.value = false;
}

function onDeleteCharacterClick() {
  if (!character.value || character.value.id === null || character.value.placeholder) return;
  playConfirm();
  dialogDeleteCharacter(character.value);
  characterSettingsPicker.value = false;
}
</script>

<template>
  <div class="h-full w-full relative" ref="rootRef">
    <div class="ps4-character-stage flex flex-col items-center mr-[-12px] relative">
      <transition name="loading-fade">
        <div
          v-if="showLoginProgress"
          class="absolute left-0 right-0 top-[39px] bottom-[39px] z-[20] pointer-events-none flex flex-col items-center justify-center"
        >
          <div class="relative h-[91px] w-[85px]">
            <div
              v-for="pulse in sizzlePulses"
              :key="pulse.id"
              class="login-sizzle-smoke"
              @animationend="removeSizzlePulse(pulse.id)"
            >
              {{ pulse.text }}
            </div>
            <div
              class="absolute left-1/2 top-[-28px] h-[22px] w-[260px] -translate-x-1/2 text-center text-[18px] text-white tracking-wide drop-shadow-[0_0_8px_rgba(0,0,0,0.85)] transition-opacity duration-200"
              :class="showFlavorText ? 'opacity-100' : 'opacity-0'"
            >
              {{ flavorText }}
            </div>
            <img
              :src="assetUrl('/ps4/progress.png')"
              alt="Loading character cards"
              class="h-[91px] w-[85px] object-none"
              :style="progressFrameStyle"
              draggable="false"
            />
          </div>
        </div>
      </transition>

      <div class="ps4-character-carousel">
        <div
          class="ps4-character-nav ps4-character-nav-left transition-opacity duration-300"
          :class="actionsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        >
          <button
            v-if="adjacentVisible && prevCharacter"
            type="button"
            class="ps4-character-nav-button ps4-character-nav-up"
            data-controller-node="character-prev"
            data-controller-size="small"
            data-controller-priority="22"
            @mouseenter="playHover()"
            @click="onPrevClick"
          ></button>
          <button
            v-if="adjacentVisible && nextCharacter"
            type="button"
            class="ps4-character-nav-button ps4-character-nav-down"
            data-controller-node="character-next"
            data-controller-size="small"
            data-controller-priority="23"
            @mouseenter="playHover()"
            @click="onNextClick"
          ></button>
        </div>

      <Character
        class="character z-[5] transition-opacity duration-200"
        :class="characterCardsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        :character="character"
        :selectable="true"
        :load-cycle="store.unitCardLoadCycle"
        :show-gear-button="false"
        :show-box-button="false"
        data-controller-node="character-card"
        @portrait-ready="onCurrentPortraitReady"
        @gear-click="emit('toggle-gear')"
        @box-click="emit('toggle-box')"
      />

        <div
          class="ps4-character-nav ps4-character-nav-right transition-opacity duration-300"
          :class="actionsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        >
        </div>
      </div>
    </div>

    <div
      class="ps4-character-actions transition-opacity duration-300"
      :class="actionsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'"
    >
      <div class="ps4-character-action-row">
        <button
          class="ps4-character-secondary-button"
          data-controller-node="character-create"
          data-controller-size="big"
          data-controller-priority="11"
          @mouseenter="playHover()"
          @click="onCreateClick"
        >
          {{ $t('create-character-label') }}
        </button>

        <div
          class="ps4-character-secondary-button ps4-character-options-shell"
        >
          <button
            class="ps4-character-options-button"
            data-controller-node="character-options"
            data-controller-size="big"
            data-controller-priority="12"
            @click="onCharOptionsToggle"
          >
            <span class="whitespace-nowrap">{{ $t('options-character-label') }}</span>
            <span :class="characterSettingsPicker ? 'arrow-up' : 'arrow-down'"></span>
          </button>

          <div v-if="characterSettingsPicker" class="absolute w-[192px]">
            <div
              class="ps4-character-options-dropdown"
            >
              <button class="w-full px-2 py-0.5 hover:bg-[#304368b8]" @click="onCopyCidClick">
                {{ $t('copy-cid-label') }}
              </button>
              <button
                v-if="character && character.id !== null && !character.placeholder"
                class="w-full px-2 py-0.5 hover:bg-[#304368b8]"
                @click="onDeleteCharacterClick"
              >
                {{ $t('delete-character-label') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="ps4-character-start-row">
        <button
          class="ps4-start-button ps4-character-start-button font-main"
          data-controller-node="character-start"
          data-controller-size="big"
          data-controller-priority="13"
          @mouseenter="playHover()"
          @click="onStartClick"
        >
          <span class="ps4-start-button-label">{{ $t('start-game-label') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>






















