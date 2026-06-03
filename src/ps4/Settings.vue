<script setup>
import { platform } from "@tauri-apps/api/os";
import { computed, getCurrentInstance, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import SettingsList from "../settings/SettingsList.vue";
import { store } from "../store";
import {
  GAME_BRANCH_INFO_ICONS,
  GAME_VERSION_INFO_IMAGES,
  SETTINGS_DESCRIPTION_SPLIT_URL as descriptionSplitUrl,
  SETTINGS_INFO,
  SETTINGS_NAV_ACTIVE_IMAGE as settingsNavActiveImage,
  SETTINGS_NAV_BASE_IMAGE as settingsNavBaseImage,
  SETTINGS_NAV_SOURCE_OFFSETS,
  SETTINGS_NAV_SPRITE_WIDTH,
  SETTINGS_NAV_TILE_HEIGHT,
  SETTINGS_NAV_TILE_WIDTH,
  SETTINGS_SECTIONS as sections,
} from "../settings/settingsInfo";
import { playHover, playSelect } from "../sfx";

const emit = defineEmits(["back"]);

const { proxy } = getCurrentInstance() ?? {};
function t(key, fallback = key) {
  const resolved = String(proxy?.$t?.(key, fallback) ?? fallback);
  return resolved === key && fallback !== key ? String(fallback) : resolved;
}

const activeSection = ref("launcher");
const hoveredInfoKey = ref(null);
const focusedInfoKey = ref(null);
const settingsRootRef = ref(null);
const infoShellRef = ref(null);
const infoContentRef = ref(null);
const missingVersionHeaderImages = ref({});
const infoDrawerCollapsed = ref(false);
const isLinuxHost = ref(false);
const sectionTransitionName = ref("settings-section-forward");
const sectionAnimationClass = ref("");
let sectionAnimationTimer = 0;

function settingsNavButtonStyle(index, sectionId) {
  const start = index * SETTINGS_NAV_TILE_WIDTH;
  const sourceX = SETTINGS_NAV_SOURCE_OFFSETS[index] ?? start;
  return {
    left: `${start}px`,
    width: `${SETTINGS_NAV_TILE_WIDTH}px`,
    backgroundImage: `url('${activeSection.value === sectionId ? settingsNavActiveImage : settingsNavBaseImage}')`,
    backgroundSize: `${SETTINGS_NAV_SPRITE_WIDTH}px ${SETTINGS_NAV_TILE_HEIGHT}px`,
    backgroundPosition: `-${sourceX}px 0`,
    backgroundRepeat: "no-repeat",
  };
}

function previousSectionNode(index) {
  return index > 0 ? `settings-nav-${sections[index - 1].id}` : null;
}

function nextSectionNode(index) {
  return index < sections.length - 1 ? `settings-nav-${sections[index + 1].id}` : null;
}

function onSectionHover(sectionId) {
  if (activeSection.value === sectionId) return;
  playHover();
}

function onSectionClick(sectionId) {
  if (!setActiveSection(sectionId)) return;
  playSelect();
}

function onSectionFocus(sectionId) {
  setActiveSection(sectionId);
}

function setActiveSection(sectionId) {
  if (activeSection.value === sectionId) return false;
  const previousIndex = sections.findIndex((section) => section.id === activeSection.value);
  const nextIndex = sections.findIndex((section) => section.id === sectionId);
  sectionTransitionName.value =
    nextIndex >= previousIndex ? "settings-section-forward" : "settings-section-back";
  activeSection.value = sectionId;
  triggerSectionAnimation(sectionTransitionName.value);
  return true;
}

function triggerSectionAnimation(animationName) {
  if (sectionAnimationTimer) {
    clearTimeout(sectionAnimationTimer);
    sectionAnimationTimer = 0;
  }
  sectionAnimationClass.value = `${animationName}-animate`;
  sectionAnimationTimer = window.setTimeout(() => {
    if (sectionTransitionName.value === animationName) {
      sectionAnimationClass.value = "";
    }
    sectionAnimationTimer = 0;
  }, 220);
}

function extractInfoKey(target) {
  if (!(target instanceof Element)) return null;
  return target.closest("[data-settings-info-key]")?.getAttribute("data-settings-info-key") ?? null;
}

function syncFocusedInfoKey() {
  const activeElement = document.activeElement;
  if (!(activeElement instanceof Element) || !settingsRootRef.value?.contains(activeElement)) {
    focusedInfoKey.value = null;
    return;
  }
  focusedInfoKey.value = extractInfoKey(activeElement);
}

function onInfoTargetMouseOver(event) {
  const key = extractInfoKey(event.target);
  if (
    key === "game-branch" &&
    event.target instanceof Element &&
    event.target.closest("[data-settings-branch-group='true']") &&
    typeof hoveredInfoKey.value === "string" &&
    hoveredInfoKey.value.startsWith("game-branch-")
  ) {
    return;
  }
  if (
    key === "game-version" &&
    event.target instanceof Element &&
    event.target.closest("[data-settings-version-group='true']") &&
    typeof hoveredInfoKey.value === "string" &&
    hoveredInfoKey.value.startsWith("game-version-")
  ) {
    return;
  }
  if (key) {
    hoveredInfoKey.value = key;
  }
}

function onInfoTargetMouseLeave() {
  hoveredInfoKey.value = null;
}

function onInfoTargetFocusIn(event) {
  focusedInfoKey.value = extractInfoKey(event.target);
}

function onInfoTargetFocusOut() {
  nextTick(() => syncFocusedInfoKey());
}

function toggleInfoDrawer() {
  playSelect();
  infoDrawerCollapsed.value = !infoDrawerCollapsed.value;
}

const activeSectionInfoKey = computed(
  () => sections.find((section) => section.id === activeSection.value)?.infoKey ?? "nav-launcher"
);

const currentInfoKey = computed(
  () => hoveredInfoKey.value || focusedInfoKey.value || activeSectionInfoKey.value
);

const currentInfo = computed(() => {
  const meta =
    SETTINGS_INFO[currentInfoKey.value] ??
    SETTINGS_INFO[activeSectionInfoKey.value] ??
    SETTINGS_INFO["nav-launcher"];
  const versionHeader =
    meta.versionImageKey != null
      ? GAME_VERSION_INFO_IMAGES[meta.versionImageKey] ?? null
      : null;
  let body = t(meta.bodyKey, meta.bodyFallback);
  if (currentInfoKey.value === "controller-fix") {
    body = isLinuxHost.value
      ? t(
          "settings-info-controller-fix-linux-body",
          "Applies Wine DLL overrides for xinput1_3, dinput, and d8input."
        )
      : t(
          "settings-info-controller-fix-windows-body",
          "Uses controller DLLs from the game folder for the R-Analog patch."
        );
  }

  return {
    title: meta.titleKey ? t(meta.titleKey, meta.titleFallback) : meta.titleFallback,
    body,
    iconGroup: meta.iconGroup ?? null,
    icons:
      meta.iconGroup === "game-branches"
        ? GAME_BRANCH_INFO_ICONS.map((icon) => ({
            ...icon,
            alt: t(icon.labelKey, icon.labelFallback),
          }))
        : [],
    activeBranch: meta.activeBranch ?? null,
    usesBranchIconHeader: meta.iconGroup === "game-branches" && Boolean(meta.activeBranch),
    usesVersionImageHeader: Boolean(versionHeader),
    versionHeaderKey: versionHeader?.id ?? "",
    versionHeaderSrc: versionHeader?.src ?? "",
    versionHeaderAlt: versionHeader?.display ?? "",
    versionHeaderLabel: versionHeader?.display ?? "",
  };
});

onMounted(async () => {
  try {
    isLinuxHost.value = (await platform()) === "linux";
  } catch (_error) {
    isLinuxHost.value = false;
  }
});

onBeforeUnmount(() => {
  if (sectionAnimationTimer) {
    clearTimeout(sectionAnimationTimer);
  }
});

function onVersionHeaderImageError(versionKey) {
  if (!versionKey || missingVersionHeaderImages.value[versionKey]) {
    return;
  }
  missingVersionHeaderImages.value = {
    ...missingVersionHeaderImages.value,
    [versionKey]: true,
  };
}
</script>

<template>
  <div
    ref="settingsRootRef"
    class="ps4-settings-shell settings-shell grow mt-7 min-h-0"
    :class="{ 'settings-info-collapsed': infoDrawerCollapsed }"
    @mouseover="onInfoTargetMouseOver"
    @mouseleave="onInfoTargetMouseLeave"
    @focusin="onInfoTargetFocusIn"
    @focusout="onInfoTargetFocusOut"
  >
    <div class="settings-top-nav" data-controller-settings-sidebar="true">
      <div class="settings-nav-strip">
        <button
          v-for="(section, index) in sections"
          :key="section.id"
          class="settings-nav-button settings-nav-hitbox"
          :style="settingsNavButtonStyle(index, section.id)"
          :data-settings-tab="section.id"
          :data-controller-node="`settings-nav-${section.id}`"
          :data-settings-active="activeSection === section.id ? 'true' : null"
          :data-settings-info-key="section.infoKey"
          data-controller-size="big"
          :data-controller-left="previousSectionNode(index)"
          :data-controller-right="nextSectionNode(index)"
          data-controller-down="settings-nav-back"
          @mouseenter="onSectionHover(section.id)"
          @focus="onSectionFocus(section.id)"
          @click.prevent="onSectionClick(section.id)"
        >
          <span class="settings-nav-label">{{ $t(section.labelKey) }}</span>
        </button>
      </div>
    </div>

    <div class="settings-body grow min-h-0">
      <div
        class="settings-pane-shell grow min-h-0 !px-4 !py-4 flex flex-col gap-2 overflow-hidden overflow-x-hidden"
        data-controller-settings-pane="true"
      >
        <div
          class="settings-section-panel h-full min-h-0 overflow-x-hidden"
          :class="sectionAnimationClass"
        >
          <SettingsList :active-section="activeSection"></SettingsList>
        </div>
      </div>

      <button
        type="button"
        class="settings-description-divider-button"
        :aria-pressed="infoDrawerCollapsed ? 'true' : 'false'"
        :aria-label="$t('settings-info-toggle-label', 'Toggle settings info')"
        :title="$t('settings-info-toggle-label', 'Toggle settings info')"
        @mouseenter="playHover()"
        @click.prevent="toggleInfoDrawer"
      >
        <img
          :src="descriptionSplitUrl"
          class="settings-description-divider"
          alt=""
          draggable="false"
          aria-hidden="true"
        />
      </button>

      <aside class="settings-info-shell" aria-live="polite">
        <div ref="infoShellRef" class="settings-info-viewport">
          <div
            ref="infoContentRef"
            class="settings-info-content"
          >
            <div
              v-if="currentInfo.usesBranchIconHeader"
              class="settings-info-branch-header"
              aria-hidden="true"
            >
              <img
                v-for="icon in currentInfo.icons"
                v-show="currentInfo.activeBranch === icon.id"
                :key="`settings-info-branch-header-${icon.id}`"
                :src="icon.src"
                :alt="icon.alt"
                class="settings-info-branch-header-image"
                draggable="false"
              />
            </div>
            <div
              v-else-if="currentInfo.usesVersionImageHeader"
              class="settings-info-version-header"
              aria-hidden="true"
            >
              <img
                v-if="currentInfo.versionHeaderSrc && !missingVersionHeaderImages[currentInfo.versionHeaderKey]"
                :src="currentInfo.versionHeaderSrc"
                :alt="currentInfo.versionHeaderAlt"
                class="settings-info-version-header-image"
                draggable="false"
                @error="onVersionHeaderImageError(currentInfo.versionHeaderKey)"
              />
              <div v-else class="settings-info-version-header-placeholder font-main">
                {{ currentInfo.versionHeaderLabel }}
              </div>
            </div>
            <h2 v-else class="settings-info-title font-main">{{ currentInfo.title }}</h2>
            <div
              v-if="currentInfo.iconGroup === 'game-branches' && currentInfo.icons.length && !currentInfo.usesBranchIconHeader"
              class="settings-info-branch-strip"
              aria-hidden="true"
            >
              <img
                v-for="icon in currentInfo.icons"
                :key="`settings-info-branch-strip-${icon.id}`"
                :src="icon.src"
                :alt="icon.alt"
                class="settings-info-branch-strip-image"
                draggable="false"
              />
            </div>
            <div
              v-else-if="currentInfo.icons.length && !currentInfo.usesBranchIconHeader"
              class="settings-info-icon-strip"
              aria-hidden="true"
            >
              <div
                v-for="icon in currentInfo.icons"
                :key="`settings-info-icon-${icon.id}`"
                class="settings-info-icon-frame"
                :class="{ 'settings-info-icon-frame-active': currentInfo.activeBranch === icon.id }"
              >
                <img
                  :src="icon.src"
                  :alt="icon.alt"
                  class="settings-info-icon-image"
                  draggable="false"
                />
              </div>
            </div>
            <p class="settings-info-body font-main">{{ currentInfo.body }}</p>
          </div>
        </div>
      </aside>
    </div>

    <button
      class="ps4-settings-button ps4-settings-button-back settings-local-back"
      data-controller-node="settings-nav-back"
      data-controller-size="big"
      data-controller-up="settings-nav-launcher"
      @mouseenter="playHover()"
      @click="emit('back')"
      :title="$t('go-back-button')"
    >
      <span class="ps4-settings-button-back-label font-main">{{ $t("go-back-button") }}</span>
    </button>
    <div class="ps4-settings-local-tag font-main">{{ store.launcherTag }}</div>
  </div>
</template>
