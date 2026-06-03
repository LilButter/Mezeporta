<script setup>
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { assetUrl, store } from "../store";
import { playHover, playSelect } from "../sfx";

const props = defineProps({
  open: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  ready: {
    type: Boolean,
    default: false,
  },
  hasCharacter: {
    type: Boolean,
    default: false,
  },
  characterName: {
    type: String,
    default: "",
  },
  returning: {
    type: Boolean,
    default: false,
  },
  timePlayedFormatted: {
    type: String,
    default: "",
  },
  coursesList: {
    type: Array,
    default: () => [],
  },
  premiumCurrencies: {
    type: Object,
    default: () => ({}),
  },
  savedata: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(["close"]);

const ITEMBOX_MERGE_STORAGE_KEY = "mezeporta.book.itemBoxMergeEnabled";

function readBooleanPreference(key, fallback) {
  try {
    const value = window.localStorage.getItem(key);
    if (value === "true") return true;
    if (value === "false") return false;
  } catch {
    // Keep the in-memory fallback when localStorage is unavailable.
  }
  return fallback;
}

function writeBooleanPreference(key, value) {
  try {
    window.localStorage.setItem(key, value ? "true" : "false");
  } catch {
    // Best-effort preference persistence.
  }
}

const contentRoot = ref(null);
const searchQuery = ref("");
const isRendered = ref(Boolean(props.open));
const isOpening = ref(false);
const isClosing = ref(false);
const itemBoxListRef = ref(null);
const visibleItemBoxCount = ref(0);
const itemBoxLoadingMore = ref(false);
const itemBoxLoadCycle = ref(0);
const bookPageLoadCycle = ref(0);
const itemBoxMergeEnabled = ref(readBooleanPreference(ITEMBOX_MERGE_STORAGE_KEY, true));
const bookInfoPage = ref("summary");
const bookInfoTurnDirection = ref("");

const BOOK_ANIMATION_MS = 320;
const ITEMBOX_PAGE_SIZE = 20;
const ITEMBOX_REVEAL_DELAY_MS = 220;
const ITEMBOX_SNAP_DEBOUNCE_MS = 300;
const ITEMBOX_SNAP_AFTER_LOAD_MS = 160;
const BOOK_INFO_ANIMATION_MS = 220;
let bookAnimationTimer = null;
let bookInfoAnimationTimer = null;
let itemBoxLoadTimer = null;
let itemBoxSnapTimer = null;
let pendingItemBoxPostLoadDelta = 0;

function clearBookAnimationTimer() {
  if (bookAnimationTimer) {
    clearTimeout(bookAnimationTimer);
    bookAnimationTimer = null;
  }
}

function clearItemBoxLoadTimer() {
  if (itemBoxLoadTimer) {
    clearTimeout(itemBoxLoadTimer);
    itemBoxLoadTimer = null;
  }
}

function clearItemBoxSnapTimer() {
  if (itemBoxSnapTimer) {
    clearTimeout(itemBoxSnapTimer);
    itemBoxSnapTimer = null;
  }
}

function clearBookInfoAnimationTimer() {
  if (bookInfoAnimationTimer) {
    clearTimeout(bookInfoAnimationTimer);
    bookInfoAnimationTimer = null;
  }
}

function startBookOpenAnimation() {
  clearBookAnimationTimer();
  isRendered.value = true;
  isClosing.value = false;
  isOpening.value = false;

  nextTick(() => {
    requestAnimationFrame(() => {
      isOpening.value = true;
      bookAnimationTimer = setTimeout(() => {
        isOpening.value = false;
        bookAnimationTimer = null;
      }, BOOK_ANIMATION_MS);
    });
  });
}

function startBookCloseAnimation() {
  clearBookAnimationTimer();
  if (!isRendered.value) return;

  isOpening.value = false;
  isClosing.value = true;
  bookAnimationTimer = setTimeout(() => {
    isClosing.value = false;
    isRendered.value = false;
    bookAnimationTimer = null;
  }, BOOK_ANIMATION_MS);
}

watch(
  () => props.open,
  (open) => {
    if (!open) {
      searchQuery.value = "";
      visibleItemBoxCount.value = 0;
      itemBoxLoadingMore.value = false;
      bookInfoPage.value = "summary";
      bookInfoTurnDirection.value = "";
      clearItemBoxLoadTimer();
      clearBookInfoAnimationTimer();
    }
    if (open) {
      startBookOpenAnimation();
    } else {
      startBookCloseAnimation();
    }
  },
  { immediate: true }
);

watch(
  () => [props.open, props.loading],
  ([open, loading]) => {
    if (open && loading) {
      bookPageLoadCycle.value += 1;
    }
  },
  { immediate: true }
);

watch(
  () => props.characterName,
  () => {
    bookInfoPage.value = "summary";
    bookInfoTurnDirection.value = "";
  }
);

const currencies = computed(() => ({
  zenny: Number(props.savedata?.currencies?.zenny ?? 0),
  gzenny: Number(props.savedata?.currencies?.gzenny ?? 0),
  caravanPoints: Number(props.savedata?.currencies?.caravanPoints ?? 0),
}));

const premiumCurrencies = computed(() => ({
  koban: Number(props.premiumCurrencies?.gachaPremium ?? 0),
  trialKoban: Number(props.premiumCurrencies?.gachaTrial ?? 0),
  frontierPoints: Number(props.premiumCurrencies?.frontierPoints ?? 0),
}));

const bookExtraCurrencyEntries = computed(() => [
  { key: "koban", label: "Shiny Koban Coin G", value: premiumCurrencies.value.koban },
  { key: "trialKoban", label: "Trial Koban Coin G", value: premiumCurrencies.value.trialKoban },
  { key: "frontierPoints", label: "Frontier Points", value: premiumCurrencies.value.frontierPoints },
].filter((entry) => Number(entry.value) > 0));

const bookCurrencyEntries = computed(() => [
  { key: "zenny", label: "Zenny", value: currencies.value.zenny },
  { key: "gzenny", label: "Gz", value: currencies.value.gzenny },
  { key: "caravanPoints", label: "Caravan Points", value: currencies.value.caravanPoints },
].filter((entry) => Number(entry.value) > 0));

function formatCurrencyEntry(entry) {
  return `${entry.label}: ${formatPanelNumber(entry.value)}`;
}

const bookCharacterName = computed(() => {
  const name = String(props.characterName ?? "").trim();
  return name || "Character";
});

const bookCharacterDisplayName = computed(() => {
  return bookCharacterName.value;
});

const bookTimePlayed = computed(() => {
  return String(props.timePlayedFormatted ?? "");
});

const bookCoursesList = computed(() => {
  const courses = props.coursesList;
  if (!Array.isArray(courses) || !courses.length) return [];
  return [...courses].sort((a, b) => Number(a?.id ?? 0) - Number(b?.id ?? 0));
});

const bookCourseColumns = computed(() => {
  const columns = [];
  const courses = bookCoursesList.value;
  for (let index = 0; index < courses.length; index += 5) {
    columns.push(courses.slice(index, index + 5));
  }
  return columns;
});

const bookInfoPages = computed(() => {
  const pages = ["summary"];
  if (bookCoursesList.value.length) {
    pages.push("courses");
  }
  if (bookExtraCurrencyEntries.value.length) {
    pages.push("extra");
  }
  return pages;
});

watch(
  () => bookCoursesList.value.length,
  (courseCount) => {
    if (courseCount <= 0 && bookInfoPage.value === "courses") {
      bookInfoPage.value = "summary";
      bookInfoTurnDirection.value = "";
    }
  }
);

watch(
  () => bookExtraCurrencyEntries.value.length,
  (extraCount) => {
    if (extraCount <= 0 && bookInfoPage.value === "extra") {
      bookInfoPage.value = "summary";
      bookInfoTurnDirection.value = "";
    }
  }
);

const gearEntries = computed(() => {
  const entries = props.savedata?.gear;
  if (!Array.isArray(entries)) return [];
  return [...entries].sort((left, right) => {
    const leftWeapon = String(left?.slotLabel ?? "").trim().toLowerCase() === "weapon";
    const rightWeapon = String(right?.slotLabel ?? "").trim().toLowerCase() === "weapon";
    if (leftWeapon === rightWeapon) return 0;
    return leftWeapon ? -1 : 1;
  });
});

const itemBoxEntries = computed(() => {
  const entries = props.savedata?.itemBox;
  return Array.isArray(entries) ? entries : [];
});

const normalizedSearchQuery = computed(() =>
  String(searchQuery.value ?? "").trim().toLowerCase()
);

const filteredItemBoxEntries = computed(() => {
  const query = normalizedSearchQuery.value;
  if (!query) return itemBoxEntries.value;

  const idQuery = query.replace(/^0x/i, "");
  return itemBoxEntries.value.filter((item) => {
    const name = String(item?.name ?? "").toLowerCase();
    const itemId = String(item?.id ?? "").toLowerCase();
    const trimmedItemId = itemId.replace(/^0+/, "");
    return (
      name.includes(query) ||
      itemId.includes(idQuery) ||
      trimmedItemId.includes(idQuery)
    );
  });
});

const mergedItemBoxEntries = computed(() => {
  const map = new Map();
  for (const item of filteredItemBoxEntries.value) {
    const key = String(item?.id ?? "");
    const existing = map.get(key);
    if (existing) {
      existing.quantity += Number(item?.quantity ?? 0);
      if (!existing.icon && item?.icon) {
        existing.icon = item.icon;
      }
      if (!existing.name && item?.name) {
        existing.name = item.name;
      }
    } else {
      map.set(key, { ...item, quantity: Number(item?.quantity ?? 0) });
    }
  }
  return Array.from(map.values());
});

const displayedItemBoxEntries = computed(() =>
  itemBoxMergeEnabled.value ? mergedItemBoxEntries.value : filteredItemBoxEntries.value
);

const visibleItemBoxEntries = computed(() =>
  displayedItemBoxEntries.value.slice(0, visibleItemBoxCount.value)
);

const hasMoreItemBoxEntries = computed(
  () => visibleItemBoxCount.value < displayedItemBoxEntries.value.length
);
const itemBoxInitialLoading = computed(
  () => itemBoxLoadingMore.value && visibleItemBoxCount.value <= 0
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

function beginItemBoxLoadCycle() {
  itemBoxLoadCycle.value += 1;
  itemBoxLoadingMore.value = true;
}

function queueInitialItemBoxReveal() {
  clearItemBoxLoadTimer();
  clearItemBoxSnapTimer();
  pendingItemBoxPostLoadDelta = 0;
  if (!props.open || !props.ready || !props.savedata) {
    visibleItemBoxCount.value = 0;
    itemBoxLoadingMore.value = false;
    return;
  }

  beginItemBoxLoadCycle();
  visibleItemBoxCount.value = 0;
  itemBoxLoadTimer = setTimeout(() => {
    visibleItemBoxCount.value = Math.min(
      ITEMBOX_PAGE_SIZE,
      displayedItemBoxEntries.value.length
    );
    itemBoxLoadingMore.value = false;
    itemBoxLoadTimer = null;
    if (itemBoxListRef.value) {
      itemBoxListRef.value.scrollTop = 0;
    }
  }, ITEMBOX_REVEAL_DELAY_MS);
}

function queueMoreItemBoxEntries() {
  if (itemBoxLoadingMore.value || !hasMoreItemBoxEntries.value) return;
  clearItemBoxLoadTimer();
  clearItemBoxSnapTimer();
  beginItemBoxLoadCycle();
  itemBoxLoadTimer = setTimeout(() => {
    visibleItemBoxCount.value = Math.min(
      displayedItemBoxEntries.value.length,
      visibleItemBoxCount.value + ITEMBOX_PAGE_SIZE
    );
    itemBoxLoadingMore.value = false;
    itemBoxLoadTimer = null;
    nextTick(() => {
      const target = itemBoxListRef.value;
      if (target && pendingItemBoxPostLoadDelta > 0) {
        const step = itemBoxEntryStep(target);
        const maxTop = Math.max(0, target.scrollHeight - target.clientHeight);
        target.scrollTop = Math.min(maxTop, target.scrollTop + step);
      }
      pendingItemBoxPostLoadDelta = 0;
      scheduleItemBoxSnap(ITEMBOX_SNAP_AFTER_LOAD_MS);
    });
  }, 3000);
}

function itemBoxEntryStep(target) {
  const entries = [...target.querySelectorAll(".book-overlay-entry")];
  if (entries.length > 1) {
    const firstTop = entries[0].getBoundingClientRect().top;
    const secondTop = entries[1].getBoundingClientRect().top;
    const step = Math.abs(secondTop - firstTop);
    if (step > 0) return step;
  }
  const firstEntry = entries[0];
  return firstEntry ? Math.max(firstEntry.getBoundingClientRect().height, 30) : 30;
}

function snapItemBoxToClosestEntry() {
  const target = itemBoxListRef.value;
  if (!target || itemBoxLoadingMore.value) return;

  const entries = [...target.querySelectorAll(".book-overlay-entry")];
  if (!entries.length) return;

  const currentTop = target.scrollTop;
  const containerTop = target.getBoundingClientRect().top;
  const entryScrollTop = (entry) =>
    entry.getBoundingClientRect().top - containerTop + target.scrollTop;
  let closestTop = entryScrollTop(entries[0]);
  let closestDistance = Math.abs(closestTop - currentTop);

  for (const entry of entries) {
    const entryTop = entryScrollTop(entry);
    const distance = Math.abs(entryTop - currentTop);
    if (distance < closestDistance) {
      closestTop = entryTop;
      closestDistance = distance;
    }
  }

  const maxTop = Math.max(0, target.scrollHeight - target.clientHeight);
  if (closestDistance < 1) return;
  target.scrollTop = Math.max(0, Math.min(maxTop, closestTop));
}

function scheduleItemBoxSnap(delay = ITEMBOX_SNAP_DEBOUNCE_MS) {
  clearItemBoxSnapTimer();
  if (itemBoxLoadingMore.value) return;
  itemBoxSnapTimer = setTimeout(() => {
    itemBoxSnapTimer = null;
    snapItemBoxToClosestEntry();
  }, delay);
}

function onItemBoxWheel(event) {
  const target = itemBoxListRef.value;
  if (!target) return;

  let delta = Number(event?.deltaY ?? 0);
  if (!delta) return;
  if (event?.deltaMode === 1) {
    delta *= 16;
  } else if (event?.deltaMode === 2) {
    delta *= target.clientHeight;
  }

  const maxTop = Math.max(0, target.scrollHeight - target.clientHeight);
  if (maxTop <= 0) return;

  if (event.cancelable !== false) {
    event.preventDefault();
  }
  const step = itemBoxEntryStep(target);
  const direction = delta > 0 ? 1 : -1;
  if (direction < 0) {
    pendingItemBoxPostLoadDelta = 0;
  }
  target.scrollTop = Math.max(0, Math.min(maxTop, target.scrollTop + direction * step));

  const nearBottom =
    target.scrollTop + target.clientHeight >= target.scrollHeight - 56;
  if (nearBottom && hasMoreItemBoxEntries.value) {
    if (direction > 0) {
      pendingItemBoxPostLoadDelta = Math.max(pendingItemBoxPostLoadDelta, step);
    }
    queueMoreItemBoxEntries();
  }
  if (!itemBoxLoadingMore.value) {
    scheduleItemBoxSnap();
  }
}

function onItemBoxScroll(event) {
  const target = event?.target;
  if (!target) return;
  const nearBottom =
    target.scrollTop + target.clientHeight >= target.scrollHeight - 56;
  if (nearBottom && hasMoreItemBoxEntries.value) {
    queueMoreItemBoxEntries();
  }
}

watch(
  () => [props.open, props.ready, props.loading, itemBoxEntries.value.length, normalizedSearchQuery.value, itemBoxMergeEnabled.value],
  ([open, ready, loading]) => {
    if (!open || loading || !ready) {
      clearItemBoxLoadTimer();
      visibleItemBoxCount.value = 0;
      itemBoxLoadingMore.value = false;
      return;
    }
    queueInitialItemBoxReveal();
  },
  { immediate: true }
);

function formatPanelNumber(value) {
  const numeric = Number(value ?? 0);
  if (!Number.isFinite(numeric)) return "0";
  return numeric.toLocaleString();
}

function formatDecorationName(value) {
  return String(value ?? "").replace(/\s+Deco$/i, "").trim();
}

function showBookInfoPage(page) {
  if (bookInfoPage.value === page) return;
  const pages = bookInfoPages.value;
  if (!pages.includes(page)) return;
  clearBookInfoAnimationTimer();
  playSelect();
  const previousIndex = pages.indexOf(bookInfoPage.value);
  const nextIndex = pages.indexOf(page);
  bookInfoTurnDirection.value = nextIndex > previousIndex ? "next" : "prev";
  bookInfoPage.value = page;
  bookInfoAnimationTimer = setTimeout(() => {
    bookInfoTurnDirection.value = "";
    bookInfoAnimationTimer = null;
  }, BOOK_INFO_ANIMATION_MS);
}

function showNextBookInfoPage() {
  const pages = bookInfoPages.value;
  const currentIndex = pages.indexOf(bookInfoPage.value);
  if (currentIndex < 0 || currentIndex >= pages.length - 1) return;
  showBookInfoPage(pages[currentIndex + 1]);
}

function showPreviousBookInfoPage() {
  const pages = bookInfoPages.value;
  const currentIndex = pages.indexOf(bookInfoPage.value);
  if (currentIndex <= 0) return;
  showBookInfoPage(pages[currentIndex - 1]);
}

function toggleItemBoxMerge() {
  itemBoxMergeEnabled.value = !itemBoxMergeEnabled.value;
  writeBooleanPreference(ITEMBOX_MERGE_STORAGE_KEY, itemBoxMergeEnabled.value);
  playSelect();
}

function onCloseClick() {
  playSelect();
  emit("close");
}

onBeforeUnmount(() => {
  clearBookAnimationTimer();
  clearBookInfoAnimationTimer();
  clearItemBoxLoadTimer();
  clearItemBoxSnapTimer();
});

defineExpose({
  contentRoot,
});
</script>

<template>
  <div
    v-if="isRendered"
    class="book-overlay"
    :class="{
      'book-overlay-opening': isOpening,
      'book-overlay-closing': isClosing,
    }"
    @click.self="onCloseClick"
  >
    <div
      class="book-overlay-backdrop"
      :class="{ 'book-overlay-backdrop-static': store.settings.linuxHardwareAcceleration === false }"
      :style="overlayBackdropStyle"
    ></div>
    <div
      ref="contentRoot"
      class="book-overlay-dialog"
      data-controller-scope="book"
    >
      <button
        type="button"
        class="book-overlay-close font-main"
        data-controller-size="big"
        @mouseenter="playHover()"
        @click="onCloseClick"
      >
        X
      </button>

      <div class="book-overlay-page book-overlay-page-left">
        <template v-if="!hasCharacter">
          <div class="book-overlay-empty font-main">No character selected.</div>
        </template>
        <template v-else-if="loading">
          <div class="book-overlay-page-loader">
            <img
              :key="`book-left-loading-${bookPageLoadCycle}`"
              :src="assetUrl('/extra/ItemBoxLoad.gif')"
              class="book-overlay-page-loader-image"
              draggable="false"
              alt=""
            />
          </div>
        </template>
        <template v-else-if="ready && savedata">
          <div
            class="book-overlay-top-panel font-main"
            :class="{
              'book-overlay-top-panel-turn-next': bookInfoTurnDirection === 'next',
              'book-overlay-top-panel-turn-prev': bookInfoTurnDirection === 'prev',
            }"
          >
            <div :key="bookInfoPage" class="book-overlay-top-panel-content">
              <template v-if="bookInfoPage === 'summary'">
                <div class="book-overlay-charinfo-name">{{ bookCharacterDisplayName }}</div>
                <div class="book-overlay-timeplayed">Time played: {{ bookTimePlayed }}</div>
                <div v-if="returning" class="book-overlay-returning-label">(Returning)</div>
                <div class="book-overlay-info-row">
                  <div class="book-overlay-currency-list">
                    <div
                      v-for="entry in bookCurrencyEntries"
                      :key="`book-currency-${entry.key}`"
                      class="book-overlay-currency-value"
                    >
                      {{ formatCurrencyEntry(entry) }}
                    </div>
                  </div>
                </div>
                <button
                  v-if="bookInfoPages.length > 1"
                  type="button"
                  class="book-overlay-top-page-button book-overlay-top-page-button-next"
                  aria-label="Show next info page"
                  @mouseenter="playHover()"
                  @click="showNextBookInfoPage"
                >
                  <img :src="assetUrl('/extra/booknext.png')" draggable="false" alt="" />
                </button>
              </template>
              <template v-else-if="bookInfoPage === 'courses'">
                <div class="book-overlay-courses-label">Courses</div>
                <div class="book-overlay-courses-grid">
                  <div
                    v-for="(column, columnIndex) in bookCourseColumns"
                    :key="`book-course-column-${columnIndex}`"
                    class="book-overlay-courses-column"
                  >
                    <div
                      v-for="(course, courseIndex) in column"
                      :key="`book-course-${columnIndex}-${courseIndex}`"
                      class="book-overlay-courses-item"
                    >
                      {{ course.name }} (#{{ course.id }})
                    </div>
                  </div>
                </div>
                <button
                  type="button"
                  class="book-overlay-top-page-button book-overlay-top-page-button-prev"
                  aria-label="Show previous info page"
                  @mouseenter="playHover()"
                  @click="showPreviousBookInfoPage"
                >
                  <img :src="assetUrl('/extra/bookprev.png')" draggable="false" alt="" />
                </button>
                <button
                  v-if="bookInfoPages.indexOf(bookInfoPage) < bookInfoPages.length - 1"
                  type="button"
                  class="book-overlay-top-page-button book-overlay-top-page-button-next"
                  aria-label="Show next info page"
                  @mouseenter="playHover()"
                  @click="showNextBookInfoPage"
                >
                  <img :src="assetUrl('/extra/booknext.png')" draggable="false" alt="" />
                </button>
              </template>
              <template v-else>
                <div class="book-overlay-courses-label">Extra</div>
                <div class="book-overlay-extra-list">
                  <div
                    v-for="entry in bookExtraCurrencyEntries"
                    :key="`book-extra-${entry.key}`"
                    class="book-overlay-extra-item"
                  >
                    {{ formatCurrencyEntry(entry) }}
                  </div>
                </div>
                <button
                  type="button"
                  class="book-overlay-top-page-button book-overlay-top-page-button-prev"
                  aria-label="Show previous info page"
                  @mouseenter="playHover()"
                  @click="showPreviousBookInfoPage"
                >
                  <img :src="assetUrl('/extra/bookprev.png')" draggable="false" alt="" />
                </button>
              </template>
            </div>
          </div>

          <div class="book-overlay-section font-main">Current Gear</div>
          <div class="book-overlay-list book-overlay-list-left scrollbar">
            <div
              v-for="(equip, equipIndex) in gearEntries"
              :key="`book-gear-${equip.id}-${equipIndex}`"
              class="book-overlay-entry book-overlay-gear-entry"
            >
              <img
                :src="equip.icon"
                class="book-overlay-entry-icon"
                draggable="false"
                @error="e => (e.target.src = assetUrl('/extra/items/Dummy.png'))"
              />
              <div class="book-overlay-entry-text">
                <div class="book-overlay-entry-meta font-main">
                  {{ equip.slotLabel }} | ID {{ equip.id }} | Lv {{ Number(equip.upgradeLevel ?? 0) + 1 }}
                  <template v-if="equip.weaponType"> | {{ equip.weaponType }}</template>
                </div>
                <div class="book-overlay-entry-name font-main">
                  {{ equip.name }}
                </div>
                <div
                  v-if="Array.isArray(equip.decorations) && equip.decorations.length"
                  class="book-overlay-decoration-list"
                >
                  <span
                    v-for="(decoration, decorationIndex) in equip.decorations"
                    :key="`book-decoration-${equip.id}-${decoration.id}-${decorationIndex}`"
                    class="book-overlay-decoration-chip font-main"
                    :title="decoration.name"
                  >
                    <img
                      :src="decoration.icon"
                      class="book-overlay-decoration-icon"
                      draggable="false"
                      alt=""
                      @error="e => (e.target.src = assetUrl('/extra/items/Dummy.png'))"
                    />
                    <span
                      class="book-overlay-decoration-name"
                      :class="{ 'book-overlay-decoration-name-scroll': formatDecorationName(decoration.name).length > 12 }"
                    >
                      {{ formatDecorationName(decoration.name) }}
                    </span>
                  </span>
                </div>
                <div v-else class="book-overlay-decoration-list book-overlay-decoration-list-empty"></div>
              </div>
            </div>
            <div v-if="!gearEntries.length" class="book-overlay-empty font-main">
              No gear data.
            </div>
          </div>
        </template>
        <template v-else>
          <div class="book-overlay-empty font-main">Unable to load savedata.</div>
        </template>
      </div>

      <div class="book-overlay-page book-overlay-page-right">
        <template v-if="!hasCharacter">
          <div class="book-overlay-empty font-main">No character selected.</div>
        </template>
        <template v-else-if="loading">
          <div class="book-overlay-page-loader">
            <img
              :key="`book-right-loading-${bookPageLoadCycle}`"
              :src="assetUrl('/extra/ItemBoxLoad.gif')"
              class="book-overlay-page-loader-image"
              draggable="false"
              alt=""
            />
          </div>
        </template>
        <template v-else-if="ready && savedata">
          <div class="book-overlay-section font-main">Item Box</div>
          <div class="book-overlay-search-shell">
            <input
              v-model="searchQuery"
              type="text"
              spellcheck="false"
              placeholder="Search by item name or ID"
              class="book-overlay-search font-main"
              data-controller-size="big"
            />
          </div>
          <div class="book-overlay-count font-main">
            <span class="book-overlay-count-left">
              <span>
                Total entries: {{ formatPanelNumber(itemBoxEntries.length) }}
                <template v-if="normalizedSearchQuery">
                  | Matches: {{ formatPanelNumber(filteredItemBoxEntries.length) }}
                </template>
              </span>
              <span v-if="itemBoxLoadingMore && visibleItemBoxCount > 0" class="book-overlay-count-loader">
                <img
                  :key="`itembox-load-${itemBoxLoadCycle}`"
                  :src="assetUrl('/extra/ItemBoxLoad.gif')"
                  class="book-overlay-count-loader-image"
                  draggable="false"
                  alt=""
                />
              </span>
            </span>
            <button
              type="button"
              class="book-overlay-merge-toggle"
              :class="{ 'book-overlay-merge-toggle-on': itemBoxMergeEnabled }"
              :aria-pressed="itemBoxMergeEnabled"
              title="Merge item quantities"
              @mouseenter="playHover()"
              @click="toggleItemBoxMerge"
            >
              <span class="book-overlay-merge-toggle-label">Merge slots</span>
              <span class="book-overlay-merge-toggle-track">
                <span class="book-overlay-merge-toggle-thumb"></span>
              </span>
            </button>
          </div>
          <div
            ref="itemBoxListRef"
            class="book-overlay-list book-overlay-list-right scrollbar book-overlay-itembox-scroll"
            @scroll="onItemBoxScroll"
            @wheel="onItemBoxWheel"
          >
            <div
              v-for="(item, itemIndex) in visibleItemBoxEntries"
              :key="`book-item-${item.id}-${item.slot}-${itemIndex}`"
              class="book-overlay-entry"
            >
              <img
                :src="item.icon"
                class="book-overlay-entry-icon"
                draggable="false"
                @error="e => (e.target.src = assetUrl('/extra/items/Dummy.png'))"
              />
              <div class="book-overlay-entry-text">
                <div class="book-overlay-entry-title font-main">{{ item.name }}</div>
                <div class="book-overlay-entry-meta font-main">
                  ID {{ item.id }}
                  <template v-if="!itemBoxMergeEnabled"> | Slot {{ item.slot }}</template>
                  | x{{ formatPanelNumber(item.quantity) }}
                </div>
              </div>
            </div>
            <div
              v-if="!itemBoxLoadingMore && itemBoxEntries.length && !displayedItemBoxEntries.length && normalizedSearchQuery"
              class="book-overlay-empty font-main"
            >
              No items match your search.
            </div>
            <div
              v-else-if="!itemBoxLoadingMore && !itemBoxEntries.length"
              class="book-overlay-empty font-main"
            >
              No items in box.
            </div>
          </div>
        </template>
        <template v-else>
          <div class="book-overlay-empty font-main">Unable to load savedata.</div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.book-overlay {
  position: absolute;
  inset: 0;
  z-index: 120;
  display: flex;
  align-items: center;
  justify-content: center;
}

.book-overlay-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(3px) brightness(0.45);
}

.book-overlay-backdrop-static {
  background: radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%);
}

