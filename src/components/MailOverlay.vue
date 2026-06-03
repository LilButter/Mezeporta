<script setup>
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { assetUrl, store } from "../store";
import { playHover, playSelect } from "../sfx";
import { getItemDisplayMeta } from "../altclient/savedataView";
import renamedItemsRaw from "../../renamed_items.txt?raw";

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
const currentIndex = ref(0);
const turnState = ref(null);
const itemMetaById = ref({});
const missingAttachmentIconKeys = ref(new Set());

const OVERLAY_ANIMATION_MS = 320;
const PAGE_TURN_OUT_MS = 110;
const PAGE_TURN_IN_MS = 150;

let overlayAnimationTimer = null;
let pageTurnTimer = null;
let metaLoadToken = 0;

const mailPaperStyle = {
  backgroundImage: `url(${assetUrl("/extra/MainMail.png")})`,
};
const mailCloseButtonStyle = {
  "--mail-close-art-base": `url(${assetUrl("/extra/MailClose.png")})`,
  "--mail-close-art-hover": `url(${assetUrl("/extra/MailCloseHover.png")})`,
};
const mailTextBaseStyle = {
  fontFamily: 'var(--launcher-font-family, "MS Gothic", "Zen Antique", serif)',
  textShadow: "none",
  color: "#5a3e2a",
};
const mailSystemStyle = {
  ...mailTextBaseStyle,
  position: "absolute",
  top: "24px",
  left: "50%",
  maxWidth: "calc(100% - 120px)",
  transform: "translateX(-50%)",
  fontSize: "15px",
  lineHeight: "1",
  letterSpacing: "1.6px",
  textTransform: "none",
  textAlign: "center",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
  color: "#65462f",
};
const mailHeaderStyle = {
  ...mailTextBaseStyle,
  position: "absolute",
  top: "48px",
  left: "34px",
  right: "34px",
  fontSize: "29px",
  lineHeight: "1.04",
  textAlign: "center",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
};
const mailFromLineStyle = {
  ...mailTextBaseStyle,
  position: "absolute",
  top: "112px",
  left: "36px",
  right: "36px",
  fontSize: "16px",
  lineHeight: "1.15",
  color: "#5f4330",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
};
const mailBodyPanelStyle = {
  position: "absolute",
  top: "148px",
  left: "34px",
  right: "34px",
  bottom: "114px",
  padding: "10px 8px 8px 10px",
  overflow: "hidden",
};
const mailBodyStyle = {
  ...mailTextBaseStyle,
  height: "100%",
  overflowY: "auto",
  whiteSpace: "pre-wrap",
  overflowWrap: "anywhere",
  wordBreak: "break-word",
  fontSize: "17px",
  lineHeight: "1.24",
  scrollbarWidth: "none",
};
const mailAttachmentBlockStyle = {
  ...mailTextBaseStyle,
  position: "absolute",
  left: "36px",
  right: "36px",
  bottom: "22px",
  color: "#5f4330",
  textAlign: "center",
};
const mailAttachmentLabelStyle = {
  ...mailTextBaseStyle,
  marginBottom: "2px",
  fontSize: "14px",
  lineHeight: "1",
  fontWeight: "700",
  color: "#5f4330",
};
const mailAttachmentValueStyle = {
  ...mailTextBaseStyle,
  fontSize: "16px",
  lineHeight: "1.1",
  color: "#5f4330",
};
const mailDateLineStyle = {
  ...mailTextBaseStyle,
  position: "absolute",
  top: "92px",
  left: "36px",
  right: "36px",
  fontSize: "14px",
  lineHeight: "1.1",
  color: "#6a4a34",
  textAlign: "center",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
};
const mailPageContentStyle = {
  position: "absolute",
  inset: "0",
  pointerEvents: "none",
};
const mailPreviewContentStyle = {
  ...mailPageContentStyle,
  opacity: 0.92,
};

function parseMailItemNames(raw) {
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

const mailItemNamesById = parseMailItemNames(renamedItemsRaw);

function mailNameLookupItemId(itemId) {
  if (itemId == null) return "";
  const exact = String(itemId);
  return mailItemNamesById.has(exact) ? exact : "";
}

function mailIconLookupItemId(itemId) {
  const numeric = Number(itemId);
  if (!Number.isFinite(numeric) || numeric < 0) return "";
  const hex = Math.trunc(numeric).toString(16).toUpperCase().padStart(4, "0").slice(-4);
  return `${hex.slice(2)}${hex.slice(0, 2)}`;
}

function mailItemName(itemId) {
  const lookupId = mailNameLookupItemId(itemId);
  return mailItemNamesById.get(lookupId) ?? "";
}

function clearOverlayAnimationTimer() {
  if (overlayAnimationTimer) {
    clearTimeout(overlayAnimationTimer);
    overlayAnimationTimer = null;
  }
}

function clearPageTurnTimer() {
  if (pageTurnTimer) {
    clearTimeout(pageTurnTimer);
    pageTurnTimer = null;
  }
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
  clearPageTurnTimer();
  turnState.value = null;

  if (!isRendered.value) return;

  isOpening.value = false;
  isClosing.value = true;
  overlayAnimationTimer = setTimeout(() => {
    isClosing.value = false;
    isRendered.value = false;
    overlayAnimationTimer = null;
  }, OVERLAY_ANIMATION_MS);
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      currentIndex.value = 0;
      turnState.value = null;
      startOpenAnimation();
    } else {
      startCloseAnimation();
    }
  },
  { immediate: true }
);

