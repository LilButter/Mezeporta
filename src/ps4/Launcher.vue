<script setup>
import "./style.css";
import "../controllerNavigation.css";

import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { open } from "@tauri-apps/api/shell";
import { appWindow } from "@tauri-apps/api/window";

import Login        from "./Login.vue";
import Characters   from "./Characters.vue";
import MessageList  from "./MessageList.vue";
import Settings     from "./Settings.vue";
import Patcher      from "./Patcher.vue";
import CharacterBookOverlay from "../components/CharacterBookOverlay.vue";
import MailOverlay from "../components/MailOverlay.vue";
import DistributionOverlay from "../components/DistributionOverlay.vue";
import FriendsOverlay from "../components/FriendsOverlay.vue";

import { availableLocales, getMessage } from "../fluent";
import {
  DELETE_DIALOG,
  SERVERS_DIALOG,
  LOGIN_PAGE,
  SETTINGS_PAGE,
  CHARACTERS_PAGE,
  openPicker,
  PATCHER_PAGE,
  PATCHER_DIALOG,
  SERVER_SWITCH_DIALOG,
  VERSION_SWITCH_DIALOG,
  EXTERNAL_LINK_DIALOG,
  RESET_PATCH_DIALOG,
  BAN_DIALOG,
  LINUX_PREFIX_DIALOG,
  CHECKING_PATCHER,
  DOWNLOADING_PATCHER,
  RESTORING_PATCHER,
  PATCHING_PATCHER,
  DONE_PATCHER,
  forceRepaint,
} from "../common";

import {
  store,
  storeMut,
  closeLauncher,
  closeDialog,
  dismissRecentLog,
  dialogRemoveEndpoint,
  dialogSaveEndpoint,
  dialogDeleteCharacterConfirm,
  recentLog,
  currentBanner,
  bannerIndex,
  setBannerIndex,
  onSettingsButton,
  dialogCallback,
  confirmExternalLinkOpen,
  dialogCancelExternalLink,
  dialogConfirmExternalLink,
  dialogConfirmExternalLinkDontShowAgain,
  dialogVersionSwitchStay,
  dialogVersionSwitchYes,
  dialogVersionSwitchDontAskAgain,
  dialogVersionSignatureSelect,
  patcherLog,
  effectiveBanners,
  currentSavedataVersionToken,
} from "../store";

import {
  launcherHeaderUrl,
  capcomUrl,
  cogUrl,
  dialogUrl,
  serverPatchUrl,
  ps4ButtonUrl,
  ps4EmblemUrl,
  assetUrl,
  effectiveMessages,
} from "../store";
import { EMPTY_ALT_CLIENT_STATS, getAltClientDistributions, getAltClientStats } from "../altClientStats";
import { buildActiveEventBadges } from "../activeEvents";
import { hasAltCharacterSavedataVersion, prefetchAltCharacterSavedata } from "../altClientSavedata";
import { getAltCharacterSavedataView, clearAltSavedataPanelCache } from "../altclient/savedataView";
import { useLauncherGamepad } from "../useLauncherGamepad";

import { playHover, playConfirm, playSelect, bindSfx } from "../sfx";

const launcherRoot = ref(null);
const settingsBtn = ref(null);
const bannerImg   = ref(null);
const srvNameEl   = ref(null);
const srvUrlEl    = ref(null);
const srvLportEl  = ref(null);
const srvGportEl  = ref(null);
const dialogScopeRef = ref(null);
const bookOverlayRef = ref(null);
const mailOverlayRef = ref(null);
const distributionOverlayRef = ref(null);
const friendsOverlayRef = ref(null);
const serverDialogNameDownNode = ref("server-dialog-host");
const ps4VersionLabel = "release ver. 1.5.2";
const ps4LauncherTitle = computed(() => {
  const version = String(store.settings?.gameVersion ?? "").trim().toUpperCase();
  if (version === "S6" || version === "S7K" || version === "F4" || version === "F5") {
    return "Monster Hunter Frontier Online Launcher";
  }
  if (version === "ZZ") {
    return "Monster Hunter Frontier ZZ Launcher";
  }
  if (version.startsWith("G")) {
    return "Monster Hunter Frontier G Launcher";
  }
  return "Monster Hunter Frontier Z Launcher";
});
function pickPs4ToastCompanion() {
  return assetUrl(
    Math.random() < 0.5 ? "/extra/ErrorFelyne.png" : "/extra/ErrorPoogie.png"
  );
}

const ps4ToastCompanion = ref(pickPs4ToastCompanion());

const RESET_BAR_WIDTH = 302;
const RESET_BAR_LEFT = 8;
const DISTRIBUTION_PAGE_SIZE = 6;
const resetPatchProgressClamped = computed(() =>
  Math.max(0, Math.min(1, Number(store.resetPatchProgress ?? 0)))
);
const resetPatchMaskWidth = computed(
  () => `${RESET_BAR_WIDTH - RESET_BAR_WIDTH * resetPatchProgressClamped.value}px`
);
const resetPatchPoogieLeft = computed(
  () =>
    `${RESET_BAR_LEFT + RESET_BAR_WIDTH * resetPatchProgressClamped.value}px`
);
const linuxPrefixInstallProgressClamped = computed(() =>
  Math.max(0, Math.min(1, Number(store.linuxPrefixInstallProgress ?? 0)))
);
const linuxPrefixInstallMaskWidth = computed(
  () => `${RESET_BAR_WIDTH - RESET_BAR_WIDTH * linuxPrefixInstallProgressClamped.value}px`
);
const linuxPrefixInstallPoogieLeft = computed(
  () =>
    `${RESET_BAR_LEFT + RESET_BAR_WIDTH * linuxPrefixInstallProgressClamped.value}px`
);

const characterLoadingLines = ref([]);
const characterLoadingActive = ref(false);
const distributionEntriesByCharacterId = ref({});
const distributionLoadingMore = ref(false);

const patcherLogLines = ref([]);
let lastPatcherLogLine = "";
let lastPatcherLogState = null;
let patcherRunHasStarted = false;
const PATCHER_QUEUE_LOG_STATE = Symbol("patcherQueue");

function resetPatcherLogLines() {
  patcherLogLines.value = [];
  lastPatcherLogLine = "";
  lastPatcherLogState = null;
  patcherRunHasStarted = false;
}

function trimPatcherLogLines() {
  if (patcherLogLines.value.length > 250) {
    patcherLogLines.value.splice(0, patcherLogLines.value.length - 250);
  }
}

function upsertPatcherLogLine(state, line) {
  if (
    state === DOWNLOADING_PATCHER &&
    lastPatcherLogState === DOWNLOADING_PATCHER &&
    patcherLogLines.value.length > 0
  ) {
    patcherLogLines.value[patcherLogLines.value.length - 1] = line;
  } else if (line !== lastPatcherLogLine || state !== lastPatcherLogState) {
    patcherLogLines.value.push(line);
    trimPatcherLogLines();
  }
  lastPatcherLogLine = line;
  lastPatcherLogState = state;
}

function onCharacterLoadingLog(payload) {
  characterLoadingActive.value = Boolean(payload?.active);
  characterLoadingLines.value = Array.isArray(payload?.lines) ? payload.lines : [];
}

// multiple link wrappers (one per v-for item)
const linkRefs    = ref([]);

const dialogBackgroundAsset = computed(() => {
  if (store.dialogKind === PATCHER_DIALOG) return serverPatchUrl.value;
  if (store.dialogKind === SERVER_SWITCH_DIALOG) return serverPatchUrl.value;
  if (store.dialogKind === VERSION_SWITCH_DIALOG) return serverPatchUrl.value;
  return dialogUrl.value;
});
const overlayBackdropStyle = computed(() => {
  if (store.settings.linuxHardwareAcceleration !== false) {
    return { backdropFilter: "blur(3px) brightness(0.45)" };
  }
  return {
    background: "radial-gradient(circle at center, #1f1a12 0%, #0b0b0b 70%)",
  };
});

const ps4RootStyle = computed(() => ({
  "--ps4-start-button-image": `url('${ps4ButtonUrl.value}')`,
}));

const ps4TopLinks = computed(() =>
  Array.isArray(store.links) ? store.links.slice(0, 4) : []
);

const ps4NewsScrollEl = ref(null);
const ps4NewsScrollTop = ref(0);
const ps4NewsScrollClientHeight = ref(1);
const ps4NewsScrollHeight = ref(1);

function syncPs4NewsScrollMetrics() {
  const el = ps4NewsScrollEl.value;
  if (!el) {
    ps4NewsScrollTop.value = 0;
    ps4NewsScrollClientHeight.value = 1;
    ps4NewsScrollHeight.value = 1;
    return;
  }

  ps4NewsScrollTop.value = el.scrollTop;
  ps4NewsScrollClientHeight.value = Math.max(1, el.clientHeight);
  ps4NewsScrollHeight.value = Math.max(1, el.scrollHeight);
}

function scrollPs4News(delta) {
  const el = ps4NewsScrollEl.value;
  if (!el) return;
  el.scrollBy({ top: delta, behavior: "smooth" });
}

const ps4NewsHasOverflow = computed(
  () => ps4NewsScrollHeight.value > ps4NewsScrollClientHeight.value + 1
);

const ps4NewsCanScrollUp = computed(() => ps4NewsScrollTop.value > 0);
const ps4NewsCanScrollDown = computed(
  () =>
    ps4NewsScrollTop.value + ps4NewsScrollClientHeight.value <
    ps4NewsScrollHeight.value - 1
);

const ps4NewsThumbStyle = computed(() => {
  if (!ps4NewsHasOverflow.value) {
    return { height: "0%", top: "0%" };
  }

  const visiblePercent = Math.max(
    12,
    (ps4NewsScrollClientHeight.value / ps4NewsScrollHeight.value) * 100
  );
  const maxTop = Math.max(0, 100 - visiblePercent);
  const denominator = Math.max(
    1,
    ps4NewsScrollHeight.value - ps4NewsScrollClientHeight.value
  );
  const topPercent = (ps4NewsScrollTop.value / denominator) * maxTop;

  return {
    height: `${visiblePercent}%`,
    top: `${topPercent}%`,
  };
});

const alertClass = {
  info: "alert-info",
  warning: "alert-warning",
  error: "alert-error",
};

// Unbinders
let unbindSettings = null;
let unbindBanner   = null;
let linkUnbinders  = [];

const showServerInfoPanel = ref(false);
const serverInfoLoading = ref(false);
const serverInfo = ref({ ...EMPTY_ALT_CLIENT_STATS });
const serverInfoPanelRoot = ref(null);
const eventPanelRoot = ref(null);
const activeEventInfoId = ref(null);
const ps4MailRoot = ref(null);
const showMailPanel = ref(false);
const showDistributionPanel = ref(false);
const showFriendsPanel = ref(false);
const showBookPanel = ref(false);
const bookPanelLoading = ref(false);
const bookPanelReady = ref(false);
const savedataPanelData = ref(null);
const selectedCharacterId = ref(null);
const lastSavedataCharacterId = ref(null);
const hasRealCharacter = ref(false);
const activeCharacterHasSavedataVersion = ref(false);
let savedataVersionCheckNonce = 0;

const canShowServerInfo = computed(
  () => storeMut.page === CHARACTERS_PAGE && !characterLoadingActive.value
);

const canShowSavedataPanels = computed(() => canShowServerInfo.value);
const canUseSavedataPanels = computed(
  () => canShowSavedataPanels.value && hasRealCharacter.value
);
const canOpenBookPanel = computed(
  () => canUseSavedataPanels.value && activeCharacterHasSavedataVersion.value
);
const anyAltPanelOpen = computed(
  () =>
    showServerInfoPanel.value ||
    showMailPanel.value ||
    showDistributionPanel.value ||
    showFriendsPanel.value ||
    showBookPanel.value ||
    Boolean(activeEventInfoId.value)
);