.book-overlay-dialog {
  position: relative;
  z-index: 1;
  width: 760px;
  height: 570px;
  background-image: url("/extra/MainBook.png");
  background-position: center;
  background-repeat: no-repeat;
  background-size: contain;
  transform-origin: center bottom;
}

.book-overlay-close {
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

.book-overlay-page {
  position: absolute;
  top: 48px;
  bottom: 54px;
  width: 298px;
  color: #3f2d1f;
  text-shadow: none;
}

.book-overlay-page-left {
  left: 42px;
  top: 18px;
}

.book-overlay-page-right {
  right: 42px;
  top: 40px;
}

.book-overlay-section {
  margin-bottom: 10px;
  font-size: 1.1rem;
  line-height: 1.1;
  font-weight: 700;
  text-shadow: none;
}

.book-overlay-top-panel {
  position: relative;
  height: 116px;
  margin: 0 0 2px;
  padding: 0 34px;
  text-align: center;
  text-shadow: none;
}

.book-overlay-top-panel-content {
  position: relative;
  height: 100%;
}

.book-overlay-top-panel-turn-next .book-overlay-top-panel-content {
  animation: bookTopPanelNext 220ms ease both;
}

.book-overlay-top-panel-turn-prev .book-overlay-top-panel-content {
  animation: bookTopPanelPrev 220ms ease both;
}

.book-overlay-timeplayed {
  text-align: center;
  margin: 0 0 2px;
  font-size: 0.9rem;
  line-height: 1.15;
  text-shadow: none;
  padding-left: 0;
  width: 100%;
}

.book-overlay-charinfo-row {
  display: flex;
  justify-content: flex-start;
  align-items: baseline;
  margin-bottom: 6px;
  font-size: 1rem;
  line-height: 1.2;
  text-shadow: none;
}

.book-overlay-charinfo-name {
  font-size: 1.1rem;
  font-weight: 700;
  line-height: 1.15;
  margin-bottom: 2px;
  text-align: center;
  text-shadow: none;
}

.book-overlay-returning-label {
  margin: 0 0 4px;
  font-size: 0.78rem;
  line-height: 1.05;
  text-align: center;
  color: #6c4a30;
  text-shadow: none;
}

.book-overlay-info-row {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  margin: 0;
}

.book-overlay-courses-grid {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: max-content;
  justify-content: center;
  align-items: start;
  column-gap: 16px;
  max-width: 218px;
  margin: 0 auto;
  line-height: 1.16;
  text-shadow: none;
}

.book-overlay-courses-column {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
}

.book-overlay-courses-item {
  white-space: nowrap;
  font-size: 0.8rem;
  text-align: center;
}

.book-overlay-courses-label {
  font-weight: 700;
  font-size: 1.1rem;
  margin-bottom: 2px;
  text-align: center;
}

.book-overlay-courses-value {
  color: #6c4a30;
  word-break: break-word;
}

.book-overlay-currency-list,
.book-overlay-extra-list {
  width: 236px;
  max-width: 100%;
  margin: 2px auto 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  line-height: 1.16;
  text-shadow: none;
}

.book-overlay-extra-list {
  margin-top: 3px;
}

.book-overlay-extra-item {
  width: 100%;
  font-size: 0.8rem;
  line-height: 1.16;
  text-align: center;
  white-space: nowrap;
}

.book-overlay-top-page-button {
  position: absolute;
  width: 48px;
  height: 48px;
  padding: 0;
  border: 0;
  background: transparent;
  cursor: pointer;
  border-radius: 999px;
  transition: filter 140ms ease, transform 140ms ease;
}

.book-overlay-top-page-button:hover,
.book-overlay-top-page-button:focus-visible,
.book-overlay-top-page-button.controller-nav-focused {
  filter: brightness(1.12);
  transform: scale(1.06);
}

.book-overlay-top-page-button img {
  width: 44px;
  height: 44px;
  object-fit: contain;
  pointer-events: none;
}

.book-overlay-top-page-button-next {
  right: -54px;
  top: 42px;
}

.book-overlay-top-page-button-prev {
  left: -54px;
  top: 42px;
}

.book-overlay-currency-value {
  width: 100%;
  max-width: 100%;
  min-width: 0;
  line-height: 1.05;
  font-size: 0.8rem;
  white-space: nowrap;
  text-align: center;
}

.book-overlay-search-shell {
  margin-bottom: 8px;
}

.book-overlay-search {
  width: 100%;
  border: 0;
  border-bottom: 1px solid rgba(71, 48, 33, 0.65);
  background: transparent;
  color: #3f2d1f;
  font-size: 13px;
  line-height: 1.2;
  padding: 4px 0;
  outline: none;
  text-shadow: none;
}

.book-overlay-search::placeholder {
  color: rgba(63, 45, 31, 0.7);
}

.book-overlay-count {
  margin-bottom: 10px;
  font-size: 0.8rem;
  font-weight: 700;
  text-shadow: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.book-overlay-count-left {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  gap: 6px;
}

.book-overlay-count-loader {
  display: inline-flex;
  align-items: center;
}

.book-overlay-count-loader-image {
  width: 56px;
  height: auto;
  object-fit: contain;
}

.book-overlay-merge-toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  flex: 0 0 auto;
  padding: 0;
  border: 0;
  background: transparent;
  color: #3f2d1f;
  font: inherit;
  line-height: 1;
  cursor: pointer;
  text-shadow: none;
}

.book-overlay-merge-toggle-label {
  font-size: 0.68rem;
}

.book-overlay-merge-toggle-track {
  position: relative;
  width: 28px;
  height: 15px;
  border: 1px solid rgba(79, 53, 31, 0.56);
  border-radius: 999px;
  background: rgba(91, 63, 37, 0.24);
  box-shadow: inset 0 1px 0 rgba(255, 244, 200, 0.25);
}

.book-overlay-merge-toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: rgba(248, 232, 188, 0.96);
  border: 1px solid rgba(80, 44, 12, 0.48);
  transition: transform 120ms ease, background 120ms ease;
}

