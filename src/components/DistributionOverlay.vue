<script setup>
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { assetUrl, store } from "../store";
import { getItemDisplayMeta } from "../altclient/savedataView";
import { playHover, playSelect } from "../sfx";
import renamedItemsRaw from "../../renamed_items.txt?raw";
import "./DistributionOverlay.css";

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
  hasMore: {
    type: Boolean,
    default: false,
  },
  loadingMore: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["close", "load-more"]);

const GUIDE_BUBBLE_STORAGE_KEY = "mezeporta.distribution.guideBubbleEnabled";

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
const isRendered = ref(Boolean(props.open));
const isOpening = ref(false);
const isClosing = ref(false);
const pageIndex = ref(0);
const turnDirection = ref("");
const flippedEntryIds = ref(new Set());
const guideBubbleEnabled = ref(readBooleanPreference(GUIDE_BUBBLE_STORAGE_KEY, true));
const showGuideBubble = computed(() => guideBubbleEnabled.value && flippedEntryIds.value.size > 0);
const distributionAutoScrollEnabled = ref(true);
const itemMetaById = ref({});
const missingRewardIconKeys = ref(new Set());
const pendingAdvanceAfterLoad = ref(false);
const nameScrollMetrics = ref({});
const descScrollMetrics = ref({});
const rewardScrollMetrics = ref({});
const nameMeasureFrame = ref(0);
const descMeasureFrame = ref(0);
const rewardMeasureFrame = ref(0);

const OVERLAY_ANIMATION_MS = 280;
const PAGE_ANIMATION_MS = 260;
const PAGE_SIZE = 3;
const DISTRIBUTION_COLOR_CODE_PATTERN = /~C(\d{2})/gi;
const DISTRIBUTION_COLOR_CODES = {
  "01": "#323232",
  "02": "#FF435D",
  "03": "#56FF56",
  "04": "#57FFFF",
  "05": "#FFFF50",
  "06": "#FFA461",
  "07": "#FF84FF",
  "08": "#BF6464",
  "09": "#A0A0A0",
  "10": "#808080",
  "11": "#F08200",
  "12": "#846B5C",
  "13": "#80212E",
  "14": "#747EFF",
  "15": "#FF9ECA",
  "16": "#4040FF",
  "17": "#202020",
  "18": "#602020",
  "19": "#32BC64",
  "20": "#001480",
  "21": "#3E9DD8",
  "22": "#72D242",
  "23": "#B4641E",
  "24": "#32BC64",
  "25": "#4040FF",
  "26": "#68ECEC",
  "27": "#C8FF6A",
  "28": "#CBA6FA",
  "29": "#96B5FD",
  "30": "#808028",
  "31": "#640011",
  "32": "#FFFFFF",
  "33": "#000000",
  "34": "#FF435D",
  "35": "#56FF56",
  "36": "#57FFFF",
  "37": "#FFFF50",
  "38": "#FFA461",
  "39": "#FF84FF",
  "40": "#BF6464",
  "41": "#A0A0A0",
  "42": "#808080",
  "43": "#F08200",
  "44": "#846B5C",
  "45": "#80212E",
  "46": "#2020A0",
  "47": "#FF9ECA",
  "48": "#4040FF",
  "49": "#202020",
  "50": "#602020",
  "51": "#32BC64",
  "52": "#001480",
  "53": "#3E9DD8",
  "54": "#72D242",
  "55": "#B4641E",
  "56": "#32BC64",
  "57": "#4040FF",
  "58": "#68ECEC",
  "59": "#C8FF6A",
  "60": "#CBA6FA",
  "61": "#96B5FD",
  "62": "#808028",
  "63": "#640011",
  "64": "#684B02",
  "65": "#014517",
  "66": "#014066",
  "67": "#C25900",
  "68": "#4040FF",
  "69": "#1F974F",
  "70": "#A3A488",
  "71": "#1C821C",
  "72": "#CC5400",
  "73": "#660066",
  "74": "#000000",
  "75": "#FF84FF",
  "76": "#BF6464",
  "77": "#FFFF50",
  "78": "#56FF56",
  "79": "#FFFFFF",
  "80": "#4040FF",
  "81": "#57FFFF",
  "82": "#FFA461",
  "83": "#E37EED",
  "84": "#291C1C",
  "85": "#CC6600",
  "86": "#CC6600",
  "87": "#CC6600",
  "88": "#CC6600",
  "89": "#CC6600",
  "90": "#CC6600",
  "91": "#CC6600",
};
const REWARD_TYPE_LABELS = {
  0: "Legs",
  1: "Head",
  2: "Chest",
  3: "Arms",
  4: "Waist",
  5: "Melee",
  6: "Ranged",
  7: "Item",
  8: "Furniture",
  9: "Campaign Quest",
  10: "Zenny",
  11: "Unused",
  12: "Festival Points",
  13: "Unused",
  14: "Tore Points",
  15: "Poogie Outfits",
  16: "Restyle Points",
  17: "N Points",
  18: "Goocoo Outfits",
  19: "Shiny Koban Coin G",
  20: "Trial Koban Coin G",
  21: "Frontier Points (FP)",
  22: "UNKNOWN",
  23: "Ryoudan Points (RP)",
  24: "UNKNOWN",
  25: "Bond/Kizuna Points",
  26: "Unused",
  27: "Unused",
  28: "Special Hall",
  29: "Song Note",
  30: "Item Box pages",
  31: "Equipment Box pages",
};
const AMOUNT_ONLY_REWARD_TYPES = new Set([10, 12, 14, 16, 17, 19, 20, 21, 23, 24, 25, 30, 31]);
const ITEM_ICON_REWARD_TYPES = new Set([7, 22]);
let overlayAnimationTimer = null;
let pageAnimationTimer = null;
let metaLoadToken = 0;

