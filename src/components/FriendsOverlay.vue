<script setup>
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { assetUrl, store } from "../store";
import { playHover, playSelect } from "../sfx";
import { getCid } from "../common";

const props = defineProps({
  open: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  hasCharacter: {
    type: Boolean,
    default: false,
  },
  entries: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(["close"]);

const contentRoot = ref(null);
const isRendered = ref(Boolean(props.open));
const isOpening = ref(false);
const isClosing = ref(false);
const pageIndex = ref(0);

const OVERLAY_ANIMATION_MS = 260;
const BOOK_PAGE_SIZE = 26;
const MAIL_PAGE_LIMIT = 13;
let overlayTimer = null;

const friendsEntries = computed(() =>
  Array.isArray(props.entries)
    ? props.entries
        .map((friend) => ({
          id: Number(friend?.id ?? 0),
          name: String(friend?.name ?? "").trim(),
          online: Boolean(friend?.online),
        }))
        .filter((friend) => Number.isFinite(friend.id) && friend.id > 0)
        .map((friend) => ({
          ...friend,
          displayId: getCid(friend.id),
        }))
        .sort((left, right) => {
          if (left.online !== right.online) return left.online ? -1 : 1;
          const nameCompare = left.name.localeCompare(right.name, undefined, {
            sensitivity: "base",
          });
          return nameCompare || left.id - right.id;
        })
    : []
);

const useBookLayout = computed(() => friendsEntries.value.length > MAIL_PAGE_LIMIT);
const totalBookPages = computed(() =>
  Math.max(1, Math.ceil(friendsEntries.value.length / BOOK_PAGE_SIZE))
);
const showBookPaging = computed(() => useBookLayout.value && totalBookPages.value > 1);
const bookPageLabel = computed(() => `Page ${pageIndex.value + 1}/${totalBookPages.value}`);
const visibleBookEntries = computed(() => {
  const start = pageIndex.value * BOOK_PAGE_SIZE;
  return friendsEntries.value.slice(start, start + BOOK_PAGE_SIZE);
});
const leftBookEntries = computed(() => visibleBookEntries.value.slice(0, 13));
const rightBookEntries = computed(() => visibleBookEntries.value.slice(13, 26));
const mailEntries = computed(() => friendsEntries.value.slice(0, MAIL_PAGE_LIMIT));
const canPagePrev = computed(() => useBookLayout.value && pageIndex.value > 0);
const canPageNext = computed(
  () => useBookLayout.value && pageIndex.value < totalBookPages.value - 1
);

const overlayBackdropStyle = computed(() => {
  if (store.settings.linuxHardwareAcceleration !== false) {
    return null;
  }
  return {
    background: "radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%)",
    backdropFilter: "none",
  };
});

function clearOverlayTimer() {
  if (overlayTimer) {
    clearTimeout(overlayTimer);
    overlayTimer = null;
  }
}

function startOpenAnimation() {
  clearOverlayTimer();
  isRendered.value = true;
  isClosing.value = false;
  isOpening.value = true;
  overlayTimer = setTimeout(() => {
    isOpening.value = false;
    overlayTimer = null;
  }, OVERLAY_ANIMATION_MS);
}

function startCloseAnimation() {
  clearOverlayTimer();
  if (!isRendered.value) return;
  isOpening.value = false;
  isClosing.value = true;
  overlayTimer = setTimeout(() => {
    isClosing.value = false;
    isRendered.value = false;
    overlayTimer = null;
  }, OVERLAY_ANIMATION_MS);
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      pageIndex.value = 0;
      startOpenAnimation();
    } else {
      startCloseAnimation();
    }
  },
  { immediate: true }
);

watch(friendsEntries, () => {
  if (pageIndex.value > totalBookPages.value - 1) {
    pageIndex.value = Math.max(0, totalBookPages.value - 1);
  }
});

function statusTitle(friend) {
  return friend.online ? "Online" : "Offline";
}

function goToPage(delta) {
  const nextPage = pageIndex.value + delta;
  if (nextPage < 0 || nextPage >= totalBookPages.value) return;
  playSelect();
  pageIndex.value = nextPage;
}

function onCloseClick() {
  playSelect();
  emit("close");
}

onBeforeUnmount(() => {
  clearOverlayTimer();
});

defineExpose({
  contentRoot,
});
</script>