const activeCharacterId = computed(() => {
  if (!hasRealCharacter.value) return null;

  const numeric = Number(selectedCharacterId.value ?? 0);
  return Number.isFinite(numeric) && numeric > 0 ? Math.trunc(numeric) : null;
});

const activeAltCharacter = computed(() => {
  if (!canUseSavedataPanels.value) return null;

  const characters = Array.isArray(serverInfo.value?.characters)
    ? serverInfo.value.characters
    : [];
  if (!characters.length) return null;

  const targetId = activeCharacterId.value;
  if (Number.isFinite(targetId) && targetId > 0) {
    return (
      characters.find(
        (character) => Number(character?.id ?? 0) === targetId
      ) ?? null
    );
  }

  return null;
});

const activeCharacterName = computed(() => {
  const serverName = String(activeAltCharacter.value?.name ?? "").trim();
  if (serverName) return serverName;

  const targetId = activeCharacterId.value;
  const character = Array.isArray(store.characters)
    ? store.characters.find(
        (entry) => Number(entry?.id ?? 0) === Number(targetId ?? 0)
      )
    : null;
  return String(character?.name ?? "").trim();
});

const activeAltCourses = computed(() => {
  const courses = activeAltCharacter.value?.courses;
  return Array.isArray(courses) ? courses : [];
});

const CHARACTER_HR_THRESHOLDS = [
  { min: 0, hr: 1 },
  { min: 30, hr: 2 },
  { min: 50, hr: 3 },
  { min: 99, hr: 4 },
  { min: 299, hr: 5 },
  { min: 998, hr: 6 },
  { min: 999, hr: 7 },
];

function getCharacterHR(character) {
  if (!character) return 0;
  const hr = Number(character?.hr ?? character?.HR ?? 0);
  if (!Number.isFinite(hr)) return 0;
  return hr;
}

function getCharacterGR(character) {
  if (!character) return 0;
  const gr = Number(character?.gr ?? character?.GR ?? 0);
  if (!Number.isFinite(gr)) return 0;
  return gr;
}

function getCharacterSR(character) {
  if (!character) return 0;
  const sr = Number(character?.sr ?? character?.SR ?? 0);
  if (!Number.isFinite(sr)) return 0;
  return sr;
}

function distributionRankValue(entry, fallback, ...keys) {
  for (const key of keys) {
    const value = entry?.[key];
    if (value == null || value === "") continue;
    const numeric = Number(value);
    if (Number.isFinite(numeric)) return numeric;
  }
  return fallback;
}

function distributionMinRank(entry, ...keys) {
  return Math.max(0, distributionRankValue(entry, 0, ...keys));
}

function distributionMaxRank(entry, ...keys) {
  const value = distributionRankValue(entry, Infinity, ...keys);
  return Number.isFinite(value) && value > 0 ? value : Infinity;
}

function characterMeetsDistributionRank(character, entry) {
  if (!character) return false;
  const charHR = getCharacterHR(character);
  const charGR = getCharacterGR(character);
  const charSR = getCharacterSR(character);

  const minHR = distributionMinRank(entry, "min_hr", "minHR", "minHr");
  const maxHR = distributionMaxRank(entry, "max_hr", "maxHR", "maxHr");
  if (charHR < minHR || charHR > maxHR) return false;

  const minGR = distributionMinRank(entry, "min_gr", "minGR", "minGr");
  const maxGR = distributionMaxRank(entry, "max_gr", "maxGR", "maxGr");
  if (charGR < minGR || charGR > maxGR) return false;

  const minSR = distributionMinRank(entry, "min_sr", "minSR", "minSr");
  const maxSR = distributionMaxRank(entry, "max_sr", "maxSR", "maxSr");
  if (charSR < minSR || charSR > maxSR) return false;

  return true;
}

const activeAltUser = computed(() => ({
  gachaPremium: Number(serverInfo.value?.user?.gachaPremium ?? 0),
  gachaTrial: Number(serverInfo.value?.user?.gachaTrial ?? 0),
  frontierPoints: Number(serverInfo.value?.user?.frontierPoints ?? 0),
}));

const activeAltOnlineFriends = computed(() => {
  const friends = Array.isArray(serverInfo.value?.onlineFriends)
    ? serverInfo.value.onlineFriends
    : [];
  const ownerCharacterId = activeCharacterId.value ?? Number(activeAltCharacter.value?.id ?? 0);
  if (!Number.isFinite(ownerCharacterId) || ownerCharacterId <= 0) {
    return [];
  }
  return friends.filter((friend) => Number(friend?.cid ?? 0) === ownerCharacterId);
});

const activeAltFriendEntries = computed(() => {
  const ownerCharacterId = activeCharacterId.value ?? Number(activeAltCharacter.value?.id ?? 0);
  if (!Number.isFinite(ownerCharacterId) || ownerCharacterId <= 0) return [];

  const onlineFriendsById = new Map(
    activeAltOnlineFriends.value.map((friend) => [Number(friend?.id ?? 0), friend])
  );

  const entries = (Array.isArray(store.friends) ? store.friends : [])
    .filter((friend) => Number(friend?.cid ?? 0) === ownerCharacterId)
    .map((friend) => {
      const friendId = Number(friend?.id ?? 0);
      const onlineFriend = onlineFriendsById.get(friendId);
      return {
        id: friendId,
        name:
          String(onlineFriend?.name ?? "").trim() ||
          String(friend?.name ?? "").trim() ||
          `Character #${friendId}`,
        online: Boolean(onlineFriend),
        serverId: Number(onlineFriend?.serverId ?? 0),
      };
    })
    .filter((friend) => Number.isFinite(friend.id) && friend.id > 0);

  for (const onlineFriend of activeAltOnlineFriends.value) {
    const friendId = Number(onlineFriend?.id ?? 0);
    if (!Number.isFinite(friendId) || friendId <= 0) continue;
    if (entries.some((friend) => friend.id === friendId)) continue;
    entries.push({
      id: friendId,
      name: String(onlineFriend?.name ?? "").trim() || `Character #${friendId}`,
      online: true,
      serverId: Number(onlineFriend?.serverId ?? 0),
    });
  }

  return entries;
});

const activeAltOnlineFriendCount = computed(() =>
  activeAltFriendEntries.value.filter((friend) => friend.online).length
);

const showFriendsButton = computed(
  () => canUseSavedataPanels.value && activeAltFriendEntries.value.length > 0
);

const activeAltUnreadMail = computed(() =>
  Math.max(0, Number(activeAltCharacter.value?.unreadMail ?? 0))
);

const activeAltUnreadMailEntries = computed(() => {
  const entries = activeAltCharacter.value?.unreadMailEntries;
  return Array.isArray(entries) ? entries : [];
});

const showPs4MailButton = computed(
  () => canUseSavedataPanels.value && activeAltUnreadMail.value > 0
);

const activeAltDistributionEntries = computed(() => {
  if (!canUseSavedataPanels.value) return [];
  const characterKey = String(activeCharacterId.value ?? "");
  const cachedEntries = distributionEntriesByCharacterId.value[characterKey];
  if (Array.isArray(cachedEntries)) {
    return cachedEntries;
  }
  const entries = activeAltCharacter.value?.unclaimedDistributionDetails;
  if (!Array.isArray(entries)) return [];
  return entries;
});

const activeAltUnclaimedDistributions = computed(() =>
  Math.max(
    activeAltDistributionEntries.value.length,
    Number(activeAltCharacter.value?.unclaimedDistributions ?? 0)
  )
);

const showDistributionButton = computed(
  () => canUseSavedataPanels.value && activeAltUnclaimedDistributions.value > 0
);

const hasMoreActiveAltDistributionEntries = computed(
  () => activeAltDistributionEntries.value.length < activeAltUnclaimedDistributions.value
);

const activeEventBadges = computed(() =>
  buildActiveEventBadges(serverInfo.value, {
    formatUnixSeconds,
    formatPanelNumber,
  })
);

const activeEventInfo = computed(
  () =>
    activeEventBadges.value.find((event) => event.id === activeEventInfoId.value) ??
    null
);

watch(activeEventBadges, (events) => {
  if (
    activeEventInfoId.value &&
    !events.some((event) => event.id === activeEventInfoId.value)
  ) {
    activeEventInfoId.value = null;
  }
});

function toggleEventInfoPanel(eventId) {
  playSelect();
  if (!canShowSavedataPanels.value) {
    closeAltPanels();
    return;
  }

  activeEventInfoId.value =
    activeEventInfoId.value === eventId ? null : eventId;
  showServerInfoPanel.value = false;
  showMailPanel.value = false;
  showDistributionPanel.value = false;
  showFriendsPanel.value = false;
  showBookPanel.value = false;
}

function decodeWorldLand(serverId) {
  const numeric = Number(serverId);
  if (!Number.isFinite(numeric)) return null;

  const normalized = Math.trunc(numeric);
  const baseServerId = 4112;
  const delta = normalized - baseServerId;
  if (delta < 0) return null;

  const world = Math.floor(delta / 256) + 1;
  const land = (delta % 256) + 1;
  if (world <= 0 || land <= 0) return null;

  return { world, land };
}

function formatWorldLand(serverId) {
  const location = decodeWorldLand(serverId);
  if (!location) {
    const numeric = Number(serverId);
    if (Number.isFinite(numeric) && numeric > 0) {
      return `Server ID ${Math.trunc(numeric)}`;
    }
    return "Unknown location";
  }

  return `World ${location.world}, Land ${location.land}`;
}

function formatUnixSeconds(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return "N/A";
  return new Date(numeric * 1000).toLocaleString();
}

function formatDuration(totalSeconds) {
  const seconds = Number(totalSeconds);
  if (!Number.isFinite(seconds) || seconds <= 0) return "0s";

  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);

  const chunks = [];
  if (h > 0) chunks.push(`${h}h`);
  if (m > 0 || h > 0) chunks.push(`${m}m`);
  chunks.push(`${s}s`);
  return chunks.join(" ");
}

function formatPanelNumber(value) {
  const numeric = Number(value ?? 0);
  if (!Number.isFinite(numeric)) return "0";
  return numeric.toLocaleString();
}

const footerStatusLabel = computed(() => {
  if (storeMut.page === CHARACTERS_PAGE) {
    return `Online Players: ${formatPanelNumber(serverInfo.value?.onlinePlayers ?? 0)}`;
  }
  return store.launcherTag;
});

function formatCourses(courses) {
  if (!Array.isArray(courses) || !courses.length) return "None";
  return courses.map((course) => `${course.name} (#${course.id})`).join(", ");
}

function cacheDistributionEntriesFromStats(characters) {
  const nextEntriesByCharacterId = {};
  for (const character of Array.isArray(characters) ? characters : []) {
    const characterId = Number(character?.id ?? 0);
    if (!Number.isFinite(characterId) || characterId <= 0) continue;
    const characterKey = String(characterId);
    const entries = Array.isArray(character?.unclaimedDistributionDetails)
      ? character.unclaimedDistributionDetails
      : [];
    const total = Number(character?.unclaimedDistributions ?? 0);
    const existing = distributionEntriesByCharacterId.value[characterKey];
    nextEntriesByCharacterId[characterKey] =
      Array.isArray(existing) && existing.length > entries.length && existing.length <= total
        ? existing
        : entries.slice(0, DISTRIBUTION_PAGE_SIZE);
  }
  distributionEntriesByCharacterId.value = nextEntriesByCharacterId;
}