watch(
  () => props.entries,
  (entries) => {
    const total = Array.isArray(entries) ? entries.length : 0;
    if (total <= 0) {
      currentIndex.value = 0;
      turnState.value = null;
      clearPageTurnTimer();
      return;
    }

    if (currentIndex.value > total - 1) {
      currentIndex.value = total - 1;
    }
  },
  { deep: true }
);

const mailEntries = computed(() =>
  Array.isArray(props.entries) ? props.entries : []
);

watch(
  mailEntries,
  (entries) => {
    const token = ++metaLoadToken;
    const itemIds = [
      ...new Set(
        entries
          .map((entry) => mailAttachedItemId(entry))
          .filter((itemId) => itemId > 0)
          .map(String)
      ),
    ];

    if (!itemIds.length) {
      itemMetaById.value = {};
      missingAttachmentIconKeys.value = new Set();
      return;
    }

    Promise.all(
      itemIds.map(async (itemId) => {
        const lookupId = mailIconLookupItemId(Number(itemId));
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
            name: mailItemName(Number(itemId)) || displayMeta?.name || `Item ${itemId}`,
            icon: displayMeta?.icon ?? "",
          },
        ];
      })
    )
      .then((pairs) => {
        if (token !== metaLoadToken) return;
        itemMetaById.value = Object.fromEntries(pairs);
        missingAttachmentIconKeys.value = new Set();
      })
      .catch(() => {
        if (token === metaLoadToken) itemMetaById.value = {};
      });
  },
  { immediate: true }
);

const totalEntries = computed(() => mailEntries.value.length);
const currentEntry = computed(() => mailEntries.value[currentIndex.value] ?? null);
const previousPreviewEntry = computed(() =>
  currentIndex.value > 0 ? mailEntries.value[currentIndex.value - 1] ?? null : null
);
const previousPreviewEntrySecondary = computed(() =>
  currentIndex.value > 1 ? mailEntries.value[currentIndex.value - 2] ?? null : null
);
const nextPreviewEntry = computed(() =>
  currentIndex.value < totalEntries.value - 1
    ? mailEntries.value[currentIndex.value + 1] ?? null
    : null
);
const nextPreviewEntrySecondary = computed(() =>
  currentIndex.value < totalEntries.value - 2
    ? mailEntries.value[currentIndex.value + 2] ?? null
    : null
);
const hasPreviousEntry = computed(() => Boolean(previousPreviewEntry.value));
const hasPreviousEntrySecondary = computed(() =>
  Boolean(previousPreviewEntrySecondary.value)
);
const hasNextEntry = computed(() => Boolean(nextPreviewEntry.value));
const hasNextEntrySecondary = computed(() => Boolean(nextPreviewEntrySecondary.value));
const currentPaperTurnClass = computed(() => {
  if (!turnState.value) return null;
  return [
    "mail-overlay-paper-current-turn",
    `mail-overlay-paper-current-turn-${turnState.value.phase}-${turnState.value.direction}`,
  ];
});
const paperShellTurnClass = computed(() => {
  if (!turnState.value) return null;
  return `mail-overlay-paper-shell-turn-${turnState.value.direction}`;
});
const overlayBackdropStyle = computed(() => {
  if (store.settings.linuxHardwareAcceleration !== false) {
    return null;
  }
  return {
    background: "radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%)",
    backdropFilter: "none",
  };
});

function mailTypeLabel(entry) {
  if (entry?.isGuildInvite) return "Guild Invite";
  if (entry?.isSystemMessage) return "System";
  const marker = String(entry?.label ?? "").trim();
  const normalized = marker.toUpperCase();
  if (normalized === "G" || normalized === "[G]" || normalized === "GUILD INVITE") {
    return "Guild Invite";
  }
  if (normalized === "S" || normalized === "[S]" || normalized === "SYSTEM") {
    return "System";
  }
  return marker;
}

function isSystemMail(entry) {
  return Boolean(mailTypeLabel(entry));
}

function mailHeader(entry) {
  return String(entry?.subject ?? "").trim() || "(No subject)";
}

function mailBody(entry) {
  return String(entry?.body ?? "").trim() || "(No body)";
}

function mailSender(entry) {
  return String(entry?.senderName ?? "Unknown").trim() || "Unknown";
}