<template>
  <div
    v-if="isRendered"
    class="friends-overlay"
    :class="{
      'friends-overlay-opening': isOpening,
      'friends-overlay-closing': isClosing,
    }"
    @click.self="onCloseClick"
  >
    <div
      class="friends-overlay-backdrop"
      :class="{ 'friends-overlay-backdrop-static': store.settings.linuxHardwareAcceleration === false }"
      :style="overlayBackdropStyle"
      @click="onCloseClick"
    ></div>

    <div ref="contentRoot" class="friends-overlay-stage" data-controller-scope="friends">
      <template v-if="useBookLayout">
        <div class="friends-book-shell">
          <button
            type="button"
            class="friends-overlay-close friends-book-close font-main"
            data-controller-size="big"
            @mouseenter="playHover()"
            @click="onCloseClick"
          >
            X
          </button>
          <div class="friends-book-title font-main">Friends List:</div>
          <div v-if="showBookPaging" class="friends-book-page-count font-main">
            {{ bookPageLabel }}
          </div>
          <button
            v-if="showBookPaging && canPagePrev"
            type="button"
            class="friends-book-page-button friends-book-page-button-prev"
            aria-label="Previous friends page"
            @mouseenter="playHover()"
            @click="goToPage(-1)"
          >
            <img :src="assetUrl('/extra/bookprev.png')" draggable="false" alt="" />
          </button>
          <button
            v-if="showBookPaging && canPageNext"
            type="button"
            class="friends-book-page-button friends-book-page-button-next"
            aria-label="Next friends page"
            @mouseenter="playHover()"
            @click="goToPage(1)"
          >
            <img :src="assetUrl('/extra/booknext.png')" draggable="false" alt="" />
          </button>
          <div class="friends-book-page friends-book-page-left">
            <div
              v-for="friend in leftBookEntries"
              :key="`friends-book-left-${friend.id}`"
              class="friends-entry font-main"
            >
              <span
                class="friends-status-icon"
                :class="{ 'friends-status-icon-online': friend.online }"
                :title="statusTitle(friend)"
                aria-hidden="true"
              ></span>
              <div class="friends-entry-text">
                <div class="friends-entry-title">{{ friend.name || `Friend #${friend.id}` }}</div>
                <div class="friends-entry-meta">ID {{ friend.displayId }}</div>
              </div>
            </div>
          </div>
          <div class="friends-book-page friends-book-page-right">
            <div
              v-for="friend in rightBookEntries"
              :key="`friends-book-right-${friend.id}`"
              class="friends-entry font-main"
            >
              <span
                class="friends-status-icon"
                :class="{ 'friends-status-icon-online': friend.online }"
                :title="statusTitle(friend)"
                aria-hidden="true"
              ></span>
              <div class="friends-entry-text">
                <div class="friends-entry-title">{{ friend.name || `Friend #${friend.id}` }}</div>
                <div class="friends-entry-meta">ID {{ friend.displayId }}</div>
              </div>
            </div>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="friends-mail-shell">
          <button
            type="button"
            class="friends-overlay-close friends-mail-close"
            data-controller-size="small"
            @mouseenter="playHover()"
            @click="onCloseClick"
          >
            <span class="friends-mail-close-art" aria-hidden="true"></span>
          </button>
          <div class="friends-mail-title font-main">Friends List:</div>
          <div class="friends-mail-list">
            <template v-if="!hasCharacter">
              <div class="friends-empty font-main">No character selected.</div>
            </template>
            <template v-else-if="loading">
              <div class="friends-empty font-main">now loading...</div>
            </template>
            <template v-else-if="mailEntries.length">
              <div
                v-for="friend in mailEntries"
                :key="`friends-mail-${friend.id}`"
                class="friends-entry font-main"
              >
                <span
                  class="friends-status-icon"
                  :class="{ 'friends-status-icon-online': friend.online }"
                  :title="statusTitle(friend)"
                  aria-hidden="true"
                ></span>
                <div class="friends-entry-text">
                  <div class="friends-entry-title">{{ friend.name || `Friend #${friend.id}` }}</div>
                  <div class="friends-entry-meta">ID {{ friend.displayId }}</div>
                </div>
              </div>
            </template>
            <template v-else>
              <div class="friends-empty font-main">No friends listed.</div>
            </template>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.friends-overlay {
  position: absolute;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.friends-overlay-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.58);
  backdrop-filter: blur(4px) brightness(0.35);
}

.friends-overlay-backdrop-static {
  background: radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%);
}

.friends-overlay-stage {
  position: relative;
  z-index: 1;
  pointer-events: none;
}

.friends-book-shell {
  position: relative;
  width: 760px;
  height: 570px;
  background-image: url("/extra/MainBook.png");
  background-position: center;
  background-repeat: no-repeat;
  background-size: contain;
  color: #3f2d1f;
  pointer-events: auto;
}