function mergeDistributionEntries(characterId, entries) {
  const characterKey = String(characterId);
  const existing = Array.isArray(distributionEntriesByCharacterId.value[characterKey])
    ? distributionEntriesByCharacterId.value[characterKey]
    : [];
  const seen = new Set(existing.map((entry) => Number(entry?.id ?? 0)).filter((id) => id > 0));
  const merged = existing.slice();
  for (const entry of Array.isArray(entries) ? entries : []) {
    const entryId = Number(entry?.id ?? 0);
    if (entryId > 0 && seen.has(entryId)) continue;
    if (entryId > 0) seen.add(entryId);
    merged.push(entry);
  }
  distributionEntriesByCharacterId.value = {
    ...distributionEntriesByCharacterId.value,
    [characterKey]: merged,
  };
}

async function refreshServerInfo() {
  serverInfoLoading.value = true;
  try {
    const next = await getAltClientStats();
    const nextCharacters = Array.isArray(next?.characters)
      ? next.characters.map((character) => ({
          ...character,
          id: Number(character?.id ?? 0),
          returning: Boolean(character?.returning),
          courses: Array.isArray(character?.courses) ? character.courses : [],
          timePlayed: Number(character?.timePlayed ?? 0),
          unreadMail: Number(character?.unreadMail ?? 0),
          unreadMailEntries: Array.isArray(character?.unreadMailEntries)
            ? character.unreadMailEntries
            : [],
          unclaimedDistributions: Number(character?.unclaimedDistributions ?? 0),
          unclaimedDistributionNames: Array.isArray(character?.unclaimedDistributionNames)
            ? character.unclaimedDistributionNames
            : [],
          unclaimedDistributionDetails: Array.isArray(character?.unclaimedDistributionDetails)
            ? character.unclaimedDistributionDetails
            : [],
        }))
      : [];
    cacheDistributionEntriesFromStats(nextCharacters);
    serverInfo.value = {
      ...EMPTY_ALT_CLIENT_STATS,
      ...next,
      events: {
        ...EMPTY_ALT_CLIENT_STATS.events,
        ...(next?.events ?? {}),
      },
      user: {
        ...EMPTY_ALT_CLIENT_STATS.user,
        ...(next?.user ?? {}),
      },
      enabledSpecialEvents: Array.isArray(next?.enabledSpecialEvents)
        ? next.enabledSpecialEvents
        : [],
      characters: nextCharacters,
      onlineFriends: Array.isArray(next?.onlineFriends)
        ? next.onlineFriends
            .map((friend) => ({
              cid: Number(friend?.cid ?? 0),
              id: Number(friend?.id ?? 0),
              name: String(friend?.name ?? "").trim(),
              serverId: Number(friend?.serverId ?? 0),
            }))
            .filter(
              (friend) =>
                Number.isFinite(friend.cid) &&
                friend.cid > 0 &&
                Number.isFinite(friend.id) &&
                friend.id > 0
            )
        : [],
    };
  } finally {
    serverInfoLoading.value = false;
  }
}

function closeAltPanels() {
  showServerInfoPanel.value = false;
  showMailPanel.value = false;
  showDistributionPanel.value = false;
  showFriendsPanel.value = false;
  showBookPanel.value = false;
  activeEventInfoId.value = null;
}

function resetSavedataPanelState() {
  bookPanelLoading.value = false;
  bookPanelReady.value = false;
  savedataPanelData.value = null;
}

async function ensureSavedataForBook() {
  if (!canUseSavedataPanels.value) return false;

  const characterId = activeCharacterId.value;
  if (!characterId) {
    bookPanelLoading.value = false;
    bookPanelReady.value = false;
    savedataPanelData.value = null;
    return false;
  }

  bookPanelLoading.value = true;
  bookPanelReady.value = false;

  const savedataVersion = currentSavedataVersionToken();
  const loaded = await prefetchAltCharacterSavedata(
    characterId,
    savedataVersion
  );
  activeCharacterHasSavedataVersion.value = await hasAltCharacterSavedataVersion(
    characterId,
    savedataVersion
  );
  let parsed = null;
  if (loaded && activeCharacterHasSavedataVersion.value) {
    parsed = await getAltCharacterSavedataView(
      characterId,
      store.currentEndpoint,
      savedataVersion
    );
  }

  const ready = Boolean(loaded && parsed);
  savedataPanelData.value = ready ? parsed : null;
  bookPanelLoading.value = false;
  bookPanelReady.value = ready;

  return ready;
}

async function toggleServerInfoPanel() {
  playSelect();
  if (!canShowSavedataPanels.value) {
    closeAltPanels();
    return;
  }

  const nextOpen = !showServerInfoPanel.value;
  showServerInfoPanel.value = nextOpen;
  activeEventInfoId.value = null;
  showBookPanel.value = false;
  showMailPanel.value = false;
  showDistributionPanel.value = false;
  showFriendsPanel.value = false;

  if (nextOpen) {
    await refreshServerInfo();
  }
}

async function toggleMailPanel() {
  playSelect();
  if (!canUseSavedataPanels.value) {
    closeAltPanels();
    return;
  }

  const nextOpen = !showMailPanel.value;
  showMailPanel.value = nextOpen;
  showServerInfoPanel.value = false;
  showDistributionPanel.value = false;
  showFriendsPanel.value = false;
  activeEventInfoId.value = null;
  showBookPanel.value = false;

  if (nextOpen) {
    await refreshServerInfo();
  }
}

async function toggleDistributionPanel() {
  playSelect();
  if (!canUseSavedataPanels.value) {
    closeAltPanels();
    return;
  }

  const nextOpen = !showDistributionPanel.value;
  showDistributionPanel.value = nextOpen;
  showServerInfoPanel.value = false;
  showMailPanel.value = false;
  showFriendsPanel.value = false;
  activeEventInfoId.value = null;
  showBookPanel.value = false;

  if (nextOpen) {
    await refreshServerInfo();
  }
}

async function toggleFriendsPanel() {
  playSelect();
  if (!canUseSavedataPanels.value) {
    closeAltPanels();
    return;
  }

  const nextOpen = !showFriendsPanel.value;
  showFriendsPanel.value = nextOpen;
  showServerInfoPanel.value = false;
  showMailPanel.value = false;
  showDistributionPanel.value = false;
  activeEventInfoId.value = null;
  showBookPanel.value = false;

  if (nextOpen) {
    await refreshServerInfo();
  }
}

async function loadMoreDistributionEntries() {
  const characterId = activeCharacterId.value;
  if (
    !Number.isFinite(characterId) ||
    characterId <= 0 ||
    distributionLoadingMore.value ||
    !hasMoreActiveAltDistributionEntries.value
  ) {
    return;
  }

  distributionLoadingMore.value = true;
  try {
    const page = await getAltClientDistributions(
      characterId,
      activeAltDistributionEntries.value.length,
      DISTRIBUTION_PAGE_SIZE
    );
    mergeDistributionEntries(characterId, page.entries);
  } finally {
    distributionLoadingMore.value = false;
  }
}

async function toggleBookPanel() {
  playSelect();
  if (!canUseSavedataPanels.value) return;

  if (!showBookPanel.value) {
    const hasVersion =
      canOpenBookPanel.value || (await refreshSavedataVersionAvailability());
    if (!hasVersion) return;
  }

  const nextOpen = !showBookPanel.value;
  showBookPanel.value = nextOpen;
  showServerInfoPanel.value = false;
  showMailPanel.value = false;
  showDistributionPanel.value = false;
  showFriendsPanel.value = false;
  activeEventInfoId.value = null;

  if (nextOpen) {
    await ensureSavedataForBook();
  }
}

async function refreshSavedataVersionAvailability() {
  const checkNonce = ++savedataVersionCheckNonce;

  if (!canShowSavedataPanels.value || !hasRealCharacter.value) {
    activeCharacterHasSavedataVersion.value = false;
    return false;
  }

  const characterId = activeCharacterId.value;
  if (!characterId) {
    activeCharacterHasSavedataVersion.value = false;
    return false;
  }

  const savedataVersion = currentSavedataVersionToken();
  let hasVersion = await hasAltCharacterSavedataVersion(
    characterId,
    savedataVersion
  );
  if (!hasVersion) {
    const prefetched = await prefetchAltCharacterSavedata(
      characterId,
      savedataVersion
    );
    if (checkNonce !== savedataVersionCheckNonce) {
      return activeCharacterHasSavedataVersion.value;
    }
    hasVersion =
      prefetched &&
      (await hasAltCharacterSavedataVersion(characterId, savedataVersion));
  }
  if (checkNonce !== savedataVersionCheckNonce) {
    return activeCharacterHasSavedataVersion.value;
  }

  activeCharacterHasSavedataVersion.value = hasVersion;
  if (!hasVersion) {
    showBookPanel.value = false;
    resetSavedataPanelState();
  }

  return hasVersion;
}
function onActiveCharacterChanged(payload) {
  const previousCharacterId = lastSavedataCharacterId.value;

  const rawId = Number(payload?.id ?? 0);
  selectedCharacterId.value = Number.isFinite(rawId) && rawId > 0 ? Math.trunc(rawId) : null;
  hasRealCharacter.value = Boolean(payload?.hasRealCharacter);
  lastSavedataCharacterId.value = selectedCharacterId.value;

  const characterChanged = previousCharacterId !== selectedCharacterId.value;

  if (!hasRealCharacter.value) {
    closeAltPanels();
    activeCharacterHasSavedataVersion.value = false;
    resetSavedataPanelState();
    return;
  }

  if (characterChanged) {
    resetSavedataPanelState();
  }

  void refreshServerInfo();

  void refreshSavedataVersionAvailability().then((hasVersion) => {
    if (!hasVersion) return;
    if (showBookPanel.value) {
      void ensureSavedataForBook();
    }
  });
}

function onGlobalMouseDown(event) {
  if (!anyAltPanelOpen.value) return;
  const target = event.target;
  if (!(target instanceof Node)) return;
  const roots = [
    serverInfoPanelRoot.value,
    eventPanelRoot.value,
    ps4MailRoot.value,
    mailOverlayRef.value?.contentRoot ?? null,
    distributionOverlayRef.value?.contentRoot ?? null,
    friendsOverlayRef.value?.contentRoot ?? null,
    bookOverlayRef.value?.contentRoot ?? null,
  ].filter(Boolean);
  if (!roots.some((root) => root.contains(target))) {
    closeAltPanels();
  }
}

function resolveControllerScope() {
  if (store.dialogOpen) return dialogScopeRef.value;
  if (showMailPanel.value) return mailOverlayRef.value?.contentRoot ?? null;
  if (showDistributionPanel.value) return distributionOverlayRef.value?.contentRoot ?? null;
  if (showFriendsPanel.value) return friendsOverlayRef.value?.contentRoot ?? null;
  if (showBookPanel.value) return bookOverlayRef.value?.contentRoot ?? null;
  const dropdownScope = launcherRoot.value?.querySelector?.(
    "[data-server-picker-open='true'] [data-controller-dropdown-scope='true']"
  );
  if (dropdownScope instanceof HTMLElement) return dropdownScope;
  const settingsDropdownScope = launcherRoot.value?.querySelector?.(
    "[data-settings-picker-open='true'] [data-controller-dropdown-scope='true']"
  );
  if (settingsDropdownScope instanceof HTMLElement) return settingsDropdownScope;
  return launcherRoot.value;
}

function sortNodeIdsByVisualOrder(nodeIds, helpers) {
  return [...nodeIds].sort((leftId, rightId) => {
    const leftRect = helpers.getNodeElement(leftId)?.getBoundingClientRect();
    const rightRect = helpers.getNodeElement(rightId)?.getBoundingClientRect();
    if (!leftRect && !rightRect) return 0;
    if (!leftRect) return 1;
    if (!rightRect) return -1;
    const topDelta = leftRect.top - rightRect.top;
    if (Math.abs(topDelta) > 8) return topDelta;
    return leftRect.left - rightRect.left;
  });
}

function getPs4MessageEntryNodeIds(helpers) {
  return sortNodeIdsByVisualOrder(
    helpers.getVisibleNodeIds("message-entry-"),
    helpers
  );
}