.book-overlay-merge-toggle-on .book-overlay-merge-toggle-thumb {
  transform: translateX(13px);
  background: linear-gradient(135deg, #fbe7a6 0%, #f0b949 58%, #d99525 100%);
}

.book-overlay-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-width: none;
}

.book-overlay-list::-webkit-scrollbar {
  display: none;
}

.book-overlay-list-left {
  position: relative;
  top: 0px;
  height: 350px;
  overflow: hidden;
  justify-content: flex-start;
  gap: 12px;
}

.book-overlay-list-right {
  height: 380px;
  margin-top: 9px;
  transform: translateY(-8px);
}

.book-overlay-itembox-scroll {
  scroll-behavior: auto;
  overflow-y: auto;
}

.book-overlay-itembox-scroll::-webkit-scrollbar {
  display: none;
}

.book-overlay-entry {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: 7px;
  align-items: start;
  min-height: 30px;
}

.book-overlay-gear-entry {
  grid-template-columns: 30px minmax(0, 1fr);
  gap: 4px;
  min-height: 0;
}

.book-overlay-page-left .book-overlay-section {
  position: relative;
  bottom: 5px;
  margin-bottom: 12px;
}

.book-overlay-itembox-scroll .book-overlay-entry {
  min-height: 30px;
}

.book-overlay-entry-icon {
  width: 22px;
  height: 22px;
  object-fit: contain;
  margin-top: 2px;
}

