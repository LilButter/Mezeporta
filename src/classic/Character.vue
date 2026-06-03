<script setup>
import { useFluent } from "fluent-vue";
import { onMounted, ref, watch, computed } from "vue";
import { formatDate, getCid } from "../common";
import { doSelectCharacter, doCreateCharacter, runCharacterLaunchAction, store, assetUrl } from "../store";
import { playHover, playStartAndWait } from "../sfx";

const { $t } = useFluent();

const HR_MAP = { 1: 1, 30: 2, 50: 3, 99: 4, 299: 5, 998: 6, 999: 7 };
const displayHr = (raw) => HR_MAP[raw] ?? raw;

function getWeaponLabel(weapon) {
  switch (weapon) {
    case 0:  return $t("greatsword-label");
    case 1:  return $t("heavy-bowgun-label");
    case 2:  return $t("hammer-label");
    case 3:  return $t("lance-label");
    case 4:  return $t("sword-and-shield-label");
    case 5:  return $t("light-bowgun-label");
    case 6:  return $t("dual-swords-label");
    case 7:  return $t("longsword-label");
    case 8:  return $t("hunting-horn-label");
    case 9:  return $t("gunlance-label");
    case 10: return $t("bow-label");
    case 11: return $t("tonfa-label");
    case 12: return $t("switch-axe-label");
    case 13: return $t("magnet-spike-label");
    default: return "";
  }
}

const props = defineProps({
  character: Object,
  selectable: Boolean,
  showGearButton: {
    type: Boolean,
    default: false,
  },
  showBoxButton: {
    type: Boolean,
    default: false,
  },
  loadCycle: {
    type: Number,
    default: 0,
  },
});
const emit = defineEmits(["portrait-ready", "gear-click", "box-click"]);

const isPlaceholder = computed(() => {
  const c = props.character;
  return !c || c.id === null || c.placeholder;
});

const unitSrc = ref(assetUrl('/units/unitbg.png'));
let activePortraitRequest = 0;

function notifyPortraitReady() {
  emit("portrait-ready", store.unitCardLoadCycle);
}

function characterKey(character) {
  if (!character) return "";
  return `${character.id ?? "null"}::${String(character.name ?? "").trim()}`;

}
function resolvedLauncherOrigin() {
  if (!store.currentEndpoint || !store.currentEndpoint.url) return '';
  if (store.currentEndpoint.url === 'OFFLINEMODE') return '';

  const rawUrl = store.currentEndpoint.url.includes('://')
    ? store.currentEndpoint.url
    : `http://${store.currentEndpoint.url}`;

  try {
    const parsed = new URL(rawUrl);
    if (store.currentEndpoint.launcherPort) {
      parsed.port = String(store.currentEndpoint.launcherPort);
    }
    return parsed.origin;
  } catch (_error) {
    return '';
  }
}

function portraitUrls() {
  const cacheBust = props.character?.lastLogin || Date.now();
  const characterId = Number(props.character?.id);
  const base = resolvedLauncherOrigin();
  if (!base || !Number.isFinite(characterId) || characterId <= 0) {
    return [];
  }

  const numericId = String(Math.trunc(characterId));
  const ids = [
    getCid(Math.trunc(characterId)),
    numericId,
  ]
    .map((value) => String(value ?? "").trim())
    .filter(Boolean);

  return [...new Set(ids)].map(
    (id) => `${base}/ClientImages/launcher/units/${encodeURIComponent(id)}.png?v=${cacheBust}`
  );
}

function loadPortrait() {
  const requestId = ++activePortraitRequest;
  const c = props.character;
  const expectedKey = characterKey(c);

  if (!c || c.id === null || c.placeholder) {
    unitSrc.value = assetUrl('/units/unitbg.png');
    notifyPortraitReady();
    return;
  }

  unitSrc.value =
    typeof c.weapon === 'number'
      ? assetUrl(`/units/${c.weapon}.png`)
      : assetUrl('/units/unitbg.png');

  const urls = portraitUrls();
  if (!urls.length) {
    notifyPortraitReady();
    return;
  }

  const tryLoadAt = (index) => {
    if (index >= urls.length) {
      notifyPortraitReady();
      return;
    }

    const img = new Image();
    img.src = urls[index];
    img.onload = () => {
      if (requestId !== activePortraitRequest) return;
      if (characterKey(props.character) !== expectedKey) return;
      unitSrc.value = img.src;
      notifyPortraitReady();
    };
    img.onerror = () => {
      if (requestId !== activePortraitRequest) return;
      if (characterKey(props.character) !== expectedKey) return;
      tryLoadAt(index + 1);
    };
  };

  tryLoadAt(0);
}

