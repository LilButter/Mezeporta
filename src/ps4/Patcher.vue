<script setup>
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { getMessage } from "../fluent";

import {
  patcherPercentage,
  cancelPatcher,
  completePatcher,
  store,
  storeMut,
  assetUrl,
} from "../store";
import {
  CHECKING_PATCHER,
  DOWNLOADING_PATCHER,
  RESTORING_PATCHER,
  PATCHING_PATCHER,
  DONE_PATCHER,
  PATCHER_PAGE,
} from "../common";

const FRAME_HEIGHT = 91;
const LOADING_FRAMES = [0, 1, 2, 3];
const COMPLETE_FRAMES = [4, 5, 6, 7];
const LOADING_FRAME_DURATIONS = [100, 300, 100, 300];
const COMPLETE_FRAME_DURATION = 100;
const FINAL_FRAME_HOLD_TICKS = 2;

const patcherStage = ref("loading");
const patcherFrame = ref(LOADING_FRAMES[0]);
const patcherFrameStyle = computed(() => ({
  objectPosition: `0 -${patcherFrame.value * FRAME_HEIGHT}px`,
}));

const patcherFlavorText = computed(() =>
  patcherStage.value === "complete" ? "So Tasty!" : ""
);
const PATCH_BAR_WIDTH = 302;
const PATCH_BAR_LEFT = 8;
const patcherMaskWidth = computed(
  () => `${PATCH_BAR_WIDTH - PATCH_BAR_WIDTH * patcherPercentage.value}px`
);
const patcherPoogieLeft = computed(
  () =>
    `${PATCH_BAR_LEFT + PATCH_BAR_WIDTH * patcherPercentage.value}px`
);

const patcherRunActive = ref(false);
let loadingFrameIndex = 0;
let completeFrameIndex = 0;
let completeFinalHoldTicks = 0;
let pendingCompleteAfterLoop = false;
let frameTimer = null;

function clearPatcherTimers() {
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

function scheduleNextFrame(delayMs = COMPLETE_FRAME_DURATION) {
  clearPatcherTimers();
  frameTimer = setTimeout(stepFrame, delayMs);
}

function resetPatcherAnimation() {
  clearPatcherTimers();
  patcherStage.value = "loading";
  patcherFrame.value = LOADING_FRAMES[0];
  loadingFrameIndex = 0;
  completeFrameIndex = 0;
  completeFinalHoldTicks = 0;
  pendingCompleteAfterLoop = false;
}

function stepFrame() {
  if (patcherStage.value === "loading") {
    if (pendingCompleteAfterLoop && loadingFrameIndex === LOADING_FRAMES.length - 1) {
      patcherStage.value = "complete";
      completeFrameIndex = 0;
      completeFinalHoldTicks = 0;
      patcherFrame.value = COMPLETE_FRAMES[completeFrameIndex];
      scheduleNextFrame(COMPLETE_FRAME_DURATION);
      return;
    }

    loadingFrameIndex = (loadingFrameIndex + 1) % LOADING_FRAMES.length;
    patcherFrame.value = LOADING_FRAMES[loadingFrameIndex];
    scheduleNextFrame(loadingFrameDuration(loadingFrameIndex));
    return;
  }

  if (completeFrameIndex >= COMPLETE_FRAMES.length - 1) {
    completeFinalHoldTicks += 1;
    if (completeFinalHoldTicks < FINAL_FRAME_HOLD_TICKS) {
      scheduleNextFrame(COMPLETE_FRAME_DURATION);
      return;
    }
    clearPatcherTimers();
    completePatcher();
    return;
  }

  completeFrameIndex += 1;
  patcherFrame.value = COMPLETE_FRAMES[completeFrameIndex];
  scheduleNextFrame(COMPLETE_FRAME_DURATION);
}

function startLoadingStage() {
  resetPatcherAnimation();
  scheduleNextFrame(loadingFrameDuration(loadingFrameIndex));
}

function finishWithCompleteSprite() {
  if (patcherStage.value !== "loading") return;
  if (pendingCompleteAfterLoop) return;
  pendingCompleteAfterLoop = true;
  if (!frameTimer) {
    scheduleNextFrame(loadingFrameDuration(loadingFrameIndex));
  }
}

watch(
  () => storeMut.page,
  (page) => {
    if (page !== PATCHER_PAGE) {
      patcherRunActive.value = false;
      resetPatcherAnimation();
    }
  },
  { immediate: true }
);

watch(
  () => store.patcher.state,
  (state) => {
    if (
      state === CHECKING_PATCHER ||
      state === DOWNLOADING_PATCHER ||
      state === RESTORING_PATCHER ||
      state === PATCHING_PATCHER
    ) {
      patcherRunActive.value = true;
      startLoadingStage();
      return;
    }

    if (state === DONE_PATCHER) {
      if (storeMut.page !== PATCHER_PAGE || !patcherRunActive.value) {
        return;
      }
      finishWithCompleteSprite();
    }
  },
  { immediate: true }
);

onBeforeUnmount(() => {
  clearPatcherTimers();
});
</script>

<template>
  <div class="flex flex-col items-center">
    <div class="mt-10 min-h-[28px] text-center text-white tracking-wide drop-shadow-[0_0_8px_rgba(0,0,0,0.8)]">
      {{ patcherFlavorText }}
    </div>
    <img
      :src="assetUrl('/ps4/progress.png')"
      class="h-[91px] w-[85px] object-none"
      :style="patcherFrameStyle"
      draggable="false"
      alt="Patcher loading"
    />
    <div class="relative mt-10 h-[18px] w-[318px]">
      <img :src="assetUrl('/ps4/bar_frame.png')" class="absolute left-0 top-0" />
      <img
        :src="assetUrl('/ps4/bar.jpg')"
        class="absolute left-[8px] top-[5px] h-[6px] w-[302px] object-left"
      />
      <div
        class="absolute top-[5px] right-[8px] h-[6px] bg-black"
        :style="{ width: patcherMaskWidth }"
      ></div>
        <img
          :src="assetUrl('/extra/PoogieR.gif')"
          class="patcher-progress-poogie"
          :style="{ '--progress-poogie-left': patcherPoogieLeft }"
          draggable="false"
        />
    </div>

    <button
      class="box-text box-btn mt-[13px] mb-[12px]"
      :disabled="store.patcher.state === RESTORING_PATCHER || store.patcher.state === PATCHING_PATCHER || store.patcher.state === DONE_PATCHER"
      @click="cancelPatcher"
    >
      {{ $t("cancel-button") }}
    </button>
  </div>
</template>






