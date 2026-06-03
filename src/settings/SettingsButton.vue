<script setup lang="ts">
import SettingsItem from "./SettingsItem.vue";
import { playHover, playSelect } from "../sfx";
import { dialogOpenResetPatch } from "../store";

const props = defineProps<{
  gameFolder: string;
  label?: string;
  buttonText?: string;
  infoKey?: string;
  disabled?: boolean;
  locked?: boolean;
  lockTooltip?: string;
}>();

function handleClick() {
  if (props.disabled) return;
  playSelect();
  dialogOpenResetPatch(props.gameFolder);
}
</script>

<template>
  <SettingsItem
    :name="label"
    :info-key="infoKey"
    :locked="locked"
    :lock-tooltip="lockTooltip"
  >
    <button
      class="px-3 py-1 rounded border border-red-400 hover:bg-red-500 hover:text-white transition"
      :class="{ 'settings-control-locked': disabled }"
      :data-settings-info-key="infoKey"
      :disabled="disabled"
      @click="handleClick"
      @mouseenter="disabled ? null : playHover()"
    >
      {{ buttonText }}
    </button>
  </SettingsItem>
</template>

<style scoped>
.settings-control-locked {
  cursor: default !important;
  opacity: 0.52;
  pointer-events: none;
}
</style>