function parseDistributionItemNames(raw) {
  const names = new Map();
  for (const line of String(raw ?? "").split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const commaIndex = trimmed.indexOf(",");
    if (commaIndex <= 0) continue;
    const id = trimmed.slice(0, commaIndex).trim();
    const name = trimmed.slice(commaIndex + 1).trim();
    if (id && name) {
      names.set(id, name);
    }
  }
  return names;
}

const distributionItemNamesById = parseDistributionItemNames(renamedItemsRaw);

function clearOverlayAnimationTimer() {
  if (overlayAnimationTimer) {
    clearTimeout(overlayAnimationTimer);
    overlayAnimationTimer = null;
  }
}

function clearPageAnimationTimer() {
  if (pageAnimationTimer) {
    clearTimeout(pageAnimationTimer);
    pageAnimationTimer = null;
  }
}

function numberFromEntry(entry, ...keys) {
  let highest = 0;
  for (const key of keys) {
    if (entry?.[key] == null || entry?.[key] === "") continue;
    const value = Number(entry?.[key]);
    if (Number.isFinite(value)) {
      highest = Math.max(highest, value);
    }
  }
  return highest;
}

function isRankUnlockedDistribution(entry) {
  if (entry?.rankUnlocked === true || entry?.rankUnlockedByRank === true) return true;
  if (
    numberFromEntry(entry, "min_hr", "minHR", "minHr", "minimumHR", "minimumHr") > 0 ||
    numberFromEntry(entry, "max_hr", "maxHR", "maxHr", "maximumHR", "maximumHr") > 0 ||
    numberFromEntry(entry, "min_sr", "minSR", "minSr", "minimumSR", "minimumSr") > 0 ||
    numberFromEntry(entry, "max_sr", "maxSR", "maxSr", "maximumSR", "maximumSr") > 0 ||
    numberFromEntry(entry, "min_gr", "minGR", "minGr", "minimumGR", "minimumGr") > 0 ||
    numberFromEntry(entry, "max_gr", "maxGR", "maxGr", "maximumGR", "maximumGr") > 0
  ) {
    return true;
  }
  return false;
}

function normalizeDistributionEntry(entry, index) {
  const id = Number(entry?.id ?? index);
  const rawEventName = normalizeDistributionText(entry?.eventName ?? entry?.event_name ?? entry?.name ?? "Distribution");
  const rawDescription = normalizeDistributionText(entry?.description ?? "");
  const eventName = cleanDistributionText(rawEventName);
  const description = cleanDistributionText(rawDescription);
  const typeLabel = cleanDistributionText(normalizeDistributionText(entry?.typeLabel ?? "Distribution"));
  const deadline = entry?.deadline != null ? entry.deadline : null;
  const timesAcceptable = Number(entry?.timesAcceptable ?? entry?.times_acceptable ?? 1);
  return {
    id: Number.isFinite(id) ? id : index,
    eventName: eventName || "Distribution",
    eventNameSegments: parseDistributionColorSegments(eventName ? rawEventName : "Distribution"),
    description: description || "No description available.",
    descriptionSegments: parseDistributionColorSegments(description ? rawDescription : "No description available."),
    typeLabel: typeLabel || "Distribution",
    type: entry?.type ?? null,
    deadline,
    timesAcceptable: Number.isFinite(timesAcceptable) && timesAcceptable > 0 ? timesAcceptable : 1,
    rankUnlocked: isRankUnlockedDistribution(entry),
    items: Array.isArray(entry?.items) ? entry.items : [],
  };
}