function mailDate(entry) {
  const seconds = Number(entry?.createdAt ?? entry?.created_at ?? 0);
  if (!Number.isFinite(seconds) || seconds <= 0) return "";
  const date = new Date(seconds * 1000);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "numeric",
    minute: "2-digit",
  });
}

function mailAttachedItemId(entry) {
  const itemId = Number(entry?.attachedItem ?? entry?.attached_item ?? entry?.itemId ?? entry?.item_id ?? 0);
  return Number.isFinite(itemId) ? Math.trunc(itemId) : 0;
}

function mailAttachedItemAmount(entry) {
  const amount = Number(entry?.attachedItemAmount ?? entry?.attached_item_amount ?? entry?.itemAmount ?? 0);
  return Number.isFinite(amount) ? Math.max(0, Math.trunc(amount)) : 0;
}

function mailAttachmentKey(entry) {
  return `mail-attachment-${mailAttachedItemId(entry)}`;
}

function mailAttachmentIconMissing(key) {
  return missingAttachmentIconKeys.value.has(key);
}

function onMailAttachmentIconError(key) {
  missingAttachmentIconKeys.value = new Set([...missingAttachmentIconKeys.value, key]);
}

function mailAttachmentMeta(entry) {
  const itemId = mailAttachedItemId(entry);
  const hasItem = Boolean(entry?.hasItem) || itemId > 0;
  if (!hasItem || itemId <= 0) {
    return {
      hasItem: false,
      key: "mail-attachment-none",
      label: "No attachment",
      quantity: 0,
      icon: "",
    };
  }

  const meta = itemMetaById.value[String(itemId)] ?? null;
  return {
    hasItem: true,
    key: mailAttachmentKey(entry),
    label: mailItemName(itemId) || meta?.name || `Item ${itemId}`,
    quantity: mailAttachedItemAmount(entry),
    icon: meta?.icon ?? "",
  };
}

function goTo(direction) {
  if (turnState.value) return;

  const offset = direction === "next" ? 1 : -1;
  const targetIndex = currentIndex.value + offset;
  if (targetIndex < 0 || targetIndex >= totalEntries.value) return;

  playSelect();
  clearPageTurnTimer();
  turnState.value = {
    direction,
    phase: "out",
  };

  pageTurnTimer = setTimeout(() => {
    currentIndex.value = targetIndex;
    turnState.value = {
      direction,
      phase: "in",
    };

    pageTurnTimer = setTimeout(() => {
      turnState.value = null;
      pageTurnTimer = null;
    }, PAGE_TURN_IN_MS);
  }, PAGE_TURN_OUT_MS);
}

function onCloseClick() {
  playSelect();
  emit("close");
}

onBeforeUnmount(() => {
  clearOverlayAnimationTimer();
  clearPageTurnTimer();
});

defineExpose({
  contentRoot,
});
</script>