.friends-book-title {
  position: absolute;
  top: 18px;
  left: 76px;
  width: 230px;
  text-align: center;
  font-size: 1.15rem;
  font-weight: 700;
  line-height: 1;
  text-shadow: none;
}

.friends-book-page-count {
  position: absolute;
  top: 40px;
  left: 76px;
  width: 230px;
  text-align: center;
  font-size: 0.8rem;
  line-height: 1.1;
  color: #6c4a30;
  text-shadow: none;
}

.friends-book-page {
  position: absolute;
  top: 83px;
  bottom: 58px;
  width: 298px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.friends-book-page-left {
  left: 42px;
}

.friends-book-page-right {
  right: 42px;
}

.friends-book-close {
  position: absolute;
  top: 10px;
  right: 38px;
  min-width: 36px;
  min-height: 24px;
  border: 0;
  background: transparent;
  color: #3e2717;
  font-size: 22px;
  line-height: 1;
  cursor: pointer;
}

.friends-book-page-button {
  position: absolute;
  top: 28px;
  z-index: 3;
  width: 48px;
  height: 48px;
  padding: 0;
  border: 0;
  background: transparent;
  cursor: pointer;
  transition: filter 140ms ease, transform 140ms ease;
}

.friends-book-page-button:hover,
.friends-book-page-button:focus-visible {
  filter: brightness(1.12);
  transform: scale(1.06);
}

.friends-book-page-button:disabled {
  cursor: default;
  opacity: 0.38;
  filter: grayscale(0.6);
  transform: none;
}

.friends-book-page-button img {
  width: 44px;
  height: 44px;
  object-fit: contain;
  pointer-events: none;
}

.friends-book-page-button-prev {
  left: 298px;
}

.friends-book-page-button-next {
  left: 400px;
}

.friends-mail-shell {
  position: relative;
  width: 476px;
  height: 558px;
  background-image: url("/extra/MainMail.png");
  background-position: center;
  background-repeat: no-repeat;
  background-size: 100% 100%;
  color: #3f2d1f;
  pointer-events: auto;
}

.friends-mail-close {
  position: absolute;
  top: 16px;
  right: 14px;
  width: 31px;
  height: 31px;
  padding: 0;
  border: 0;
  background: transparent;
  cursor: pointer;
}

.friends-mail-close-art {
  display: block;
  width: 100%;
  height: 100%;
  background-image: url("/extra/MailClose.png");
  background-position: center;
  background-repeat: no-repeat;
  background-size: contain;
}

.friends-mail-close:hover .friends-mail-close-art,
.friends-mail-close:focus-visible .friends-mail-close-art {
  background-image: url("/extra/MailCloseHover.png");
}

.friends-mail-title {
  position: absolute;
  top: 64px;
  left: 34px;
  right: 34px;
  text-align: center;
  color: #5a3e2a;
  font-size: 27px;
  line-height: 1.04;
  font-weight: 400;
  text-shadow: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.friends-mail-list {
  position: absolute;
  top: 126px;
  left: 62px;
  right: 58px;
  bottom: 72px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  overflow: hidden;
}

.friends-entry {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: 7px;
  align-items: start;
  min-height: 30px;
  color: #3f2d1f;
  text-shadow: none;
}

.friends-status-icon {
  width: 22px;
  height: 22px;
  margin-top: 2px;
  background-image: url("/extra/FriendStatus.png");
  background-position: 0 0;
  background-repeat: no-repeat;
  background-size: 22px 44px;
}

.friends-status-icon-online {
  background-position: 0 -22px;
}

.friends-entry-text {
  min-width: 0;
}

.friends-entry-title {
  font-size: 1rem;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.friends-entry-meta {
  font-size: 0.7rem;
  line-height: 1.12;
  color: #3f2d1f;
}

.friends-empty {
  margin-top: 64px;
  text-align: center;
  font-size: 16px;
  line-height: 1.2;
  text-shadow: none;
}

.friends-overlay-opening .friends-book-shell,
.friends-overlay-opening .friends-mail-shell {
  animation: friends-overlay-in 260ms ease both;
}

.friends-overlay-closing .friends-book-shell,
.friends-overlay-closing .friends-mail-shell {
  animation: friends-overlay-out 260ms ease both;
}

@keyframes friends-overlay-in {
  from {
    opacity: 0;
    transform: translateY(32px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes friends-overlay-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(32px) scale(0.98);
  }
}
</style>