function getPs4VisibleLinkNodeIds(helpers) {
  return sortNodeIdsByVisualOrder(helpers.getVisibleNodeIds("link-"), helpers);
}

function enterPs4MessageBox(helpers, sourceNodeId) {
  helpers.state.messageSource = sourceNodeId;
  helpers.state.messageEntryMode = false;
  return "message_box";
}

function getPs4NearestLinkNode(helpers, sourceNodeId) {
  const linkNodeIds = getPs4VisibleLinkNodeIds(helpers);
  if (!linkNodeIds.length) return null;
  return helpers.findNearestNodeId(linkNodeIds, sourceNodeId) ?? linkNodeIds[0] ?? null;
}

function cycleNodeIds(nodeIds, currentNodeId, direction) {
  if (!nodeIds.length) return null;
  const currentIndex = nodeIds.indexOf(currentNodeId);
  if (currentIndex < 0) return nodeIds[0];
  const delta = direction === "left" || direction === "up" ? -1 : 1;
  return nodeIds[(currentIndex + delta + nodeIds.length) % nodeIds.length];
}

function getCharacterIconNodeIds(helpers) {
  return helpers.sortNodeIdsByLeft(
    [
      "character-distribution",
      "character-mail",
      "character-friends",
      "character-book",
    ].filter((nodeId) => helpers.getNodeElement(nodeId))
  );
}

function getCharacterEventNodeIds(helpers) {
  return helpers.sortNodeIdsByLeft(helpers.getVisibleNodeIds("character-event-"));
}

function activateCharacterStep(helpers, nodeId) {
  if (!helpers.activateNode(nodeId)) return null;
  window.requestAnimationFrame(() => {
    helpers.focusNode("character-card", { playSound: false });
  });
  return true;
}

const ps4CharacterPageGraph = {
  id: "ps4-character-page",
  initialNode: "character-card",
  getInitialState() {
    return {
      messageSource: "character-card",
      messageEntryMode: false,
    };
  },
  move({ direction, currentNodeId, ...helpers }) {
    if (helpers.state.messageEntryMode && currentNodeId?.startsWith("message-entry-")) {
      const entryNodeIds = getPs4MessageEntryNodeIds(helpers);
      const currentIndex = entryNodeIds.indexOf(currentNodeId);
      if (direction === "left") {
        helpers.state.messageEntryMode = false;
        return "message_box";
      }
      if (direction === "up" && currentIndex > 0) return entryNodeIds[currentIndex - 1];
      if (direction === "down" && currentIndex >= 0 && currentIndex < entryNodeIds.length - 1) {
        return entryNodeIds[currentIndex + 1];
      }
      return null;
    }

    if (currentNodeId?.startsWith("link-")) {
      const linkNodeIds = getPs4VisibleLinkNodeIds(helpers);
      const currentIndex = linkNodeIds.indexOf(currentNodeId);
      if (direction === "left" && currentIndex > 0) return linkNodeIds[currentIndex - 1];
      if (direction === "right" && currentIndex >= 0 && currentIndex < linkNodeIds.length - 1) {
        return linkNodeIds[currentIndex + 1];
      }
      if (direction === "down") return "message_box";
      return null;
    }

    if (currentNodeId?.startsWith("character-event-")) {
      const eventNodeIds = getCharacterEventNodeIds(helpers);
      if (direction === "left") {
        const currentIndex = eventNodeIds.indexOf(currentNodeId);
        if (currentIndex <= 0) return "settings";
        return eventNodeIds[currentIndex - 1];
      }
      if (direction === "right") return cycleNodeIds(eventNodeIds, currentNodeId, direction);
      if (direction === "up") return "message_box";
      return null;
    }

    const iconNodeIds = getCharacterIconNodeIds(helpers);
    if (iconNodeIds.includes(currentNodeId)) {
      if (direction === "left" || direction === "right") {
        return cycleNodeIds(iconNodeIds, currentNodeId, direction);
      }
      if (direction === "down") return "character-card";
      return null;
    }

    switch (currentNodeId) {
      case "character-card":
        if (direction === "left") return activateCharacterStep(helpers, "character-prev");
        if (direction === "right") return activateCharacterStep(helpers, "character-next");
        if (direction === "up") {
          return helpers.findNearestNodeId(iconNodeIds, "character-card") ?? iconNodeIds[0] ?? null;
        }
        if (direction === "down") return "character-create";
        return null;
      case "character-create":
        if (direction === "up") return "character-card";
        if (direction === "right") return "character-options";
        if (direction === "down") return "character-start";
        return null;
      case "character-options":
        if (direction === "left") return "character-create";
        if (direction === "right") return enterPs4MessageBox(helpers, "character-options");
        if (direction === "up") return "character-card";
        if (direction === "down") return "character-start";
        return null;
      case "character-start":
        if (direction === "up") return "character-create";
        if (direction === "right") return enterPs4MessageBox(helpers, "character-start");
        if (direction === "down") return "settings";
        return null;
      case "settings":
        if (direction === "up") return "character-start";
        if (direction === "right") return getCharacterEventNodeIds(helpers)[0] ?? null;
        return null;
      case "message_box":
        if (direction === "up") return getPs4NearestLinkNode(helpers, "message_box");
        if (direction === "down") return getCharacterEventNodeIds(helpers)[0] ?? "settings";
        if (direction === "left") return helpers.state.messageSource ?? "character-options";
        return null;
      case "banner":
        if (direction === "left") {
          if (effectiveBanners.value.length > 1) {
            playSelect();
            setBannerIndex(
              (bannerIndex.value - 1 + effectiveBanners.value.length) % effectiveBanners.value.length
            );
          }
          return { focus: "banner", playSound: false };
        }
        if (direction === "right") {
          if (effectiveBanners.value.length > 1) {
            playSelect();
            setBannerIndex((bannerIndex.value + 1) % effectiveBanners.value.length);
          }
          return { focus: "banner", playSound: false };
        }
        if (direction === "down") return "message_box";
        return null;
      default:
        return null;
    }
  },
  activate({ currentNodeId, ...helpers }) {
    if (currentNodeId !== "message_box") return false;
    const entryNodeIds = getPs4MessageEntryNodeIds(helpers);
    if (!entryNodeIds.length) return true;
    helpers.state.messageEntryMode = true;
    return helpers.focusNode(entryNodeIds[0], { playSound: true });
  },
  back({ ...helpers }) {
    if (!helpers.state.messageEntryMode) return false;
    helpers.state.messageEntryMode = false;
    return helpers.focusNode("message_box", { playSound: true });
  },
};

const ps4FrontPageGraph = {
  id: "ps4-front-page",
  getInitialState() {
    return {
      messageSource: "login",
      messageEntryMode: false,
    };
  },
  initialNode: "login",
  move({ direction, currentNodeId, ...helpers }) {
    const isOfflineMode = store.currentEndpoint?.url === "OFFLINEMODE";

    if (helpers.state.messageEntryMode && currentNodeId?.startsWith("message-entry-")) {
      const entryNodeIds = getPs4MessageEntryNodeIds(helpers);
      const currentIndex = entryNodeIds.indexOf(currentNodeId);
      if (direction === "left") {
        helpers.state.messageEntryMode = false;
        return "message_box";
      }
      if (direction === "up" && currentIndex > 0) {
        return entryNodeIds[currentIndex - 1];
      }
      if (direction === "down" && currentIndex >= 0 && currentIndex < entryNodeIds.length - 1) {
        return entryNodeIds[currentIndex + 1];
      }
      return null;
    }

    if (currentNodeId?.startsWith("link-")) {
      const linkNodeIds = getPs4VisibleLinkNodeIds(helpers);
      const currentIndex = linkNodeIds.indexOf(currentNodeId);
      if (direction === "left" && currentIndex > 0) {
        return linkNodeIds[currentIndex - 1];
      }
      if (direction === "right" && currentIndex >= 0 && currentIndex < linkNodeIds.length - 1) {
        return linkNodeIds[currentIndex + 1];
      }
      if (direction === "down") return "message_box";
      return null;
    }

    switch (currentNodeId) {
      case "login":
        if (isOfflineMode) {
          if (direction === "right") return enterPs4MessageBox(helpers, "login");
          if (direction === "down") return "server";
          return null;
        }
        if (direction === "up") return "server";
        if (direction === "right") return enterPs4MessageBox(helpers, "login");
        if (direction === "down") return "banner";
        return null;
      case "server":
        if (isOfflineMode) {
          if (direction === "right") return enterPs4MessageBox(helpers, "server");
          if (direction === "up") return "login";
          if (direction === "down") return "banner";
          return null;
        }
        if (direction === "right") return "register";
        if (direction === "up") return "username";
        if (direction === "down") return "login";
        return null;
      case "register":
        if (direction === "left") return "server";
        if (direction === "right") return enterPs4MessageBox(helpers, "register");
        if (direction === "up") return "remember";
        if (direction === "down") return "login";
        return null;
      case "remember":
        if (direction === "up") return "password";
        if (direction === "down") return "register";
        if (direction === "right") return enterPs4MessageBox(helpers, "remember");
        return null;
      case "password":
        if (direction === "left") return "username";
        if (direction === "right") return enterPs4MessageBox(helpers, "password");
        if (direction === "down") return "remember";
        return null;
      case "username":
        if (direction === "right") return "password";
        if (direction === "down") return "server";
        return null;
      case "message_box":
        if (direction === "up") {
          const linkNodeIds = getPs4VisibleLinkNodeIds(helpers);
          return linkNodeIds[0] ?? null;
        }
        if (direction === "left") return helpers.state.messageSource ?? "login";
        return null;
      case "banner":
        if (isOfflineMode) {
          if (direction === "up") return "server";
          if (direction === "down") return "settings";
          if (direction === "left") {
            if (effectiveBanners.value.length > 1) {
              playSelect();
              setBannerIndex(
                (bannerIndex.value - 1 + effectiveBanners.value.length) % effectiveBanners.value.length
              );
            }
            return { focus: "banner", playSound: false };
          }
          if (direction === "right") {
            if (effectiveBanners.value.length > 1) {
              playSelect();
              setBannerIndex((bannerIndex.value + 1) % effectiveBanners.value.length);
            }
            return { focus: "banner", playSound: false };
          }
          return null;
        }
        if (direction === "up") return "login";
        if (direction === "left") {
          if (effectiveBanners.value.length > 1) {
            playSelect();
            setBannerIndex(
              (bannerIndex.value - 1 + effectiveBanners.value.length) % effectiveBanners.value.length
            );
          }
          return { focus: "banner", playSound: false };
        }
        if (direction === "right") {
          if (effectiveBanners.value.length > 1) {
            playSelect();
            setBannerIndex((bannerIndex.value + 1) % effectiveBanners.value.length);
          }
          return { focus: "banner", playSound: false };
        }
        if (direction === "down") return "settings";
        return null;
      case "settings":
        if (direction === "up") return "banner";
        return null;
      default:
        return null;
    }
  },
  activate({ currentNodeId, ...helpers }) {
    if (currentNodeId !== "message_box") return false;
    const entryNodeIds = getPs4MessageEntryNodeIds(helpers);
    if (!entryNodeIds.length) return true;
    helpers.state.messageEntryMode = true;
    return helpers.focusNode(entryNodeIds[0], { playSound: true });
  },
  back({ ...helpers }) {
    if (!helpers.state.messageEntryMode) return false;
    helpers.state.messageEntryMode = false;
    return helpers.focusNode("message_box", { playSound: true });
  },
};

const SETTINGS_TAB_ORDER = [
  "settings-nav-launcher",
  "settings-nav-version",
  "settings-nav-settings",
  "settings-nav-graphics",
  "settings-nav-audio",
  "settings-nav-controls",
  "settings-nav-advanced",
];

const SETTINGS_NAV_ORDER = [
  ...SETTINGS_TAB_ORDER,
  "settings-nav-back",
];