<template>
  <div
    v-if="isRendered"
    class="mail-overlay"
    :class="{
      'mail-overlay-opening': isOpening,
      'mail-overlay-closing': isClosing,
    }"
    @click.self="onCloseClick"
  >
    <div
      class="mail-overlay-backdrop"
      :class="{ 'mail-overlay-backdrop-static': store.settings.linuxHardwareAcceleration === false }"
      :style="overlayBackdropStyle"
      @click="onCloseClick"
    ></div>

    <div
      ref="contentRoot"
      class="mail-overlay-stage"
      data-controller-scope="mail"
    >
      <div class="mail-overlay-paper-shell" :class="paperShellTurnClass">
        <div
          v-if="hasPreviousEntrySecondary"
          class="mail-overlay-side-stack mail-overlay-side-stack-left mail-overlay-side-stack-secondary"
        >
          <div class="mail-overlay-side-stack-clip">
            <div class="mail-overlay-paper mail-overlay-paper-preview" :style="mailPaperStyle"></div>
          </div>
        </div>

        <div
          v-if="hasPreviousEntry"
          class="mail-overlay-side-stack mail-overlay-side-stack-left"
        >
          <div class="mail-overlay-side-stack-clip">
            <div class="mail-overlay-paper mail-overlay-paper-preview" :style="mailPaperStyle">
            <div
              v-if="previousPreviewEntry"
              class="mail-overlay-page-content mail-overlay-page-content-preview"
              :style="mailPreviewContentStyle"
            >
              <div
                v-if="isSystemMail(previousPreviewEntry)"
                class="mail-overlay-page-system"
                :style="mailSystemStyle"
              >
                {{ mailTypeLabel(previousPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-header" :style="mailHeaderStyle">
                {{ mailHeader(previousPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-body-panel" :style="mailBodyPanelStyle">
                <div class="mail-overlay-page-body" :style="mailBodyStyle">
                  {{ mailBody(previousPreviewEntry) }}
                </div>
              </div>
              <div class="mail-overlay-page-from-line" :style="mailFromLineStyle">
                From: {{ mailSender(previousPreviewEntry) }}
              </div>
              <div
                v-if="mailDate(previousPreviewEntry)"
                class="mail-overlay-page-date-line"
                :style="mailDateLineStyle"
              >
                {{ mailDate(previousPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-attachment-block" :style="mailAttachmentBlockStyle">
                <div class="mail-overlay-page-attachment-label" :style="mailAttachmentLabelStyle">Attachments</div>
                <div class="mail-overlay-page-attachment-value" :style="mailAttachmentValueStyle">
                  <span
                    v-if="mailAttachmentMeta(previousPreviewEntry).hasItem"
                    class="mail-overlay-attachment-line"
                  >
                    <img
                      v-if="mailAttachmentMeta(previousPreviewEntry).icon && !mailAttachmentIconMissing(mailAttachmentMeta(previousPreviewEntry).key)"
                      :src="mailAttachmentMeta(previousPreviewEntry).icon"
                      class="mail-overlay-attachment-icon"
                      draggable="false"
                      alt=""
                      @error="onMailAttachmentIconError(mailAttachmentMeta(previousPreviewEntry).key)"
                    />
                    <span class="mail-overlay-attachment-name">{{ mailAttachmentMeta(previousPreviewEntry).label }}</span>
                    <span
                      v-if="mailAttachmentMeta(previousPreviewEntry).quantity > 0"
                      class="mail-overlay-attachment-quantity"
                    >x{{ mailAttachmentMeta(previousPreviewEntry).quantity }}</span>
                  </span>
                  <span v-else>No attachment</span>
                </div>
              </div>
            </div>
            </div>
          </div>
        </div>

        <div
          v-if="hasNextEntrySecondary"
          class="mail-overlay-side-stack mail-overlay-side-stack-right mail-overlay-side-stack-secondary"
        >
          <div class="mail-overlay-side-stack-clip">
            <div class="mail-overlay-paper mail-overlay-paper-preview" :style="mailPaperStyle"></div>
          </div>
        </div>

        <div
          v-if="hasNextEntry"
          class="mail-overlay-side-stack mail-overlay-side-stack-right"
        >
          <div class="mail-overlay-side-stack-clip">
            <div class="mail-overlay-paper mail-overlay-paper-preview" :style="mailPaperStyle">
            <div
              v-if="nextPreviewEntry"
              class="mail-overlay-page-content mail-overlay-page-content-preview"
              :style="mailPreviewContentStyle"
            >
              <div
                v-if="isSystemMail(nextPreviewEntry)"
                class="mail-overlay-page-system"
                :style="mailSystemStyle"
              >
                {{ mailTypeLabel(nextPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-header" :style="mailHeaderStyle">
                {{ mailHeader(nextPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-body-panel" :style="mailBodyPanelStyle">
                <div class="mail-overlay-page-body" :style="mailBodyStyle">
                  {{ mailBody(nextPreviewEntry) }}
                </div>
              </div>
              <div class="mail-overlay-page-from-line" :style="mailFromLineStyle">
                From: {{ mailSender(nextPreviewEntry) }}
              </div>
              <div
                v-if="mailDate(nextPreviewEntry)"
                class="mail-overlay-page-date-line"
                :style="mailDateLineStyle"
              >
                {{ mailDate(nextPreviewEntry) }}
              </div>
              <div class="mail-overlay-page-attachment-block" :style="mailAttachmentBlockStyle">
                <div class="mail-overlay-page-attachment-label" :style="mailAttachmentLabelStyle">Attachments</div>
                <div class="mail-overlay-page-attachment-value" :style="mailAttachmentValueStyle">
                  <span
                    v-if="mailAttachmentMeta(nextPreviewEntry).hasItem"
                    class="mail-overlay-attachment-line"
                  >
                    <img
                      v-if="mailAttachmentMeta(nextPreviewEntry).icon && !mailAttachmentIconMissing(mailAttachmentMeta(nextPreviewEntry).key)"
                      :src="mailAttachmentMeta(nextPreviewEntry).icon"
                      class="mail-overlay-attachment-icon"
                      draggable="false"
                      alt=""
                      @error="onMailAttachmentIconError(mailAttachmentMeta(nextPreviewEntry).key)"
                    />
                    <span class="mail-overlay-attachment-name">{{ mailAttachmentMeta(nextPreviewEntry).label }}</span>
                    <span
                      v-if="mailAttachmentMeta(nextPreviewEntry).quantity > 0"
                      class="mail-overlay-attachment-quantity"
                    >x{{ mailAttachmentMeta(nextPreviewEntry).quantity }}</span>
                  </span>
                  <span v-else>No attachment</span>
                </div>
              </div>
            </div>
            </div>
          </div>
        </div>

        <button
          v-if="hasPreviousEntry"
          type="button"
          class="mail-overlay-arrow mail-overlay-arrow-left"
          data-controller-size="big"
          :disabled="Boolean(turnState)"
          @mouseenter="playHover()"
          @click="goTo('prev')"
        >
          <img
            :src="assetUrl('/extra/LeftArrow.png')"
            class="mail-overlay-arrow-art mail-overlay-arrow-art-base"
            alt=""
            draggable="false"
          />
          <img
            :src="assetUrl('/extra/LeftArrowHover.png')"
            class="mail-overlay-arrow-art mail-overlay-arrow-art-hover"
            alt=""
            draggable="false"
          />
        </button>

        <div
          class="mail-overlay-paper mail-overlay-paper-current"
          :class="currentPaperTurnClass"
          :style="mailPaperStyle"
        >
          <button
            type="button"
            class="mail-overlay-close"
            data-controller-size="small"
            :style="mailCloseButtonStyle"
            @mouseenter="playHover()"
            @click="onCloseClick"
          >
            <span class="mail-overlay-close-art" aria-hidden="true"></span>
          </button>

          <template v-if="!hasCharacter">
            <div class="mail-overlay-status">No character selected.</div>
          </template>
          <template v-else-if="loading">
            <div class="mail-overlay-status">now loading...</div>
          </template>
          <template v-else-if="currentEntry">
            <div
              class="mail-overlay-page-content"
              :style="mailPageContentStyle"
            >
              <div
                v-if="isSystemMail(currentEntry)"
                class="mail-overlay-page-system"
                :style="mailSystemStyle"
              >
                {{ mailTypeLabel(currentEntry) }}
              </div>
              <div class="mail-overlay-page-header" :style="mailHeaderStyle">
                {{ mailHeader(currentEntry) }}
              </div>
              <div class="mail-overlay-page-body-panel" :style="mailBodyPanelStyle">
                <div class="mail-overlay-page-body" :style="mailBodyStyle">
                  {{ mailBody(currentEntry) }}
                </div>
              </div>
              <div class="mail-overlay-page-from-line" :style="mailFromLineStyle">
                From: {{ mailSender(currentEntry) }}
              </div>
              <div
                v-if="mailDate(currentEntry)"
                class="mail-overlay-page-date-line"
                :style="mailDateLineStyle"
              >
                {{ mailDate(currentEntry) }}
              </div>
              <div class="mail-overlay-page-attachment-block" :style="mailAttachmentBlockStyle">
                <div class="mail-overlay-page-attachment-label" :style="mailAttachmentLabelStyle">Attachments</div>
                <div class="mail-overlay-page-attachment-value" :style="mailAttachmentValueStyle">
                  <span
                    v-if="mailAttachmentMeta(currentEntry).hasItem"
                    class="mail-overlay-attachment-line"
                  >
                    <img
                      v-if="mailAttachmentMeta(currentEntry).icon && !mailAttachmentIconMissing(mailAttachmentMeta(currentEntry).key)"
                      :src="mailAttachmentMeta(currentEntry).icon"
                      class="mail-overlay-attachment-icon"
                      draggable="false"
                      alt=""
                      @error="onMailAttachmentIconError(mailAttachmentMeta(currentEntry).key)"
                    />
                    <span class="mail-overlay-attachment-name">{{ mailAttachmentMeta(currentEntry).label }}</span>
                    <span
                      v-if="mailAttachmentMeta(currentEntry).quantity > 0"
                      class="mail-overlay-attachment-quantity"
                    >x{{ mailAttachmentMeta(currentEntry).quantity }}</span>
                  </span>
                  <span v-else>No attachment</span>
                </div>
              </div>
            </div>
          </template>
          <template v-else>
            <div class="mail-overlay-status">No unread mail.</div>
          </template>
        </div>

        <button
          v-if="hasNextEntry"
          type="button"
          class="mail-overlay-arrow mail-overlay-arrow-right"
          data-controller-size="big"
          :disabled="Boolean(turnState)"
          @mouseenter="playHover()"
          @click="goTo('next')"
        >
          <img
            :src="assetUrl('/extra/RightArrow.png')"
            class="mail-overlay-arrow-art mail-overlay-arrow-art-base"
            alt=""
            draggable="false"
          />
          <img
            :src="assetUrl('/extra/RightArrowHover.png')"
            class="mail-overlay-arrow-art mail-overlay-arrow-art-hover"
            alt=""
            draggable="false"
          />
        </button>

      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-overlay {
  position: absolute;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mail-overlay-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.58);
  backdrop-filter: blur(4px) brightness(0.35);
}

.mail-overlay-backdrop-static {
  background: radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%);
}

.mail-overlay-stage {
  position: relative;
  z-index: 1;
  width: min(1080px, calc(100vw - 56px));
  height: min(760px, calc(100vh - 40px));
  pointer-events: none;
}

.mail-overlay-paper-shell {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 476px;
  height: 558px;
  transform: translate(-50%, -50%);
  pointer-events: none;
  z-index: 3;
  transition: transform 180ms ease;
}

.mail-overlay-paper {
  position: absolute;
  inset: 0;
  overflow: hidden;
  background-position: center;
  background-repeat: no-repeat;
  background-size: 100% 100%;
  box-shadow: none;
  pointer-events: auto;
  z-index: 2;
}

.mail-overlay-side-stack {
  position: absolute;
  bottom: 10px;
  width: 360px;
  height: 422px;
  pointer-events: none;
  z-index: 1;
  transition: transform 180ms ease, opacity 180ms ease;
}

.mail-overlay-side-stack-secondary {
  z-index: 0;
}

.mail-overlay-side-stack-clip {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.mail-overlay-side-stack-left {
  left: -154px;
  transform-origin: right bottom;
  transform: rotate(-3deg);
}

.mail-overlay-side-stack-right {
  right: -154px;
  transform-origin: left bottom;
  transform: rotate(3deg);
}

.mail-overlay-side-stack-left.mail-overlay-side-stack-secondary {
  left: -186px;
  bottom: 18px;
  transform: rotate(-5.2deg);
}

.mail-overlay-side-stack-right.mail-overlay-side-stack-secondary {
  right: -186px;
  bottom: 18px;
  transform: rotate(5.2deg);
}

.mail-overlay-paper-preview {
  box-shadow: none;
  pointer-events: none;
  z-index: 1;
  top: 0;
  bottom: 0;
}

.mail-overlay-paper-preview::after {
  content: "";
  position: absolute;
  inset: 0;
  background: rgba(28, 21, 17, 0.28);
  -webkit-backdrop-filter: blur(1.1px);
  backdrop-filter: blur(1.1px);
  pointer-events: none;
  z-index: 2;
}

.mail-overlay-side-stack-left .mail-overlay-paper-preview {
  left: auto;
  right: -12px;
  width: 100%;
}

.mail-overlay-side-stack-right .mail-overlay-paper-preview {
  left: -12px;
  right: auto;
  width: 100%;
}

.mail-overlay-side-stack-left:not(.mail-overlay-side-stack-secondary) .mail-overlay-paper-preview::after {
  left: 5px;
}

.mail-overlay-side-stack-right:not(.mail-overlay-side-stack-secondary) .mail-overlay-paper-preview::after {
  right: 5px;
}

.mail-overlay-side-stack-left.mail-overlay-side-stack-secondary .mail-overlay-paper-preview {
  right: -26px;
  width: calc(100% + 26px);
}

.mail-overlay-side-stack-right.mail-overlay-side-stack-secondary .mail-overlay-paper-preview {
  left: -26px;
  width: calc(100% + 26px);
}

.mail-overlay-side-stack-left.mail-overlay-side-stack-secondary .mail-overlay-paper-preview::after {
  left: 2px;
}

.mail-overlay-side-stack-right.mail-overlay-side-stack-secondary .mail-overlay-paper-preview::after {
  right: 2px;
}

.mail-overlay-paper-shell-turn-next .mail-overlay-side-stack-left {
  transform: translateX(-10px) rotate(-4deg);
}

.mail-overlay-paper-shell-turn-next .mail-overlay-side-stack-left.mail-overlay-side-stack-secondary {
  transform: translateX(-16px) rotate(-5.9deg);
}

.mail-overlay-paper-shell-turn-next .mail-overlay-side-stack-right {
  transform: translateX(6px) rotate(2.2deg);
}

.mail-overlay-paper-shell-turn-next .mail-overlay-side-stack-right.mail-overlay-side-stack-secondary {
  transform: translateX(10px) rotate(4.6deg);
}

.mail-overlay-paper-shell-turn-prev .mail-overlay-side-stack-right {
  transform: translateX(10px) rotate(4deg);
}

.mail-overlay-paper-shell-turn-prev .mail-overlay-side-stack-right.mail-overlay-side-stack-secondary {
  transform: translateX(16px) rotate(5.9deg);
}

.mail-overlay-paper-shell-turn-prev .mail-overlay-side-stack-left {
  transform: translateX(-6px) rotate(-2.2deg);
}

.mail-overlay-paper-shell-turn-prev .mail-overlay-side-stack-left.mail-overlay-side-stack-secondary {
  transform: translateX(-10px) rotate(-4.6deg);
}

.mail-overlay-paper-current,
.mail-overlay-paper-outgoing,
.mail-overlay-paper-incoming {
  z-index: 3;
}

.mail-overlay-paper-current-turn {
  will-change: transform, opacity;
  transform-origin: center center;
}

.mail-overlay-close {
  position: absolute;
  top: 20px;
  right: 19px;
  width: 26px;
  height: 26px;
  border: 0;
  padding: 0;
  background: transparent;
  cursor: pointer;
  pointer-events: auto;
  z-index: 5;
}

.mail-overlay-arrow-art {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  transition: opacity 120ms ease, transform 120ms ease, filter 120ms ease;
}

.mail-overlay-arrow-art-hover {
  opacity: 0;
}

.mail-overlay-close-art {
  position: absolute;
  inset: 0;
  display: block;
  background-image: var(--mail-close-art-base);
  background-position: center;
  background-repeat: no-repeat;
  background-size: contain;
  transition: transform 120ms ease, filter 120ms ease, opacity 120ms ease,
    background-image 120ms ease;
}

.mail-overlay-close:hover .mail-overlay-close-art,
.mail-overlay-close:focus-visible .mail-overlay-close-art,
.mail-overlay-close.controller-nav-focused .mail-overlay-close-art {
  background-image: var(--mail-close-art-hover);
  transform: scale(1.03);
  filter: drop-shadow(0 0 10px rgba(255, 215, 124, 0.24));
}

.mail-overlay-arrow:hover .mail-overlay-arrow-art-base,
.mail-overlay-arrow:focus-visible .mail-overlay-arrow-art-base,
.mail-overlay-arrow.controller-nav-focused .mail-overlay-arrow-art-base {
  opacity: 0;
}

.mail-overlay-arrow:hover .mail-overlay-arrow-art-hover,
.mail-overlay-arrow:focus-visible .mail-overlay-arrow-art-hover,
.mail-overlay-arrow.controller-nav-focused .mail-overlay-arrow-art-hover {
  opacity: 1;
}

.mail-overlay-arrow {
  position: absolute;
  top: 50%;
  width: 58px;
  height: 56px;
  border: 0;
  padding: 0;
  background: transparent;
  cursor: pointer;
  transform: translateY(-50%);
  pointer-events: auto;
  z-index: 4;
}

.mail-overlay-arrow:disabled {
  cursor: default;
}

.mail-overlay-arrow-left {
  left: -29px;
}

.mail-overlay-arrow-right {
  right: -29px;
}

.mail-overlay-arrow:hover .mail-overlay-arrow-art-hover,
.mail-overlay-arrow:focus-visible .mail-overlay-arrow-art-hover,
.mail-overlay-arrow.controller-nav-focused .mail-overlay-arrow-art-hover {
  transform: scale(1.04);
  filter: drop-shadow(0 0 10px rgba(255, 215, 124, 0.28));
}

.mail-overlay-page-content {
  position: absolute;
  inset: 0;
  pointer-events: none;
  backface-visibility: hidden;
  transform: translateZ(0);
}

.mail-overlay-page-content-preview {
  opacity: 1;
}

.mail-overlay-page-content-turn {
  will-change: transform, opacity;
  transform-origin: center top;
}

.mail-overlay-page-content-turn-out-next,
.mail-overlay-page-content-turn-out-prev {
  animation-duration: 110ms;
  animation-fill-mode: both;
  animation-timing-function: cubic-bezier(0.32, 0, 0.2, 1);
}

.mail-overlay-page-content-turn-in-next,
.mail-overlay-page-content-turn-in-prev {
  animation-duration: 150ms;
  animation-fill-mode: both;
  animation-timing-function: cubic-bezier(0.2, 0.82, 0.24, 1);
}

.mail-overlay-page-content-turn-out-next {
  animation-name: mail-page-turn-out-next;
}

.mail-overlay-page-content-turn-in-next {
  animation-name: mail-page-turn-in-next;
}

.mail-overlay-page-content-turn-out-prev {
  animation-name: mail-page-turn-out-prev;
}

.mail-overlay-page-content-turn-in-prev {
  animation-name: mail-page-turn-in-prev;
}

.mail-overlay-paper-current-turn-out-next,
.mail-overlay-paper-current-turn-out-prev {
  animation-duration: 110ms;
  animation-fill-mode: both;
  animation-timing-function: cubic-bezier(0.32, 0, 0.2, 1);
}

.mail-overlay-paper-current-turn-in-next,
.mail-overlay-paper-current-turn-in-prev {
  animation-duration: 150ms;
  animation-fill-mode: both;
  animation-timing-function: cubic-bezier(0.2, 0.82, 0.24, 1);
}

.mail-overlay-paper-current-turn-out-next {
  animation-name: mail-paper-turn-out-next;
}

.mail-overlay-paper-current-turn-in-next {
  animation-name: mail-paper-turn-in-next;
}

.mail-overlay-paper-current-turn-out-prev {
  animation-name: mail-paper-turn-out-prev;
}

.mail-overlay-paper-current-turn-in-prev {
  animation-name: mail-paper-turn-in-prev;
}

.mail-overlay-page-system {
  position: absolute;
  top: 24px;
  left: 50%;
  max-width: calc(100% - 120px);
  color: #65462f;
  font-size: 15px;
  line-height: 1;
  letter-spacing: 1.6px;
  text-transform: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transform: translateX(-50%);
  text-align: center;
  text-shadow: none;
  height: 100%;
}

.mail-overlay-page-header {
  position: absolute;
  top: 48px;
  left: 34px;
  right: 34px;
  color: #5a3e2a;
  font-size: 29px;
  line-height: 1.04;
  text-align: center;
  text-shadow: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mail-overlay-page-body-panel {
  position: absolute;
  padding: 10px 8px 8px 10px !important;
  display: flex !important;
  height: 370px;
  top: 140px !important;
}

.mail-overlay-page-body {
  height: 100%;
  overflow-y: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
  color: #5a3e2a;
  font-size: 17px;
  line-height: 1.24;
  text-shadow: none;
  scrollbar-width: none;
}

.mail-overlay-page-body::-webkit-scrollbar {
  display: none;
}

.mail-overlay-page-attachment-block {
  position: absolute;
  left: 36px;
  right: 36px;
  bottom: 22px;
  color: #5f4330;
  text-align: center;
  text-shadow: none;
}

.mail-overlay-page-attachment-label {
  margin-bottom: 10px !important;
  font-size: 14px;
  line-height: 1;
  font-weight: 700;
}

.mail-overlay-page-attachment-value {
  font-size: 16px;
  line-height: 1.1;
  height: 35px;
}

.mail-overlay-page-date-line {
  position: absolute;
  top: 92px;
  left: 36px;
  right: 36px;
  color: #6a4a34;
  font-size: 14px;
  line-height: 1.1;
  text-align: center;
  text-shadow: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mail-overlay-attachment-line {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  max-width: 100%;
  gap: 3px;
  min-width: 0;
}

.mail-overlay-attachment-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
  image-rendering: auto;
  flex: 0 0 auto;
}

.mail-overlay-attachment-name {
  min-width: 0;
  overflow: visible;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mail-overlay-attachment-quantity {
  flex: 0 0 auto;
}

.mail-overlay-status {
  position: absolute;
  inset: 84px 34px 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #5a3e2a;
  font-family: var(--launcher-font-family, "MS Gothic", "Zen Antique", serif);
  font-size: 24px;
  line-height: 1.2;
  text-align: center;
  text-shadow: none;
}

@keyframes mail-page-turn-out-next {
  from {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
  to {
    opacity: 0;
    transform: translateX(-34px) rotate(-2.6deg) scale(0.987);
  }
}

@keyframes mail-paper-turn-out-next {
  from {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
  to {
    opacity: 0.985;
    transform: translateX(-18px) rotate(-1.9deg) scale(0.992);
  }
}

@keyframes mail-paper-turn-in-next {
  from {
    opacity: 0.985;
    transform: translateX(18px) rotate(1.9deg) scale(0.992);
  }
  to {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
}

@keyframes mail-paper-turn-out-prev {
  from {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
  to {
    opacity: 0.985;
    transform: translateX(18px) rotate(1.9deg) scale(0.992);
  }
}

@keyframes mail-paper-turn-in-prev {
  from {
    opacity: 0.985;
    transform: translateX(-18px) rotate(-1.9deg) scale(0.992);
  }
  to {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
}

@keyframes mail-page-turn-in-next {
  from {
    opacity: 0;
    transform: translateX(38px) rotate(2.7deg) scale(0.987);
  }
  to {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
}

@keyframes mail-page-turn-out-prev {
  from {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
  to {
    opacity: 0;
    transform: translateX(34px) rotate(2.6deg) scale(0.987);
  }
}

@keyframes mail-page-turn-in-prev {
  from {
    opacity: 0;
    transform: translateX(-38px) rotate(-2.7deg) scale(0.987);
  }
  to {
    opacity: 1;
    transform: translateX(0) rotate(0deg) scale(1);
  }
}


@keyframes mail-overlay-bounce-in {
  0% {
    opacity: 0;
    transform: translateY(76px) scale(0.94);
  }
  62% {
    opacity: 1;
    transform: translateY(-8px) scale(1.015);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes mail-overlay-bounce-out {
  0% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  100% {
    opacity: 0;
    transform: translateY(72px) scale(0.94);
  }
}

@keyframes mail-overlay-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes mail-overlay-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

.mail-overlay-opening .mail-overlay-stage {
  animation: mail-overlay-bounce-in 320ms cubic-bezier(0.18, 0.9, 0.28, 1.2) both;
}

.mail-overlay-closing .mail-overlay-stage {
  animation: mail-overlay-bounce-out 320ms cubic-bezier(0.4, 0, 0.65, 1) both;
}

.mail-overlay-opening .mail-overlay-backdrop {
  animation: mail-overlay-fade-in 320ms ease both;
}

.mail-overlay-closing .mail-overlay-backdrop {
  animation: mail-overlay-fade-out 320ms ease both;
}

@media (max-width: 1100px) {
  .mail-overlay-stage {
    width: min(980px, calc(100vw - 36px));
  }

  .mail-overlay-preview {
    width: 320px;
    height: 376px;
    top: calc(50% + 91px);
  }

  .mail-overlay-preview-left {
    left: calc(50% - 322px);
  }

  .mail-overlay-preview-right {
    right: calc(50% - 322px);
  }

  .mail-overlay-arrow-left {
    left: calc(50% - 230px);
  }

  .mail-overlay-arrow-right {
    right: calc(50% - 230px);
  }
}
</style>