.book-overlay-gear-entry .book-overlay-entry-icon {
  width: 40px;
  height: 40px;
  align-self: start;
  margin-top: 0px;
}

.book-overlay-entry-text {
  min-width: 0;
}

.book-overlay-entry-title {
  font-size: 1rem;
  line-height: 1.15;
  text-shadow: none;
}

.book-overlay-entry-name {
  font-size: 1rem;
  line-height: 1.08;
  text-shadow: none;
}

.book-overlay-gear-entry .book-overlay-entry-name {
  font-size: 1rem;
  line-height: 1.06;
}

.book-overlay-decoration-list {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  align-items: center;
  column-gap: 3px;
  row-gap: 0;
  margin: 1px 0 0;
  min-width: 0;
  overflow: hidden;
}

.book-overlay-decoration-list-empty {
  visibility: hidden;
  min-height: 16px;
}

.book-overlay-decoration-chip {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  min-width: 0;
  max-width: none;
  overflow: hidden;
  color: #3f2d1f;
  font-size: 0.7rem;
  line-height: 1.06;
  text-shadow: none;
}

.book-overlay-decoration-icon {
  width: 16px;
  height: 16px;
  flex: 0 0 auto;
  object-fit: contain;
}

.book-overlay-decoration-name {
  display: inline-block;
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: clip;
  white-space: nowrap;
}