function sortSettingsPaneElements(elements) {
  return [...elements].sort((left, right) => {
    const leftRect = left.getBoundingClientRect();
    const rightRect = right.getBoundingClientRect();
    const topDelta = leftRect.top - rightRect.top;
    if (Math.abs(topDelta) > 8) return topDelta;
    return leftRect.left - rightRect.left;
  });
}

function getActiveSettingsNavNodeId(scope) {
  return (
    scope
      ?.querySelector?.("[data-settings-active='true'][data-controller-node]")
      ?.getAttribute("data-controller-node") ?? "settings-nav-launcher"
  );
}

function getSettingsPaneElements(helpers) {
  const pane = helpers.scope?.querySelector?.("[data-controller-settings-pane='true']");
  if (!(pane instanceof HTMLElement)) return [];
  return sortSettingsPaneElements(
    helpers
      .getFocusableElements()
        .filter((element) => pane.contains(element) && !String(element?.dataset?.controllerNode ?? "").startsWith("settings-nav-"))
  );
}

function getExplicitControllerTargetId(element, direction) {
  if (!(element instanceof HTMLElement)) return null;
  const mapping = {
    up: element.dataset?.controllerUp,
    down: element.dataset?.controllerDown,
    left: element.dataset?.controllerLeft,
    right: element.dataset?.controllerRight,
  };
  return String(mapping[direction] ?? "").trim() || null;
}

function focusSettingsPaneElement(helpers, element, { playSound = true } = {}) {
  if (!(element instanceof HTMLElement)) return false;
  element.scrollIntoView?.({ block: "nearest" });
  return helpers.focusElement(element, { playSound });
}

const settingsPageGraph = {
  id: "settings-page",
  getInitialState() {
    return {
      paneMode: false,
    };
  },
  initialNode: "settings-nav-launcher",
  move({ direction, currentNodeId, focusedElement, ...helpers }) {
    const tabIndex = SETTINGS_TAB_ORDER.indexOf(currentNodeId);
    if (tabIndex >= 0) {
      if (direction === "left" && tabIndex > 0) {
        return SETTINGS_TAB_ORDER[tabIndex - 1];
      }
      if (direction === "right" && tabIndex < SETTINGS_TAB_ORDER.length - 1) {
        return SETTINGS_TAB_ORDER[tabIndex + 1];
      }
      if (direction === "down") {
        const paneElements = getSettingsPaneElements(helpers);
        if (paneElements.length > 0) {
          return focusSettingsPaneElement(helpers, paneElements[0], { playSound: true });
        }
        return "settings-nav-back";
      }
      return null;
    }

    if (currentNodeId === "settings-nav-back") {
      if (direction === "up" || direction === "right") {
        return getActiveSettingsNavNodeId(helpers.scope);
      }
      return null;
    }

    const paneElements = getSettingsPaneElements(helpers);
    const currentIndex = paneElements.indexOf(focusedElement);
    if (currentIndex < 0) return null;

    const explicitTargetId = getExplicitControllerTargetId(focusedElement, direction);
    if (explicitTargetId) {
      if (explicitTargetId === "__block__") {
        return null;
      }
      const explicitTarget = helpers.getNodeElement(explicitTargetId);
      if (explicitTarget instanceof HTMLElement) {
        return focusSettingsPaneElement(helpers, explicitTarget, { playSound: true });
      }
    }

    if (
      !(focusedElement instanceof HTMLSelectElement) &&
      (direction === "left" || direction === "right") &&
      helpers.adjustFocused(direction)
    ) {
      return true;
    }
    if (direction === "up" && currentIndex > 0) {
      return focusSettingsPaneElement(helpers, paneElements[currentIndex - 1], { playSound: true });
    }
    if (direction === "down" && currentIndex < paneElements.length - 1) {
      return focusSettingsPaneElement(helpers, paneElements[currentIndex + 1], { playSound: true });
    }
    return null;
  },
  activate({ currentNodeId, ...helpers }) {
    const sidebarIndex = SETTINGS_NAV_ORDER.indexOf(currentNodeId);
    if (sidebarIndex < 0) return false;

    if (currentNodeId === "settings-nav-back") {
      return helpers.activateNode(currentNodeId);
    }

    helpers.state.paneMode = true;
    helpers.activateNode(currentNodeId);
    window.requestAnimationFrame(() => {
      const paneElements = getSettingsPaneElements(helpers);
      if (paneElements.length > 0) {
        focusSettingsPaneElement(helpers, paneElements[0], { playSound: true });
      }
    });
    return true;
  },
  back({ focusedElement, ...helpers }) {
    const paneElements = getSettingsPaneElements(helpers);
    if (paneElements.includes(focusedElement)) {
      helpers.state.paneMode = false;
      return helpers.focusNode(getActiveSettingsNavNodeId(helpers.scope), {
        playSound: true,
      });
    }
    return false;
  },
};

function resolveControllerGraph() {
  if (store.dialogOpen || localePicker.value) return null;
  if (anyAltPanelOpen.value || characterLoadingActive.value) return null;
  if (launcherRoot.value?.querySelector?.("[data-server-picker-open='true']")) return null;
  if (launcherRoot.value?.querySelector?.("[data-settings-picker-open='true']")) return null;
  if (storeMut.page === SETTINGS_PAGE) return settingsPageGraph;
  if (storeMut.page === CHARACTERS_PAGE) return ps4CharacterPageGraph;
  if (storeMut.page !== LOGIN_PAGE) return null;
  return ps4FrontPageGraph;
}

function onControllerBack() {
  if (store.dialogOpen) {
    closeDialog();
    return;
  }
  const openServerPickerToggle = launcherRoot.value?.querySelector?.(
    "[data-server-picker-open='true'] [data-controller-server-toggle='true']"
  );
  if (openServerPickerToggle instanceof HTMLElement && typeof openServerPickerToggle.click === "function") {
    openServerPickerToggle.click();
    return;
  }
  const openSettingsPickerToggle = launcherRoot.value?.querySelector?.(
    "[data-settings-picker-open='true'] [data-controller-settings-toggle='true']"
  );
  if (openSettingsPickerToggle instanceof HTMLElement && typeof openSettingsPickerToggle.click === "function") {
    openSettingsPickerToggle.click();
    return;
  }
  if (showBookPanel.value || anyAltPanelOpen.value) {
    closeAltPanels();
    return;
  }
  if (localePicker.value) {
    localePicker.value = false;
    return;
  }
  if (storeMut.page === SETTINGS_PAGE) {
    onSettingsBack();
  }
}

function onSettingsClick() {
  playSelect();
  onSettingsButton();
}

function onSettingsBack() {
  onSettingsButton();
}

function onNameserverClick() {
  playSelect();
  storeMut.page = LOGIN_PAGE
}

async function onbanlinkClick(url) {
  playSelect();
  const canOpen = await confirmExternalLinkOpen(url);
  if (!canOpen) return;
  open(url);
}

function onDotClick(i) {
  playSelect();
  setBannerIndex(i);
}

async function onLinkClick(url) {
  playSelect();
  const canOpen = await confirmExternalLinkOpen(url);
  if (!canOpen) return;
  open(url);
}

function onCloseClick() {
  playSelect();
  closeLauncher();
}

async function onMaximizeClick() {
  playSelect();
  await appWindow.toggleMaximize();
}

// called from template to capture element refs of v-for
function setLinkRef(i, el) {
  linkRefs.value[i] = el;
}

// bind hover to a single element
function bindHover(el) {
  return el ? bindSfx(el, { hover: true, click: null }) : null;
}

// (re)bind all link hover handlers
async function bindLinks() {
  // clear old
  linkUnbinders.forEach(fn => fn && fn());
  linkUnbinders = [];
  await nextTick();
  linkRefs.value.forEach(el => {
    const isDisabledButton = el instanceof HTMLButtonElement && el.disabled;
    const isClickable = el?.dataset?.controllerClickable === "true" || el instanceof HTMLButtonElement;
    if (el && isClickable && !isDisabledButton) {
      linkUnbinders.push(bindHover(el));
    }
  });
}

// rebind for single refs (settings & banner)
async function bindSingles() {
  await nextTick();
  if (unbindSettings) unbindSettings();
  if (settingsBtn.value) unbindSettings = bindHover(settingsBtn.value);

  if (unbindBanner) unbindBanner();
  if (bannerImg.value) unbindBanner = bindHover(bannerImg.value);
}

onMounted(() => {
  bindSingles();
  bindLinks();
  document.addEventListener("mousedown", onGlobalMouseDown);
  window.addEventListener("mezeporta:test-alt-stats-changed", refreshServerInfo);
  window.addEventListener("resize", syncPs4NewsScrollMetrics);
  nextTick(() => syncPs4NewsScrollMetrics());
});

watch(
  () => [storeMut.page, currentBanner.value?.src, store.links.length],
  () => {
    bindSingles();
    bindLinks();
  },
  { flush: "post" }
);


watch(
  () => storeMut.page,
  (page) => {
    if (page !== PATCHER_PAGE) {
      resetPatcherLogLines();
    }
  },
  { immediate: true }
);

watch(
  () => [storeMut.page, store.patcher.queueNoticePosition, store.patcher.state],
  ([page, queuePosition, patcherState]) => {
    if (page !== PATCHER_PAGE) return;

    const numericQueuePosition = Number(queuePosition || 0);
    if (numericQueuePosition <= 0) return;

    const logState =
      patcherState === CHECKING_PATCHER ? CHECKING_PATCHER : PATCHER_QUEUE_LOG_STATE;

    upsertPatcherLogLine(
      logState,
      getMessage("patcher-queue", {
        position: numericQueuePosition,
      })
    );
  },
  { immediate: true }
);

watch(
  () => [storeMut.page, store.patcher.state, patcherLog.value],
  ([page, state, line]) => {
    if (page !== PATCHER_PAGE) return;
    if (!line) return;

    const isActiveState =
      state === CHECKING_PATCHER ||
      state === DOWNLOADING_PATCHER ||
      state === RESTORING_PATCHER ||
      state === PATCHING_PATCHER;

    if (isActiveState) {
      patcherRunHasStarted = true;
    }

    if (state === DONE_PATCHER && !patcherRunHasStarted) {
      return;
    }

    upsertPatcherLogLine(state, line);

    if (state === DONE_PATCHER) {
      patcherRunHasStarted = false;
    }
  },
  { immediate: true }
);


watch(
  () => [store.currentEndpoint?.name, storeMut.page],
  async ([, page]) => {
    const isCharacterPage = page === CHARACTERS_PAGE;
    clearAltSavedataPanelCache();
    if (!isCharacterPage) {
      closeAltPanels();
      resetSavedataPanelState();
      activeCharacterHasSavedataVersion.value = false;
      return;
    }

    await refreshServerInfo();

    const hasSavedataVersion = await refreshSavedataVersionAvailability();
    if (!hasSavedataVersion) {
      return;
    }
    if (showBookPanel.value) {
      void ensureSavedataForBook();
    }
  },
  { immediate: true }
);

watch(
  () => [canShowSavedataPanels.value, activeCharacterId.value],
  ([canShowPanels, characterId], [previousCanShowPanels, previousCharacterId]) => {
    if (!canShowPanels) return;
    if (canShowPanels !== previousCanShowPanels || characterId !== previousCharacterId) {
      void refreshServerInfo();
      void refreshSavedataVersionAvailability().then((hasVersion) => {
        if (hasVersion && showBookPanel.value) {
          void ensureSavedataForBook();
        }
      });
    }
  }
);

watch(recentLog, (nextLog, previousLog) => {
  if (!nextLog || nextLog === previousLog) return;
  ps4ToastCompanion.value = pickPs4ToastCompanion();
});

watch(activeAltUnreadMail, (unread) => {
  if (unread <= 0) {
    showMailPanel.value = false;
  }
});

