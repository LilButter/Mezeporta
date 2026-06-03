<script setup>
import { playSelect, forceSfxUnlock } from "../sfx";
import { computed, nextTick, ref, watch } from "vue";
import { forceRepaint } from "../common";

const props = defineProps({
  name: String,
  modelValue: Boolean,
  controllerNode: {
    type: String,
    default: null,
  },
  controllerUp: {
    type: String,
    default: null,
  },
  controllerDown: {
    type: String,
    default: null,
  },
  controllerLeft: {
    type: String,
    default: null,
  },
  controllerRight: {
    type: String,
    default: null,
  },
  infoKey: {
    type: String,
    default: null,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  locked: {
    type: Boolean,
    default: false,
  },
  lockTooltip: {
    type: String,
    default: "Log-out to adjust",
  },
});

const emit = defineEmits(["update:modelValue"]);
const inputEl = ref(null);

const checkedValue = computed({
  get: () => Boolean(props.modelValue),
  set: (value) => emit("update:modelValue", value),
});

function toggleChecked() {
  if (props.disabled) return;
  checkedValue.value = !checkedValue.value;
  forceSfxUnlock();
  playSelect();
  if (inputEl.value) inputEl.value.focus();
}

watch(
  () => props.modelValue,
  async () => {
    await nextTick();
    forceRepaint(inputEl.value);
  }
);
</script>

<template>
  <div
    class="settings-checkbox-row flex flex-wrap items-center justify-center gap-3 min-h-[45px] cursor-pointer text-center"
    :class="{ 'settings-checkbox-locked-control': disabled }"
    :data-settings-info-key="infoKey"
    :data-controller-clickable="disabled ? null : 'true'"
    data-controller-size="big"
    :data-controller-toggle-state="checkedValue ? 'on' : 'off'"
    :data-controller-node="controllerNode"
    :data-controller-up="controllerUp"
    :data-controller-down="controllerDown"
    :data-controller-left="controllerLeft"
    :data-controller-right="controllerRight"
    :tabindex="disabled ? -1 : 0"
    @click="toggleChecked"
    @keydown.enter.prevent="toggleChecked"
    @keydown.space.prevent="toggleChecked"
  >
    <img
      v-if="locked"
      src="/extra/GreyLock.png"
      class="settings-locked-icon"
      :title="lockTooltip"
      alt=""
      draggable="false"
    />
    <h2 class="leading-tight">
      {{ name }}
    </h2>
    <label
      class="relative inline-flex items-center"
      :class="disabled ? 'cursor-default' : 'cursor-pointer'"
      @click.stop.prevent="toggleChecked"
    >
      <input
        type="checkbox"
        class="sr-only peer"
        ref="inputEl"
        v-model="checkedValue"
        :disabled="disabled"
        @click.stop
        @change.stop
      />
      <div
        class="settings-checkbox-track w-12 h-7 rounded-full bg-black/50 border border-white/30 transition-colors peer-focus:ring-2"
      ></div>
      <div
        class="settings-checkbox-thumb absolute left-[3px] top-[3px] w-5 h-5 rounded-full bg-[#f5f5f5] shadow transition-transform transition-colors peer-checked:translate-x-5"
        :class="{ 'settings-checkbox-thumb-checked': checkedValue }"
      ></div>
    </label>
  </div>
  <slot name="extended"></slot>
</template>

<style scoped>
.settings-checkbox-locked-control {
  cursor: default;
  opacity: 1;
}

.settings-checkbox-locked-control .settings-checkbox-track,
.settings-checkbox-locked-control .settings-checkbox-thumb {
  opacity: 0.58;
}

.settings-checkbox-locked-control h2,
.settings-checkbox-locked-control .settings-locked-icon {
  opacity: 1;
}

.settings-checkbox-locked-control h2 {
  color: #3f2a15 !important;
}

.settings-checkbox-locked-control .settings-locked-icon {
  opacity: 1 !important;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.65)) !important;
}

.settings-locked-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.65));
}
</style>