.book-overlay-decoration-name-scroll {
  padding-right: 8px;
  animation: bookDecoTicker 4.8s ease-in-out infinite alternate;
}

@keyframes bookDecoTicker {
  from {
    transform: translateX(0);
  }

  to {
    transform: translateX(-35%);
  }
}

@keyframes bookTopPanelNext {
  from {
    opacity: 0;
    transform: translateX(16px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes bookTopPanelPrev {
  from {
    opacity: 0;
    transform: translateX(-16px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.book-bold {
  font-weight: 700;
}

.book-overlay-entry-meta {
  font-size: 0.7rem;
  line-height: 1.12;
  text-shadow: none;
}

.book-overlay-gear-entry .book-overlay-entry-meta {
  font-size: 0.7rem;
  line-height: 1.1;
  color: #3f2d1f;
}

.book-overlay-empty {
  font-size: 16px;
  line-height: 1.2;
  text-shadow: none;
}

.book-overlay-page-loader {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.book-overlay-page-loader-image {
  width: 96px;
  height: auto;
  object-fit: contain;
}

.book-overlay-itembox-loader {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 10px 0 4px;
}

.book-overlay-itembox-loader-centered {
  min-height: 100%;
  padding: 0;
}

.book-overlay-itembox-loader-image {
  width: 76px;
  height: auto;
  object-fit: contain;
}

@keyframes book-overlay-bounce-in {
  0% {
    opacity: 0;
    transform: translateY(84px) scale(0.94);
  }
  62% {
    opacity: 1;
    transform: translateY(-10px) scale(1.015);
  }
  78% {
    transform: translateY(4px) scale(0.995);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes book-overlay-bounce-out {
  0% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  18% {
    transform: translateY(-6px) scale(1.01);
  }
  100% {
    opacity: 0;
    transform: translateY(84px) scale(0.94);
  }
}

.book-overlay-opening .book-overlay-dialog {
  animation: book-overlay-bounce-in 320ms cubic-bezier(0.18, 0.9, 0.28, 1.2) both;
}

.book-overlay-closing .book-overlay-dialog {
  animation: book-overlay-bounce-out 320ms cubic-bezier(0.4, 0, 0.65, 1) both;
}

.book-overlay-opening .book-overlay-backdrop {
  animation: book-overlay-fade-in 320ms ease both;
}

.book-overlay-closing .book-overlay-backdrop {
  animation: book-overlay-fade-out 320ms ease both;
}

@keyframes book-overlay-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes book-overlay-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
</style>