watch(activeAltUnclaimedDistributions, (count) => {
  if (count <= 0) {
    showDistributionPanel.value = false;
  }
});
onUnmounted(() => {
  if (unbindSettings) unbindSettings();
  if (unbindBanner)   unbindBanner();
  resetPatcherLogLines();
  linkUnbinders.forEach(fn => fn && fn());
  document.removeEventListener("mousedown", onGlobalMouseDown);
  window.removeEventListener("mezeporta:test-alt-stats-changed", refreshServerInfo);
  window.removeEventListener("resize", syncPs4NewsScrollMetrics);
  clearAltSavedataPanelCache();
});
// Locale picker
const localePicker = ref(false);
function openLocalePicker() { openPicker(localePicker); }
function onLocaleSelect(locale) {
  storeMut.locale = String(locale ?? "en").toLowerCase();
  localePicker.value = false;
}

useLauncherGamepad({
  enabled: computed(() => Boolean(store.settings.launcherController)),
  resolveScope: resolveControllerScope,
  resolveGraph: resolveControllerGraph,
  onBack: onControllerBack,
});

// Messages split
const messages = computed(() => {
  let announcements = [];
  let news = [];
  for (const m of effectiveMessages.value) {
    (m.kind === 1 ? announcements : news).push(m);
  }
  for (const m of store.remoteMessages) {
    (m.kind === 1 ? announcements : news).push(m);
  }
  announcements.sort((a, b) => b.date - a.date);
  news.sort((a, b) => b.date - a.date);
  return { announcements, news };
});

watch(
  () => [
    storeMut.page,
    messages.value.announcements.length,
    messages.value.news.length,
  ],
  async () => {
    await nextTick();
    syncPs4NewsScrollMetrics();
  },
  { immediate: true }
);

const srvFocused = { name: false, url: false, lport: false, gport: false };

function syncServerDialogNameDownNode(key) {
  if (!store.editEndpointNew) return;
  if (key === "url") serverDialogNameDownNode.value = "server-dialog-host";
  if (key === "lport") serverDialogNameDownNode.value = "server-dialog-launcher-port";
  if (key === "gport") serverDialogNameDownNode.value = "server-dialog-game-port";
}

function onSrvFocus(key) {
  if (!srvFocused[key]) {
    playSelect();
    srvFocused[key] = true;
  }
  syncServerDialogNameDownNode(key);
}
function onSrvBlur(key) {
  srvFocused[key] = false;
}

function onSrvControllerNavFocus(key) {
  syncServerDialogNameDownNode(key);
}

let lastSrvKeyTs = 0;
function srvTypeSfx(e) {
  // ignore modifier keys & repeats
  if (
    e.repeat ||
    e.key === 'Shift'   ||
    e.key === 'Control' ||
    e.key === 'Alt'     ||
    e.key === 'Meta'
  ) return;

  const now = performance.now();
  if (now - lastSrvKeyTs < 45) return;
  lastSrvKeyTs = now;
  playHover();
  forceRepaint(e.target);
}

function repaintSrvInput(el) {
  if (!el || document.activeElement === el) return;
  forceRepaint(el);
}

watch(
  () => [store.dialogKind, store.editEndpointNew],
  ([dialogKind, editEndpointNew]) => {
    if (dialogKind === SERVERS_DIALOG && editEndpointNew) {
      serverDialogNameDownNode.value = "server-dialog-host";
    }
  },
  { immediate: true }
);

watch(
  () => storeMut.editEndpoint?.name,
  async () => {
    await nextTick();
    repaintSrvInput(srvNameEl.value);
  }
);

watch(
  () => storeMut.editEndpoint?.url,
  async () => {
    await nextTick();
    repaintSrvInput(srvUrlEl.value);
  }
);

watch(
  () => storeMut.editEndpoint?.launcherPort,
  async () => {
    await nextTick();
    repaintSrvInput(srvLportEl.value);
  }
);