function normalizeDistributionText(value) {
  return String(value ?? "")
    .replace(/。/g, ". ")
    .replace(/，/g, ", ")
    .replace(/`/g, "'");
}

function cleanDistributionText(value) {
  return normalizeDistributionText(value).replace(DISTRIBUTION_COLOR_CODE_PATTERN, "").trim();
}

function parseDistributionColorSegments(value) {
  const raw = String(value ?? "").trim();
  if (!raw) return [];

  const segments = [];
  let activeColor = "";
  let activeCode = "";
  let lastIndex = 0;

  for (const match of raw.matchAll(DISTRIBUTION_COLOR_CODE_PATTERN)) {
    const index = match.index ?? 0;
    const text = raw.slice(lastIndex, index);
    if (text) {
      segments.push({ text, color: activeColor, code: activeCode });
    }

    const code = String(match[1] ?? "").padStart(2, "0");
    activeCode = code === "00" ? "" : code;
    activeColor = code === "00" ? "" : (DISTRIBUTION_COLOR_CODES[code] ?? activeColor);
    lastIndex = index + match[0].length;
  }

  const tail = raw.slice(lastIndex);
  if (tail) {
    segments.push({ text: tail, color: activeColor, code: activeCode });
  }

  return segments.length ? segments : [{ text: cleanDistributionText(raw), color: "", code: "" }];
}

function distributionTextSegmentStyle(segment) {
  const color = String(segment?.color ?? "").trim();
  return color ? { color } : null;
}

function shouldStrokeDistributionSegment(segment) {
  return false;
}

const normalizedEntries = computed(() =>
  (Array.isArray(props.entries) ? props.entries : [])
    .map(normalizeDistributionEntry)
    .sort((left, right) => left.id - right.id)
);

const totalPages = computed(() =>
  Math.max(1, Math.ceil(normalizedEntries.value.length / PAGE_SIZE))
);

const visibleEntries = computed(() => {
  const start = pageIndex.value * PAGE_SIZE;
  return normalizedEntries.value.slice(start, start + PAGE_SIZE);
});

const canPagePrev = computed(() => pageIndex.value > 0);
const canPageNext = computed(
  () => pageIndex.value < totalPages.value - 1 || Boolean(props.hasMore)
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

function rewardType(item) {
  const type = Number(item?.itemType ?? 7);
  return Number.isFinite(type) ? type : 7;
}

function rewardItemId(item) {
  if (item?.itemId == null) return null;
  const itemId = Number(item.itemId);
  return Number.isFinite(itemId) ? Math.trunc(itemId) : null;
}

function distributionNameLookupItemId(itemId) {
  if (itemId == null) return "";
  const exact = String(itemId);
  return distributionItemNamesById.has(exact) ? exact : "";
}

function distributionIconLookupItemId(itemId) {
  const numeric = Number(itemId);
  if (!Number.isFinite(numeric) || numeric < 0) return "";
  const hex = Math.trunc(numeric).toString(16).toUpperCase().padStart(4, "0").slice(-4);
  return `${hex.slice(2)}${hex.slice(0, 2)}`;
}

function distributionItemName(itemId) {
  const lookupId = distributionNameLookupItemId(itemId);
  return distributionItemNamesById.get(lookupId) ?? "";
}

function rewardTypeLabel(type) {
  return REWARD_TYPE_LABELS[type] ?? `Type ${type}`;
}

function amountOnlyRewardLabel(type, quantity) {
  const count = Number(quantity ?? 0);
  switch (Number(type)) {
    case 30:
      return count === 1 ? "Item Box Page" : "Item Box Pages";
    case 31:
      return count === 1 ? "Equipment Box Page" : "Equipment Box Pages";
    default:
      return rewardTypeLabel(type);
  }
}

function shouldLoadItemMeta(item) {
  const itemId = rewardItemId(item);
  return itemId != null && itemId >= 0;
}

function rewardKey(item, index) {
  return `${Number(item?.itemType ?? 0)}-${rewardItemId(item) ?? "none"}-${Number(item?.id ?? index)}-${index}`;
}

function rewardMeta(item, index) {
  const type = rewardType(item);
  const itemId = rewardItemId(item);
  const quantity = Math.max(0, Number(item?.quantity ?? 0));
  const key = rewardKey(item, index);
  const hasItemId = itemId != null && itemId >= 0;
  const meta = hasItemId ? itemMetaById.value[String(itemId)] ?? null : null;
  const knownType = REWARD_TYPE_LABELS[type] != null;

  if (AMOUNT_ONLY_REWARD_TYPES.has(type)) {
    return {
      key,
      label: amountOnlyRewardLabel(type, quantity),
      quantity,
      icon: "",
      amountOnly: true,
    };
  }

  if (ITEM_ICON_REWARD_TYPES.has(type) || (!knownType && hasItemId)) {
    return {
      key,
      label: distributionItemName(itemId) || meta?.name || (hasItemId ? `Item ${itemId}` : "Reward"),
      quantity,
      icon: meta?.icon ?? "",
      amountOnly: false,
    };
  }

  return {
    key,
    label: hasItemId ? `${rewardTypeLabel(type)} #${itemId}` : rewardTypeLabel(type),
    quantity,
    icon: "",
    amountOnly: false,
  };
}