onMounted(loadPortrait);
watch(
  () => [props.character?.id, props.character?.name, props.character?.lastLogin],
  loadPortrait
);
watch(
  () => store.unitCardLoadCycle,
  loadPortrait
);

async function onCardClick() {
  if (!props.selectable) return;
  await runCharacterLaunchAction(async () => {
    await playStartAndWait();
    const c = props.character;
    if (!c || c.id === null || c.placeholder) {
      await doCreateCharacter();
    } else {
      await doSelectCharacter(c.id);
    }
  }, { showLaunchOverlay: true });
}

function onCardHover() {
  if (!props.selectable) return;
  playHover();
}
</script>

<template>
  <div
    class="text-black my-2 h-[143px] w-[520px] p-2 relative"
    :style="{ backgroundImage: `url(${unitSrc})` }"
    :data-controller-clickable="selectable ? 'true' : null"
    :data-controller-size="selectable ? 'big' : null"
    :data-controller-priority="selectable ? '10' : null"
    :tabindex="selectable ? 0 : null"
  >
    <div
      v-if="!isPlaceholder && (showGearButton || showBoxButton)"
      class="classic-card-panel-buttons"
    >
      <button
        v-if="showGearButton"
        type="button"
        class="classic-card-panel-button"
        title="Character"
        @mouseenter="playHover()"
        @click.stop="emit('gear-click')"
      >
        <img
          :src="assetUrl('/extra/Character.png')"
          class="classic-card-panel-icon"
          draggable="false"
        />
      </button>
      <button
        v-if="showBoxButton"
        type="button"
        class="classic-card-panel-button"
        title="Storage"
        @mouseenter="playHover()"
        @click.stop="emit('box-click')"
      >
        <img
          :src="assetUrl('/extra/Storage.png')"
          class="classic-card-panel-icon"
          draggable="false"
        />
      </button>
    </div>
    <div
      class="w-full h-full flex flex-col items-center"
      :class="{ 'cursor-pointer': selectable }"
      @mouseenter="onCardHover"
      @click="onCardClick()"
    >
      <div class="text-3xl mt-2 font-bold">
        {{ isPlaceholder ? $t('create-character-label') : character.name }}
      </div>

      <div class="grow py-2 px-4 w-full h-full flex">
        <div class="flex-1 flex gap-2">
          <img
            v-if="!isPlaceholder"
            :src="assetUrl(`/weapons/${character.weapon}.png`)"
            class="h-[48px] m-2"
            draggable="false"
          />
          <div
            class="weapon-col grow flex flex-col items-center leading-4 justify-center mr-7"
          >
            <div class="font-bold">
              {{ isPlaceholder ? '' : $t('weapon-label') }}
            </div>
            <div
              class="font-bold whitespace-nowrap"
              :class="
                isPlaceholder
                  ? 'text-xl'
                  : getWeaponLabel(character.weapon).length > 12
                  ? 'text-lg'
                  : 'text-xl'
              "
            >
              {{ isPlaceholder ? '' : getWeaponLabel(character.weapon) }}
            </div>
          </div>
        </div>

        <div class="flex-1 flex flex-col text-lg leading-[1] mt-1">
          <div class="flex gap-4" v-if="!isPlaceholder">
            <span>{{ $t("hr-short-label", "HR") }}{{ displayHr(character.hr) }}</span>
            <span v-if="character.gr">{{ $t("gr-short-label", "GR") }}{{ character.gr }}</span>
            <span class="font-mono">
              <span v-if="character.isFemale">&#9792;</span>
              <span v-else>&#9794;</span>
            </span>
          </div>
          <div>
            {{ $t("id-short-label", "ID") }}:
            {{ isPlaceholder ? 'To be Determined' : getCid(character.id) }}
          </div>
          <div>
            {{
              isPlaceholder
                ? '--'
                : `${$t('last-online-label')}: ${formatDate(character.lastLogin)}`
            }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.classic-card-panel-buttons {
  position: absolute;
  top: 10px;
  right: 18px;
  z-index: 8;
  display: flex;
  align-items: center;
  gap: 2px;
}

.classic-card-panel-button {
  width: 38px;
  height: 38px;
  padding: 0;
  border: 0;
  background: transparent;
}

.classic-card-panel-icon {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.weapon-col {
  min-width: 140px;
}
</style>