watch(
  () => storeMut.editEndpoint?.gamePort,
  async () => {
    await nextTick();
    repaintSrvInput(srvGportEl.value);
  }
);
</script>

  <template>
    <div ref="launcherRoot" class="h-full w-full ps4-root" :class="storeMut.locale" :style="ps4RootStyle">
      <div class="ps4-shell">
        <Transition name="ps4-settings">
          <div v-show="storeMut.page === SETTINGS_PAGE" class="ps4-settings-page flex flex-col min-h-0">
            <Settings @back="onSettingsBack"></Settings>
          </div>
        </Transition>
        <div v-show="storeMut.page !== SETTINGS_PAGE">
          <div class="ps4-header">
          <img
            draggable="false"
            class="ps4-logo"
            :src="launcherHeaderUrl"
          />

          <div class="ps4-top-links">
            <button
              v-for="(link, i) in ps4TopLinks"
              :key="link.link || `ps4-link-${i}`"
              class="ps4-top-link"
              :class="{ 'ps4-top-link-disabled': !link.link }"
              :ref="el => setLinkRef(i, el)"
              :disabled="!link.link"
              :data-controller-node="link.link ? `link-${i}` : null"
              data-controller-size="small"
              :title="link.name"
              @click="link.link ? onLinkClick(link.link) : null"
            >
              <span class="ps4-top-link-icon-wrap">
                <img
                  class="ps4-top-link-icon"
                  draggable="false"
                  :src="link.icon || assetUrl('/ps4/icon-inquiry.png')"
                  loading="lazy"
                />
              </span>
              <span class="ps4-top-link-text font-main">{{ link.name }}</span>
            </button>
          </div>

          <div class="ps4-header-rail">
            <div class="ps4-header-rail-title font-main">{{ ps4LauncherTitle }}</div>
            <img
              class="ps4-header-rail-emblem"
              :src="ps4EmblemUrl"
              draggable="false"
            />
            <div class="ps4-header-rail-version font-main">{{ ps4VersionLabel }}</div>
          </div>
        </div>

        <div class="ps4-main">
          <div
            v-if="storeMut.page === CHARACTERS_PAGE"
            class="ps4-connection-rail text-[12px] text-[#c7cfc8]"
          >
            {{ storeMut.username }}@{{ store.currentEndpoint.name }} |
            <span class="cursor-pointer hover:text-white" @mouseenter="playHover()" @click="onNameserverClick">
              Disconnect
            </span>
          </div>

          <div class="ps4-left-column">
            <div
              v-if="storeMut.page === CHARACTERS_PAGE"
              ref="ps4MailRoot"
              class="ps4-mail-shell"
              :class="{ 'ps4-mail-shell-loading': characterLoadingActive }"
            >
              <TransitionGroup
                name="character-icon-pop"
                tag="div"
                class="ps4-mail-shell-icons"
              >
              <button
                v-if="canOpenBookPanel"
                key="ps4-book"
                class="ps4-mail-button ps4-book-button"
                :class="{ 'ps4-book-button-active': showBookPanel }"
                data-controller-node="character-book"
                data-controller-size="small"
                data-controller-priority="18"
                @mouseenter="playHover()"
                @click="toggleBookPanel"
                title="Book"
              >
                <img
                  :src="assetUrl('/extra/book.png')"
                  class="ps4-mail-button-image ps4-book-button-image-closed"
                  draggable="false"
                />
                <img
                  :src="assetUrl('/extra/bookOpen.png')"
                  class="ps4-mail-button-image ps4-book-button-image-open"
                  draggable="false"
                />
              </button>
              <button
                v-if="showFriendsButton"
                key="ps4-friends"
                class="ps4-mail-button ps4-friends-button"
                :class="{
                  'ps4-friends-button-active': showFriendsPanel,
                  'ps4-friends-button-no-count': activeAltOnlineFriendCount <= 0,
                }"
                data-controller-node="character-friends"
                data-controller-size="small"
                data-controller-priority="15"
                @mouseenter="playHover()"
                @click="toggleFriendsPanel"
                title="Friends"
              >
                <img
                  :src="assetUrl('/extra/Face.png')"
                  class="ps4-mail-button-image ps4-friends-button-image-closed"
                  draggable="false"
                />
                <img
                  :src="assetUrl('/extra/FaceHighlight.png')"
                  class="ps4-mail-button-image ps4-friends-button-image-open"
                  draggable="false"
                />
                <span
                  v-if="activeAltOnlineFriendCount > 0"
                  class="ps4-friends-button-count"
                >
                  {{ activeAltOnlineFriendCount > 99 ? "99+" : activeAltOnlineFriendCount }}
                </span>
              </button>
              <button
                v-if="showPs4MailButton"
                key="ps4-mail"
                class="ps4-mail-button"
                :class="{ 'ps4-mail-button-active': showMailPanel }"
                data-controller-node="character-mail"
                data-controller-size="small"
                data-controller-priority="17"
                @mouseenter="playHover()"
                @click="toggleMailPanel"
                title="Mail"
              >
                <img
                  :src="assetUrl('/extra/MailClosed.png')"
                  class="ps4-mail-button-image ps4-mail-button-image-closed"
                  draggable="false"
                />
                <img
                  :src="assetUrl('/extra/MailOpen.png')"
                  class="ps4-mail-button-image ps4-mail-button-image-open"
                  draggable="false"
                />
                <span class="ps4-mail-button-counts">
                  {{ activeAltUnreadMail > 99 ? "99+" : activeAltUnreadMail }}
                </span>
              </button>
              <button
                v-if="showDistributionButton"
                key="ps4-distribution"
                class="ps4-mail-button ps4-distribution-button"
                :class="{ 'ps4-distribution-button-active': showDistributionPanel }"
                data-controller-node="character-distribution"
                data-controller-size="small"
                data-controller-priority="16"
                @mouseenter="playHover()"
                @click="toggleDistributionPanel"
                title="Distribution"
              >
                <img
                  :src="assetUrl('/extra/ChestClosed.png')"
                  class="ps4-mail-button-image ps4-distribution-button-image-closed"
                  draggable="false"
                />
                <img
                  :src="assetUrl('/extra/ChestOpen.png')"
                  class="ps4-mail-button-image ps4-distribution-button-image-open"
                  draggable="false"
                />
                <span class="ps4-distribution-button-count">
                  {{ activeAltUnclaimedDistributions > 99 ? "99+" : activeAltUnclaimedDistributions }}
                </span>
              </button>
              </TransitionGroup>
            </div>
            <div class="ps4-stage-card">
              <Characters
                v-show="storeMut.page === CHARACTERS_PAGE"
                @loading-log="onCharacterLoadingLog"
                @active-character="onActiveCharacterChanged"
              ></Characters>

              <template v-if="storeMut.page !== CHARACTERS_PAGE">
                <Login v-show="storeMut.page === LOGIN_PAGE"></Login>
                <Patcher v-show="storeMut.page === PATCHER_PAGE"></Patcher>
              </template>
            </div>

            <div class="ps4-banner-card">
              <div class="ps4-banner-title-shell">
                <img
                  class="ps4-banner-divider"
                  :src="assetUrl('/ps4/msg-line-banner.png')"
                  draggable="false"
                />
                <div class="ps4-banner-title font-main">Event / Promotion Campaign</div>
              </div>
              <div class="ps4-banner-shell">
                <img
                  ref="bannerImg"
                  class="ps4-banner-image cursor-pointer"
                  :src="currentBanner?.src"
                  draggable="false"
                  data-controller-node="banner"
                  data-controller-clickable="true"
                  data-controller-size="big"
                  tabindex="0"
                  @click="onbanlinkClick(currentBanner?.link)"
                />
              </div>
              <div class="ps4-banner-dots">
                <button
                  v-for="(_, i) in effectiveBanners"
                  class="ps4-banner-dot hover:bg-[#9da7b9]"
                  :class="i === bannerIndex ? 'bg-[#7eccff]' : 'bg-[#5b646f]'"
                  @click="onDotClick(i)"
                ></button>
              </div>
            </div>
          </div>

          <div class="ps4-news-panel">
            <div class="ps4-news-scroll-shell">
              <div
                ref="ps4NewsScrollEl"
                class="ps4-news-scroll ps4-message-box scrollbar"
                data-controller-node="message_box"
                data-controller-clickable="true"
                data-controller-size="big"
                tabindex="0"
                @scroll="syncPs4NewsScrollMetrics"
              >
                <MessageList
                  :messages="messages.announcements"
                  :title="$t('announcements-label')"
                  :important="true"
                  node-prefix="message-entry-ann"
                ></MessageList>
                <MessageList
                  :messages="messages.news"
                  :title="$t('news-label')"
                  node-prefix="message-entry-news"
                ></MessageList>
              </div>
              <div v-if="ps4NewsHasOverflow" class="ps4-news-scrollbar">
                <button
                  type="button"
                  class="ps4-news-scrollbar-button ps4-news-scrollbar-up"
                  :disabled="!ps4NewsCanScrollUp"
                  @mouseenter="playHover()"
                  @click="scrollPs4News(-140)"
                ></button>
                <div class="ps4-news-scrollbar-track">
                  <div class="ps4-news-scrollbar-thumb" :style="ps4NewsThumbStyle"></div>
                </div>
                <button
                  type="button"
                  class="ps4-news-scrollbar-button ps4-news-scrollbar-down"
                  :disabled="!ps4NewsCanScrollDown"
                  @mouseenter="playHover()"
                  @click="scrollPs4News(140)"
                ></button>
              </div>
            </div>
          </div>
        </div>

        <div class="ps4-footer">
          <div class="ps4-footer-brand" :class="{ 'ps4-footer-brand-settings': storeMut.page === SETTINGS_PAGE }">
            <template v-if="storeMut.page !== SETTINGS_PAGE">
              <img
                :src="capcomUrl"
                class="ps4-footer-logo ps4-footer-capcom"
                draggable="false"
                loading="lazy"
              />
              <img
                :src="cogUrl"
                class="ps4-footer-logo ps4-footer-cog"
                draggable="false"
                loading="lazy"
              />
            </template>
            <transition name="footer-status-crossfade" mode="out-in">
              <div :key="footerStatusLabel" class="ps4-footer-tag font-main">{{ footerStatusLabel }}</div>
            </transition>
          </div>
          <button
            v-if="storeMut.page !== SETTINGS_PAGE"
            ref="settingsBtn"
            class="ps4-settings-button"
            data-controller-node="settings"
            data-controller-size="big"
            @click="onSettingsClick()"
            title="Settings"
          ></button>
          <div class="grow"></div>
          <div v-if="canShowServerInfo" ref="serverInfoPanelRoot" class="ps4-footer-panels relative flex items-center gap-[12px]">
        <div ref="eventPanelRoot" class="relative flex items-center gap-[8px]">
          <TransitionGroup
            name="character-icon-pop"
            tag="div"
            class="relative flex items-center gap-[8px]"
          >
          <button
            v-for="(eventBadge, eventIndex) in activeEventBadges"
            :key="`ps4-event-${eventBadge.id}`"
            type="button"
            class="ps4-event-badge-button"
            :data-controller-node="`character-event-${eventIndex}`"
            data-controller-size="small"
            :data-controller-priority="14 + eventIndex"
            @mouseenter="playHover()"
            @click="toggleEventInfoPanel(eventBadge.id)"
            :title="eventBadge.label"
          >
            <img
              :src="eventBadge.image"
              class="ps4-event-badge-image"
              draggable="false"
              alt=""
            />
          </button>
          </TransitionGroup>
          <transition name="event-info-pop">
            <div
              v-if="activeEventInfo && canShowSavedataPanels"
              class="eventpaneltext rounded-sm"
            >
              <div class="mb-2 eventpaneltextLabel">{{ activeEventInfo.label }}</div>
              <div class="grid grid-cols-[104px_1fr] gap-y-1 gap-x-2">
                <template
                  v-for="([label, value], rowIndex) in activeEventInfo.rows"
                  :key="`ps4-event-row-${activeEventInfo.id}-${rowIndex}`"
                >
                  <div>{{ label }}</div>
                  <div class="break-words">{{ value }}</div>
                </template>
              </div>
            </div>
          </transition>
        </div>
          </div>
        </div>
      </div>
    </div>
    <CharacterBookOverlay
      ref="bookOverlayRef"
      :open="showBookPanel"
      :loading="serverInfoLoading"
      :ready="bookPanelReady"
      :has-character="Boolean(activeCharacterId)"
      :character-name="activeCharacterName"
      :returning="activeAltCharacter?.returning ?? false"
      :time-played-formatted="activeAltCharacter?.timePlayed != null ? formatDuration(activeAltCharacter.timePlayed) : ''"
      :courses-list="activeAltCourses"
      :premium-currencies="activeAltUser"
      :savedata="savedataPanelData"
      @close="closeAltPanels"
    />
    <MailOverlay
      ref="mailOverlayRef"
      :open="showMailPanel"
      :loading="serverInfoLoading"
      :has-character="Boolean(activeCharacterId)"
      :entries="activeAltUnreadMailEntries"
      @close="closeAltPanels"
    />
    <DistributionOverlay
      ref="distributionOverlayRef"
      :open="showDistributionPanel"
      :loading="serverInfoLoading"
      :has-character="Boolean(activeCharacterId)"
      :entries="activeAltDistributionEntries"
      :has-more="hasMoreActiveAltDistributionEntries"
      :loading-more="distributionLoadingMore"
      @close="closeAltPanels"
      @load-more="loadMoreDistributionEntries"
    />
    <FriendsOverlay
      ref="friendsOverlayRef"
      :open="showFriendsPanel"
      :loading="serverInfoLoading"
      :has-character="Boolean(activeCharacterId)"
      :entries="activeAltFriendEntries"
      @close="closeAltPanels"
    />
  <div
    v-if="recentLog"
    class="fixed bottom-4 right-4 z-[220]"
    @click="dismissRecentLog"
  >
    <div class="alert cursor-pointer py-2 px-3 text-[16px] flex items-center gap-3 min-w-[360px]" :class="alertClass[recentLog.level]">
      <span class="grow text-black font-medium">{{ recentLog.message }}</span>
      <img
        :src="ps4ToastCompanion"
        class="h-[54px] w-auto shrink-0 object-contain"
        draggable="false"
      />
    </div>
  </div>
  <div class="ps4-window-bar">
    <div data-tauri-drag-region class="ps4-window-drag"></div>
    <div class="ps4-window-locale">
      <div
        class="locale-picker flex flex-col bg-[#00000099] w-max leading-5 text-sm uppercase cursor-pointer"
      >
        <div
          class="flex w-[60px] hover:bg-[#1b1b1b99]"
          data-controller-clickable="true"
          data-controller-size="small"
          tabindex="0"
          @click="openLocalePicker"
        >
          <img
            class="w-[16px] ml-2"
            :src="assetUrl(`/flags/${storeMut.locale}.svg`)"
            draggable="false"
          />
          <span class="ml-2">{{ storeMut.locale }}</span>
        </div>
        <template v-if="localePicker">
          <template v-for="l in availableLocales">
            <template v-if="l !== storeMut.locale">
              <div
                class="flex w-[60px] hover:bg-[#1b1b1b99]"
                data-controller-clickable="true"
                data-controller-size="small"
                tabindex="0"
                @click="onLocaleSelect(l)"
              >
                <img
                  class="w-[16px] ml-2"
                  :src="assetUrl(`/flags/${l}.svg`)"
                  draggable="false"
                />
                <span class="ml-2">{{ l }}</span>
              </div>
            </template>
          </template>
        </template>
      </div>
    </div>
    <button type="button" class="ps4-window-button" @click="appWindow.minimize">
      <img
        :src="assetUrl('/ps4/minimize.png')"
        class="ps4-window-control-image state-img"
        draggable="false"
      />
    </button>
    <button type="button" class="ps4-window-button" @click="onMaximizeClick">
      <img
        :src="assetUrl('/ps4/Maximize.png')"
        class="ps4-window-control-image state-img"
        draggable="false"
      />
    </button>
    <button type="button" class="ps4-window-button" @click="onCloseClick">
      <img
        :src="assetUrl('/ps4/close.png')"
        class="ps4-window-control-image state-img"
        draggable="false"
      />
    </button>
  </div>
  <dialog
    :open="store.dialogOpen"
    @close="closeDialog"
    class="absolute top-0 h-full w-full bg-transparent z-[1100]"
  >
    <div v-if="store.dialogOpen" class="relative flex items-center h-full">
      <div
        class="absolute inset-0 bg-black/55"
        :style="overlayBackdropStyle"
      ></div>
      <div
        ref="dialogScopeRef"
        class="relative z-[1] bg-contain flex flex-col items-center m-auto news-default gap-1 px-14"
        :style="{
          backgroundImage: `url('${dialogBackgroundAsset}')`,
        }"
        :class="
          store.dialogKind === DELETE_DIALOG
            ? 'w-[560px] h-[320px] pt-[90px]'
            : 'w-[700px] h-[400px] pt-[112px]'
        "
      >
        <div
          v-if="store.dialogKind !== SERVERS_DIALOG"
          class="launcher-dialog-info-panel"
        >
        <template
          v-if="store.dialogKind === DELETE_DIALOG && store.deleteCharacter"
          class=""
        >
          <div class="text-xl">
            {{ $t("delete-character-label") }}
          </div>
          <div class="warning">
            {{
              $t("delete-character-confirmation", {
                character_name: store.deleteCharacter.name,
              })
            }}
          </div>
        </template>
        <template v-else-if="store.dialogKind === PATCHER_DIALOG">
          <div class="text-xl text-center">
            {{ $t("patcher-updates-label") }}
          </div>
          <div class="text-center" v-html="$t('patcher-updates-confirmation')"></div>
        </template>
        <template v-else-if="store.dialogKind === SERVER_SWITCH_DIALOG">
          <div class="text-xl text-center">
            {{ $t("server-switch-label") }}
          </div>
          <div class="text-center" v-html="store.dialogMessage"></div>
        </template>
        <template v-else-if="store.dialogKind === VERSION_SWITCH_DIALOG">
          <div class="text-xl text-center versionchecklabel">
            {{ $t("version-switch-label", "Version Check") }}
          </div>
          <div class="text-center versioncheckbody" v-html="store.dialogMessage"></div>
        </template>
        <template v-else-if="store.dialogKind === EXTERNAL_LINK_DIALOG">
          <div class="text-xl">
            {{ $t("external-link-open-title", "Open external link") }}
          </div>
          <div class="text-center px-2">
            {{ $t("external-link-open-message", "This will open in your browser. Continue?") }}
          </div>
          <div class="text-xs opacity-70 break-all text-center px-2">{{ store.dialogMessage }}</div>
        </template>
        <template v-else-if="store.dialogKind === RESET_PATCH_DIALOG">
          <div class="text-xl text-center">
            {{ $t("reset-patch-label") }}
          </div>
          <div class="text-center px-2 pb-3">
            {{
              store.dialogMessage ||
              $t(
                "reset-patch-confirmation",
                "Restore all patched files back to original for this game folder?"
              )
            }}
          </div>
          <div v-if="store.resetPatchCompleted" class="restore-felyne-complete">
            <img
              :src="assetUrl('/extra/RestoreFelyne.png')"
              class="restore-felyne-image"
              draggable="false"
              alt=""
            />
          </div>
          <div class="relative mt-4 h-[18px] w-[318px]" v-else-if="store.dialogLoading">
            <img :src="assetUrl('/ps4/bar_frame.png')" class="absolute left-0 top-0" />
            <img
              :src="assetUrl('/ps4/bar.jpg')"
              class="absolute left-[8px] top-[5px] h-[6px] w-[302px] object-left"
            />
            <div
              class="absolute top-[5px] right-[8px] h-[6px] bg-black"
              :style="{ width: resetPatchMaskWidth }"
            ></div>
              <img
                :src="assetUrl('/extra/PoogieR.gif')"
                class="dialog-progress-poogie"
                :style="{ '--progress-poogie-left': resetPatchPoogieLeft }"
                draggable="false"
              />
          </div>
        </template>
        <template v-else-if="store.dialogKind === LINUX_PREFIX_DIALOG">
          <div class="text-xl text-center">
            {{ $t("linux-prefix-install-label", "Portable Prefix Install") }}
          </div>
          <div class="text-center px-2" v-html="store.dialogMessage"></div>
          <div class="relative mt-4 h-[18px] w-[318px]" v-if="store.dialogLoading || store.linuxPrefixInstallCompleted">
            <img :src="assetUrl('/ps4/bar_frame.png')" class="absolute left-0 top-0" />
            <img
              :src="assetUrl('/ps4/bar.jpg')"
              class="absolute left-[8px] top-[5px] h-[6px] w-[302px] object-left"
            />
            <div
              class="absolute top-[5px] right-[8px] h-[6px] bg-black pointer-events-none"
              :style="{ width: linuxPrefixInstallMaskWidth }"
            ></div>
            <img
              :src="assetUrl('/extra/PoogieR.gif')"
              class="dialog-progress-poogie"
              :style="{ '--progress-poogie-left': linuxPrefixInstallPoogieLeft }"
              draggable="false"
            />
          </div>
        </template>
        <template v-else-if="store.dialogKind === BAN_DIALOG">
          <div class="text-xl text-center">
            {{ $t("Access denied", "Access denied") }}
          </div>
          <div class="text-center px-2">{{ store.dialogMessage }}</div>
        </template>
        </div>
        <template
          v-if="store.dialogKind === SERVERS_DIALOG && storeMut.editEndpoint"
        >
          <div class="text-xl">
            <span v-if="store.editEndpointNew">
              {{ $t("server-add-dialog-label") }}
            </span>
            <span v-else>
              {{ $t("server-edit-label") }}
            </span>
          </div>
          <div class="grid grid-cols-7 gap-x-2 items-end gap-y-0.5 px-[100px]">
            <label for="server-name" class="col-span-7">
              {{ $t("server-name-label") }}
            </label>
            <input
              v-model="storeMut.editEndpoint.name"
              type="text"
              class="box-text w-full col-span-5 text-white"
              spellcheck="false"
              ref="srvNameEl"
              :data-controller-node="store.editEndpointNew ? 'server-dialog-name' : null"
              :data-controller-down="store.editEndpointNew ? serverDialogNameDownNode : null"
              :data-controller-focus-mode="store.editEndpointNew ? 'manual' : null"
              :class="
                (store.editEndpointNew || storeMut.editEndpoint.isRemote
                  ? 'col-span-7'
                  : 'col-span-5') +
                (storeMut.editEndpoint.isRemote ? ' disabled' : '')
              "
              :disabled="storeMut.editEndpoint.isRemote"
              @focus="onSrvFocus('name')"
              @blur="onSrvBlur('name')"
              @controller-nav-focus="onSrvControllerNavFocus('name')"
              @keydown="srvTypeSfx"
            />
            <button
              v-if="!store.editEndpointNew && !storeMut.editEndpoint.isRemote"
              class="box-text box-btn col-span-2"
              @mouseenter="playHover()"
              @click.prevent="playSelect(); dialogRemoveEndpoint()"
            >
              X {{ $t("delete-button") }}
            </button>
            <label for="server-host" class="col-span-3">{{
              $t("server-host-label")
            }}</label>
            <label class="text-[14px] leading-tight news-default col-span-2">{{
              $t("server-launcher-port-label")
            }}</label>
            <label class="text-[14px] leading-tight news-default col-span-2">{{
              $t("server-game-port-label")
            }}</label>
            <input
              v-model="storeMut.editEndpoint.url"
              type="text"
              spellcheck="false"
              class="box-text w-full col-span-3 text-white"
              ref="srvUrlEl"
              :data-controller-node="store.editEndpointNew ? 'server-dialog-host' : null"
              :data-controller-up="store.editEndpointNew ? 'server-dialog-name' : null"
              :data-controller-right="store.editEndpointNew ? 'server-dialog-launcher-port' : null"
              :data-controller-down="store.editEndpointNew ? 'server-dialog-cancel' : null"
              :data-controller-focus-mode="store.editEndpointNew ? 'manual' : null"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
              @focus="onSrvFocus('url')"
              @blur="onSrvBlur('url')"
              @controller-nav-focus="onSrvControllerNavFocus('url')"
              @keydown="srvTypeSfx"
            />
            <input
              v-model.number="storeMut.editEndpoint.launcherPort"
              type="text"
              class="box-text col-span-2 text-white"
              spellcheck="false"
              ref="srvLportEl"
              placeholder="8080"
              :data-controller-node="store.editEndpointNew ? 'server-dialog-launcher-port' : null"
              :data-controller-up="store.editEndpointNew ? 'server-dialog-name' : null"
              :data-controller-left="store.editEndpointNew ? 'server-dialog-host' : null"
              :data-controller-right="store.editEndpointNew ? 'server-dialog-game-port' : null"
              :data-controller-down="store.editEndpointNew ? 'server-dialog-add' : null"
              :data-controller-focus-mode="store.editEndpointNew ? 'manual' : null"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
              @focus="onSrvFocus('lport')"
              @blur="onSrvBlur('lport')"
              @controller-nav-focus="onSrvControllerNavFocus('lport')"
              @keydown="srvTypeSfx"
            />
            <input
              v-model.number="storeMut.editEndpoint.gamePort"
              type="text"
              class="box-text col-span-2 text-white"
              spellcheck="false"
              ref="srvGportEl"
              placeholder="53310"
              :data-controller-node="store.editEndpointNew ? 'server-dialog-game-port' : null"
              :data-controller-up="store.editEndpointNew ? 'server-dialog-name' : null"
              :data-controller-left="store.editEndpointNew ? 'server-dialog-launcher-port' : null"
              :data-controller-down="store.editEndpointNew ? 'server-dialog-add' : null"
              :data-controller-focus-mode="store.editEndpointNew ? 'manual' : null"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
              @focus="onSrvFocus('gport')"
              @blur="onSrvBlur('gport')"
              @controller-nav-focus="onSrvControllerNavFocus('gport')"
              @keydown="srvTypeSfx"
            />
          </div>
        </template>
        <div class="grow"></div>
        <div
          v-if="store.dialogKind === EXTERNAL_LINK_DIALOG"
          class="flex m-4 news-default items-center justify-center gap-3"
        >
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogCancelExternalLink()">
            {{ $t("cancel-button") }}
          </button>
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogConfirmExternalLink()">
            {{ $t("ok-button", "OK") }}
          </button>
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogConfirmExternalLinkDontShowAgain()">
            {{ $t("ok-dont-show-again-button", "OK, don't show again") }}
          </button>
        </div>
        <div
          v-else-if="store.dialogKind === VERSION_SWITCH_DIALOG && store.versionSignatureChoices?.length"
          class="version-signature-actions news-default"
        >
          <button
            v-for="choice in store.versionSignatureChoices"
            :key="`ps4-version-signature-${choice.value}`"
            class="box-text box-lg box-btn version-signature-button"
            @mouseenter="playHover()"
            @click.prevent="playSelect(); dialogVersionSignatureSelect(choice.value)"
          >
            {{ choice.label }}
          </button>
          <button
            class="box-text box-lg box-btn version-signature-button"
            @mouseenter="playHover()"
            @click.prevent="playSelect(); dialogVersionSignatureSelect('none')"
          >
            {{ $t("version-signature-unknown-label", "I don't know") }}
          </button>
        </div>
        <div
          v-else-if="store.dialogKind === VERSION_SWITCH_DIALOG"
          class="version-switch-actions news-default"
        >
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogVersionSwitchDontAskAgain()">
                    {{ $t("version-switch-dont-ask-again", "No, and dont ask again") }}
          </button>
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogVersionSwitchStay()">
                    {{ $t("version-switch-stay-label", "Stay on current version") }}
          </button>
          <button class="box-text box-lg box-btn" @mouseenter="playHover()" @click.prevent="playSelect(); dialogVersionSwitchYes()">
            {{ $t("Yes", "Yes") }}
          </button>
        </div>
        <div
          v-else
          class="flex m-4 news-default items-center gap-3"
          :class="((store.dialogKind === RESET_PATCH_DIALOG && store.resetPatchCompleted) || (store.dialogKind === LINUX_PREFIX_DIALOG && store.linuxPrefixInstallCompleted) || store.dialogKind === BAN_DIALOG) ? 'justify-center' : 'justify-between'"
        >
          <form
            method="dialog"
            v-if="store.dialogKind !== BAN_DIALOG && !(store.dialogKind === RESET_PATCH_DIALOG && store.resetPatchCompleted) && !(store.dialogKind === RESET_PATCH_DIALOG && store.dialogLoading) && !(store.dialogKind === LINUX_PREFIX_DIALOG && store.linuxPrefixInstallCompleted) && !(store.dialogKind === LINUX_PREFIX_DIALOG && store.dialogLoading)"
          >
            <button
              class="box-text box-lg box-btn"
              :data-controller-node="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-cancel' : null"
              :data-controller-up="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-host' : null"
              :data-controller-right="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-add' : null"
              @mouseenter="playHover()"
              @click="playSelect();"
            >
              {{ $t("cancel-button") }}
            </button>
          </form>
          <div class="warning">
            {{ store.dialogError }}
          </div>
          <form method="dialog">
            <button
              class="box-text box-lg box-btn"
              :data-controller-node="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-add' : null"
              :data-controller-up="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-game-port' : null"
              :data-controller-left="store.dialogKind === SERVERS_DIALOG && store.editEndpointNew ? 'server-dialog-cancel' : null"
              @mouseenter="playHover()"
              @click="playConfirm();"
              @click.prevent="dialogCallback"
              :disabled="store.dialogLoading"
            >
              <span v-if="store.dialogKind === DELETE_DIALOG">
                {{ $t("delete-button") }}
              </span>
              <span v-else-if="store.dialogKind === PATCHER_DIALOG">
                {{ $t("install-button") }}
              </span>
              <span v-else-if="store.dialogKind === SERVER_SWITCH_DIALOG">
                {{ $t("switch-button") }}
              </span>
              <span v-else-if="store.dialogKind === RESET_PATCH_DIALOG">
                {{ store.resetPatchCompleted ? $t("ok-button", "OK") : $t("reset-button-label") }}
              </span>
              <span v-else-if="store.dialogKind === LINUX_PREFIX_DIALOG">
                {{ store.linuxPrefixInstallCompleted ? $t("ok-button", "OK") : $t("install-button") }}
              </span>
              <span v-else-if="store.dialogKind === BAN_DIALOG">
                {{ $t("ok-button", "OK") }}
              </span>
              <span v-else-if="store.editEndpointNew">
                {{ $t("add-button") }}
              </span>
              <span v-else>
                {{ $t("save-button") }}
              </span>
            </button>
          </form>
        </div>
      </div>
    </div>
  </dialog>
  <dialog
    :open="store.gameLaunching"
    class="absolute inset-0 m-0 h-full max-h-none w-full max-w-none border-0 bg-transparent p-0 z-[1100]"
  >
    <div v-if="store.gameLaunching" class="game-launch-loading">
      <div class="game-launch-loading-tip font-main">
        {{ store.gameLaunchMessage }}
      </div>
      <img
        class="game-launch-loading-status-image"
        :src="assetUrl('/extra/Now-Loading.gif')"
        alt=""
        draggable="false"
      />
    </div>
  </dialog>
  </div>
</template>




































