function rewardIconMissing(key) {
  return missingRewardIconKeys.value.has(key);
}

function onRewardIconError(key) {
  const next = new Set(missingRewardIconKeys.value);
  next.add(key);
  missingRewardIconKeys.value = next;
}

function formatQuantity(value) {
  const numeric = Number(value ?? 0);
  if (!Number.isFinite(numeric)) return "0";
  return numeric.toLocaleString();
}

function formatDistributionDeadline(value) {
  if (value == null) return "";
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return String(value);
  if (numeric > 9999999999) {
    const date = new Date(numeric * 1000);
    if (Number.isNaN(date.getTime())) return String(value);
    return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
  }
  const date = new Date(numeric * 1000);
  if (Number.isNaN(date.getTime())) return String(value);
  return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

function measureNameScroll() {
  if (nameMeasureFrame.value) {
    cancelAnimationFrame(nameMeasureFrame.value);
  }
  nameMeasureFrame.value = requestAnimationFrame(() => {
    nameMeasureFrame.value = 0;
    const elements = [...document.querySelectorAll(".distribution-entry-name-scroll[data-entry-id]")];
    if (!elements.length) {
      nameScrollMetrics.value = {};
      return;
    }
    nameScrollMetrics.value = Object.fromEntries(
      elements.map((el) => {
        const distance = Math.max(0, Math.ceil(el.scrollWidth - el.clientWidth));
        return [
          String(el.dataset.entryId ?? ""),
          {
            distance,
            duration: distance > 0 ? Math.min(12, Math.max(6, Math.round(distance / 14) + 6)) : 0,
          },
        ];
      })
    );
  });
}

function measureDescScroll() {
  if (descMeasureFrame.value) {
    cancelAnimationFrame(descMeasureFrame.value);
  }
  descMeasureFrame.value = requestAnimationFrame(() => {
    descMeasureFrame.value = 0;
    const elements = [...document.querySelectorAll(".distribution-entry-body[data-entry-id]")];
    if (distributionAutoScrollEnabled.value) {
      resetDistributionManualScroll(elements);
    }
    if (!elements.length) {
      descScrollMetrics.value = {};
      return;
    }
    descScrollMetrics.value = Object.fromEntries(
      elements.map((el) => {
        const content = el.querySelector(".distribution-entry-desc-scroll");
        const distance = content ? Math.max(0, Math.ceil(content.scrollHeight - el.clientHeight)) : 0;
        return [
          String(el.dataset.entryId ?? ""),
          {
            distance,
            duration: distance > 0 ? Math.min(24, Math.max(12, Math.round(distance / 7) + 12)) : 0,
          },
        ];
      })
    );
  });
}

function measureRewardScroll() {
  if (rewardMeasureFrame.value) {
    cancelAnimationFrame(rewardMeasureFrame.value);
  }
  rewardMeasureFrame.value = requestAnimationFrame(() => {
    rewardMeasureFrame.value = 0;
    const elements = [...document.querySelectorAll(".distribution-reward-list-viewport[data-entry-id]")];
    if (distributionAutoScrollEnabled.value) {
      resetDistributionManualScroll(elements);
    }
    if (!elements.length) {
      rewardScrollMetrics.value = {};
      return;
    }
    rewardScrollMetrics.value = Object.fromEntries(
      elements.map((el) => {
        const list = el.querySelector(".distribution-reward-list");
        const distance = list ? Math.max(0, Math.ceil(list.scrollHeight - el.clientHeight)) : 0;
        return [
          String(el.dataset.entryId ?? ""),
          {
            distance,
            duration: distance > 0 ? Math.min(26, Math.max(13, Math.round(distance / 7) + 13)) : 0,
          },
        ];
      })
    );
  });
}

function scrollMetric(metrics, entryId) {
  return metrics.value[String(entryId)] ?? { distance: 0, duration: 0 };
}

function resetDistributionManualScroll(elements = null) {
  const scrollElements = elements ?? [
    ...document.querySelectorAll(".distribution-entry-body[data-entry-id]"),
    ...document.querySelectorAll(".distribution-reward-list-viewport[data-entry-id]"),
  ];
  for (const el of scrollElements) {
    el.scrollTop = 0;
    el.scrollLeft = 0;
  }
}

function nameMetric(entryId) {
  return scrollMetric(nameScrollMetrics, entryId);
}

function descMetric(entryId) {
  return scrollMetric(descScrollMetrics, entryId);
}

function rewardMetric(entryId) {
  return scrollMetric(rewardScrollMetrics, entryId);
}

function scheduleNameMeasure() {
  nextTick(() => {
    measureNameScroll();
    setTimeout(measureNameScroll, 100);
    setTimeout(measureNameScroll, 500);
  });
}

function scheduleDescMeasure() {
  nextTick(() => {
    measureDescScroll();
    setTimeout(measureDescScroll, 100);
    setTimeout(measureDescScroll, 500);
  });
}

function scheduleRewardMeasure() {
  nextTick(() => {
    measureRewardScroll();
    setTimeout(measureRewardScroll, 100);
    setTimeout(measureRewardScroll, 500);
  });
}

function startOpenAnimation() {
  clearOverlayAnimationTimer();
  isRendered.value = true;
  isClosing.value = false;
  isOpening.value = false;

  nextTick(() => {
    requestAnimationFrame(() => {
      isOpening.value = true;
      overlayAnimationTimer = setTimeout(() => {
        isOpening.value = false;
        overlayAnimationTimer = null;
      }, OVERLAY_ANIMATION_MS);
    });
  });
}

function startCloseAnimation() {
  clearOverlayAnimationTimer();
  clearPageAnimationTimer();
  turnDirection.value = "";

  if (!isRendered.value) return;

  isOpening.value = false;
  isClosing.value = true;
  overlayAnimationTimer = setTimeout(() => {
    isClosing.value = false;
    isRendered.value = false;
    overlayAnimationTimer = null;
  }, OVERLAY_ANIMATION_MS);
}

function onCloseClick() {
  playSelect();
  emit("close");
}

function setPage(nextPage, direction) {
  if (nextPage < 0 || nextPage > totalPages.value - 1 || nextPage === pageIndex.value) {
    return;
  }
  clearPageAnimationTimer();
  turnDirection.value = direction;
  pageIndex.value = nextPage;
  flippedEntryIds.value = new Set();
  scheduleNameMeasure();
  scheduleDescMeasure();
  scheduleRewardMeasure();
  pageAnimationTimer = setTimeout(() => {
    turnDirection.value = "";
    pageAnimationTimer = null;
  }, PAGE_ANIMATION_MS);
}

function onPrevPage() {
  playSelect();
  setPage(pageIndex.value - 1, "prev");
}

function onNextPage() {
  playSelect();
  if (pageIndex.value < totalPages.value - 1) {
    setPage(pageIndex.value + 1, "next");
    return;
  }
  if (props.hasMore && !props.loadingMore) {
    pendingAdvanceAfterLoad.value = true;
    emit("load-more");
  }
}

function toggleGuideBubble() {
  playSelect();
  guideBubbleEnabled.value = !guideBubbleEnabled.value;
  writeBooleanPreference(GUIDE_BUBBLE_STORAGE_KEY, guideBubbleEnabled.value);
}

function toggleDistributionAutoScroll() {
  playSelect();
  distributionAutoScrollEnabled.value = !distributionAutoScrollEnabled.value;
}

function scrollDistributionDescription(event) {
  if (distributionAutoScrollEnabled.value) return;
  const target = event?.currentTarget;
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
  target.scrollTo({
    top: Math.max(0, Math.min(maxTop, target.scrollTop + delta)),
    behavior: "smooth",
  });
}

function toggleEntryFlip(entryId) {
  playSelect();
  const next = new Set(flippedEntryIds.value);
  if (next.has(entryId)) {
    next.delete(entryId);
  } else {
    next.add(entryId);
  }
  flippedEntryIds.value = next;
  scheduleRewardMeasure();
}

watch(distributionAutoScrollEnabled, (enabled) => {
  if (!enabled) return;
  descScrollMetrics.value = {};
  rewardScrollMetrics.value = {};
  nextTick(() => {
    resetDistributionManualScroll();
    scheduleDescMeasure();
    scheduleRewardMeasure();
  });
});

watch(
  () => props.open,
  (open) => {
    if (open) {
      pageIndex.value = 0;
      flippedEntryIds.value = new Set();
      startOpenAnimation();
    } else {
      startCloseAnimation();
    }
  },
  { immediate: true }
);

watch(
  normalizedEntries,
  (entries) => {
    if (pageIndex.value > totalPages.value - 1) {
      pageIndex.value = totalPages.value - 1;
    }
    flippedEntryIds.value = new Set();
    nameScrollMetrics.value = {};
    descScrollMetrics.value = {};
    rewardScrollMetrics.value = {};

    const token = ++metaLoadToken;
    const itemIds = [
      ...new Set(
        entries
          .flatMap((entry) => entry.items)
          .filter((item) => shouldLoadItemMeta(item))
          .map((item) => rewardItemId(item))
          .filter((itemId) => itemId != null && itemId >= 0)
          .map(String)
      ),
    ];

    if (!itemIds.length) {
      itemMetaById.value = {};
      missingRewardIconKeys.value = new Set();
    } else {
      Promise.all(
        itemIds.map(async (itemId) => {
          const lookupId = distributionIconLookupItemId(Number(itemId));
          let displayMeta = null;
          try {
            displayMeta = await getItemDisplayMeta(lookupId || itemId);
          } catch {
            displayMeta = null;
          }
          return [
            itemId,
            {
              id: itemId,
              name: distributionItemName(Number(itemId)) || displayMeta?.name || `Item ${itemId}`,
              icon: displayMeta?.icon ?? "",
            },
          ];
        })
      )
        .then((pairs) => {
          if (token !== metaLoadToken) return;
          itemMetaById.value = Object.fromEntries(pairs);
          missingRewardIconKeys.value = new Set();
        })
        .catch(() => {
          if (token === metaLoadToken) itemMetaById.value = {};
        });
    }

    nextTick(() => {
      scheduleNameMeasure();
      scheduleDescMeasure();
      scheduleRewardMeasure();
    });
  },
  { immediate: true }
);

watch(
  () => normalizedEntries.value.length,
  (length, previousLength) => {
    if (!pendingAdvanceAfterLoad.value || length <= previousLength) return;
    pendingAdvanceAfterLoad.value = false;
    setPage(Math.min(pageIndex.value + 1, totalPages.value - 1), "next");
  }
);

onBeforeUnmount(() => {
  clearOverlayAnimationTimer();
  clearPageAnimationTimer();
  if (nameMeasureFrame.value) cancelAnimationFrame(nameMeasureFrame.value);
  if (descMeasureFrame.value) cancelAnimationFrame(descMeasureFrame.value);
  if (rewardMeasureFrame.value) cancelAnimationFrame(rewardMeasureFrame.value);
});

defineExpose({
  contentRoot,
});
</script>

<template>
  <div
    v-if="isRendered"
    class="distribution-overlay"
    :class="{
      'distribution-overlay-opening': isOpening,
      'distribution-overlay-closing': isClosing,
    }"
    @click.self="onCloseClick"
  >
    <div
      class="distribution-overlay-backdrop"
      :class="{ 'distribution-overlay-backdrop-static': store.settings.linuxHardwareAcceleration === false }"
      :style="overlayBackdropStyle"
    ></div>
    <div
      ref="contentRoot"
      class="distribution-dialog"
      data-controller-scope="distribution"
    >
      <button
        type="button"
        class="distribution-close"
        data-controller-size="big"
        @mouseenter="playHover()"
        @click="onCloseClick"
        aria-label="Close"
      >
        <span class="distribution-close-art" aria-hidden="true"></span>
      </button>

      <div class="distribution-title font-main">Distribution</div>

      <div
        class="distribution-guide"
        :class="{
          'distribution-guide-bubble-hidden': !showGuideBubble,
          'distribution-guide-disabled': !guideBubbleEnabled,
        }"
      >
        <div class="distribution-guide-bubble font-main">
          Talk to the Guide in-game to claim these rewards!
        </div>
        <button
          type="button"
          class="distribution-guide-button"
          data-controller-size="small"
          @mouseenter="playHover()"
          @click="toggleGuideBubble"
          :aria-pressed="guideBubbleEnabled"
          aria-label="Toggle guide popout"
        >
          <img
            :src="assetUrl(guideBubbleEnabled ? '/extra/DistroGuide.png' : '/extra/DistroGuideHidden.png')"
            class="distribution-guide-image"
            draggable="false"
            alt=""
          />
        </button>
      </div>

      <button
        type="button"
        class="distribution-autoscroll-toggle"
        :class="{ 'distribution-autoscroll-toggle-on': distributionAutoScrollEnabled }"
        :aria-pressed="distributionAutoScrollEnabled"
        title="Toggle description and reward auto scroll"
        @mouseenter="playHover()"
        @click.stop="toggleDistributionAutoScroll"
      >
        <span class="distribution-autoscroll-toggle-label font-main">Auto scroll</span>
        <span class="distribution-autoscroll-toggle-track">
          <span class="distribution-autoscroll-toggle-thumb"></span>
        </span>
      </button>

      <template v-if="!hasCharacter">
        <div class="distribution-status font-main">No character selected.</div>
      </template>
      <template v-else-if="loading">
        <div class="distribution-status distribution-status-loading font-main">
          <img
            :src="assetUrl('/extra/ItemBoxLoad.gif')"
            class="distribution-loading-gif"
            draggable="false"
            alt=""
          />
        </div>
      </template>
      <template v-else-if="!normalizedEntries.length">
        <div class="distribution-status font-main">No unclaimed distributions.</div>
      </template>
      <template v-else>
        <div
          class="distribution-entry-row"
          :class="{
            'distribution-entry-row-next': turnDirection === 'next',
            'distribution-entry-row-prev': turnDirection === 'prev',
          }"
        >
          <button
            v-for="(entry, entryIndex) in visibleEntries"
            :key="`distribution-entry-${pageIndex}-${entry.id}-${entryIndex}`"
            type="button"
            class="distribution-entry-card"
            :class="{ 'distribution-entry-card-flipped': flippedEntryIds.has(entry.id) }"
            data-controller-size="big"
            @click="toggleEntryFlip(entry.id)"
          >
            <img
              v-if="entry.rankUnlocked"
              :key="`distribution-rank-new-${pageIndex}-${entry.id}-${entryIndex}`"
              :src="assetUrl(`/extra/Distronew.gif?entry=${pageIndex}-${entry.id}-${entryIndex}`)"
              class="distribution-entry-rank-new"
              draggable="false"
              alt=""
            />
            <span class="distribution-entry-face distribution-entry-front">
              <span class="distribution-entry-header">
                <span
                  class="distribution-entry-name font-main distribution-entry-name-scroll"
                  :data-entry-id="entry.id"
                  :class="{ 'distribution-entry-name-scrolling': nameMetric(entry.id).distance > 0 }"
                  :style="nameMetric(entry.id).distance > 0 ? { '--distribution-name-scroll-distance': nameMetric(entry.id).distance + 'px', '--distribution-name-scroll-duration': nameMetric(entry.id).duration + 's' } : {}"
                >
                  <span class="distribution-entry-name-marquee">
                    <span class="distribution-entry-name-copy">
                      <span
                        v-for="(segment, segmentIndex) in entry.eventNameSegments"
                        :key="`distribution-title-segment-${entry.id}-${segmentIndex}`"
                        :class="{ 'distribution-color-segment-stroked': shouldStrokeDistributionSegment(segment) }"
                        :style="distributionTextSegmentStyle(segment)"
                      >{{ segment.text }}</span>
                    </span>
                    <span
                      v-if="nameMetric(entry.id).distance > 0"
                      class="distribution-entry-name-copy"
                      aria-hidden="true"
                    >
                      <span
                        v-for="(segment, segmentIndex) in entry.eventNameSegments"
                        :key="`distribution-title-repeat-segment-${entry.id}-${segmentIndex}`"
                        :class="{ 'distribution-color-segment-stroked': shouldStrokeDistributionSegment(segment) }"
                        :style="distributionTextSegmentStyle(segment)"
                      >{{ segment.text }}</span>
                    </span>
                  </span>
                </span>
              </span>
              <span
                class="distribution-entry-body"
                :class="{ 'distribution-entry-body-scrollable': !distributionAutoScrollEnabled }"
                :data-entry-id="entry.id"
                @wheel.stop="scrollDistributionDescription"
              >
                <span
                  class="distribution-entry-description font-main distribution-entry-desc-scroll"
                  :class="{ 'distribution-entry-desc-scrolling': distributionAutoScrollEnabled && descMetric(entry.id).distance > 0 }"
                  :style="distributionAutoScrollEnabled && descMetric(entry.id).distance > 0 ? { '--distribution-desc-scroll-distance': descMetric(entry.id).distance + 'px', '--distribution-desc-scroll-duration': descMetric(entry.id).duration + 's' } : {}"
                >
                  <span
                    v-for="(segment, segmentIndex) in entry.descriptionSegments"
                    :key="`distribution-desc-segment-${entry.id}-${segmentIndex}`"
                    :class="{ 'distribution-color-segment-stroked': shouldStrokeDistributionSegment(segment) }"
                    :style="distributionTextSegmentStyle(segment)"
                  >{{ segment.text }}</span>
                </span>
              </span>
              <span class="distribution-entry-footer font-main">
                <span class="distribution-entry-type">{{ entry.typeLabel }}</span>
                <span v-if="entry.deadline" class="distribution-entry-deadline">Deadline: {{ formatDistributionDeadline(entry.deadline) }}</span>
              </span>
            </span>
            <span class="distribution-entry-face distribution-entry-back">
              <span class="distribution-entry-rewards-title font-main">Rewards</span>
              <span
                v-if="entry.items.length"
                class="distribution-reward-list-viewport"
                :class="{ 'distribution-reward-list-viewport-scrollable': !distributionAutoScrollEnabled }"
                :data-entry-id="entry.id"
                @wheel.stop
              >
                <span
                  class="distribution-reward-list"
                  :class="{ 'distribution-reward-list-scrolling': distributionAutoScrollEnabled && rewardMetric(entry.id).distance > 0 }"
                  :style="distributionAutoScrollEnabled && rewardMetric(entry.id).distance > 0 ? { '--distribution-reward-scroll-distance': rewardMetric(entry.id).distance + 'px', '--distribution-reward-scroll-duration': rewardMetric(entry.id).duration + 's' } : {}"
                >
                  <span
                    v-for="(item, itemIndex) in entry.items"
                    :key="rewardMeta(item, itemIndex).key"
                    class="distribution-reward-item font-main"
                    :class="{
                      'distribution-reward-item-amount-only': rewardMeta(item, itemIndex).amountOnly,
                      'distribution-reward-item-no-icon': !rewardMeta(item, itemIndex).amountOnly && (!rewardMeta(item, itemIndex).icon || rewardIconMissing(rewardMeta(item, itemIndex).key)),
                    }"
                  >
                    <span
                      v-if="rewardMeta(item, itemIndex).quantity > 0"
                      class="distribution-reward-quantity"
                    >
                      x{{ formatQuantity(rewardMeta(item, itemIndex).quantity) }}
                    </span>
                    <img
                      v-if="rewardMeta(item, itemIndex).icon && !rewardIconMissing(rewardMeta(item, itemIndex).key)"
                      :src="rewardMeta(item, itemIndex).icon"
                      class="distribution-reward-icon"
                      draggable="false"
                      alt=""
                      @error="onRewardIconError(rewardMeta(item, itemIndex).key)"
                    />
                    <span class="distribution-reward-text">
                      {{ rewardMeta(item, itemIndex).label }}
                    </span>
                  </span>
                </span>
              </span>
              <span v-else class="distribution-entry-empty font-main">
                Reward details are unavailable.
              </span>
              <span class="distribution-entry-back-footer font-main">
                <span class="distribution-entry-back-type">{{ entry.typeLabel }}</span>
                <span v-if="entry.timesAcceptable > 1" class="distribution-entry-claim-count">Claimable x{{ entry.timesAcceptable }}</span>
              </span>
            </span>
          </button>
        </div>

        <div
          v-if="loadingMore"
          class="distribution-page-loading"
        >
          <img
            :src="assetUrl('/extra/ItemBoxLoad.gif')"
            class="distribution-loading-gif"
            draggable="false"
            alt=""
          />
        </div>

        <button
          v-if="canPagePrev"
          type="button"
          class="distribution-page-arrow distribution-page-arrow-prev"
          :class="{ 'distribution-page-arrow-prev-bubble-hidden': !showGuideBubble }"
          data-controller-size="small"
          @mouseenter="playHover()"
          @click="onPrevPage"
          aria-label="Previous distributions"
        >
          <img
            :src="assetUrl('/extra/LeftArrow.png')"
            class="distribution-page-arrow-art distribution-page-arrow-art-base"
            draggable="false"
            alt=""
          />
          <img
            :src="assetUrl('/extra/LeftArrowHover.png')"
            class="distribution-page-arrow-art distribution-page-arrow-art-hover"
            draggable="false"
            alt=""
          />
        </button>
        <button
          v-if="canPageNext"
          type="button"
          class="distribution-page-arrow distribution-page-arrow-next"
          data-controller-size="small"
          @mouseenter="playHover()"
          @click="onNextPage"
          aria-label="Next distributions"
        >
          <img
            :src="assetUrl('/extra/RightArrow.png')"
            class="distribution-page-arrow-art distribution-page-arrow-art-base"
            draggable="false"
            alt=""
          />
          <img
            :src="assetUrl('/extra/RightArrowHover.png')"
            class="distribution-page-arrow-art distribution-page-arrow-art-hover"
            draggable="false"
            alt=""
          />
        </button>
      </template>
    </div>
  </div>
</template>
