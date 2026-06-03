<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

import { forceRepaint } from "../common";
import { playSelect } from "../sfx";

const props = defineProps({
  modelValue: {
    type: [String, Number],
    default: "",
  },
  options: {
    type: Array,
    default: () => [],
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  widthClass: {
    type: String,
    default: "settings-select-fixed",
  },
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
});

const emit = defineEmits(["update:modelValue", "change"]);

const pickerOpen = ref(false);
const wrapRef = ref(null);
const toggleRef = ref(null);
const dropdownRef = ref(null);

const currentOption = computed(
  () =>
    props.options.find(
      (option) => String(option?.value ?? "") === String(props.modelValue ?? "")
    ) ?? null
);

const currentLabel = computed(
  () => currentOption.value?.label ?? String(props.modelValue ?? "")
);
const currentInfoKey = computed(
  () => currentOption.value?.infoKey ?? props.infoKey
);

function focusToggle() {
  if (typeof toggleRef.value?.focus === "function") {
    toggleRef.value.focus({ preventScroll: true });
  }
}

async function focusInitialOption() {
  await nextTick();
  const selectedOption =
    dropdownRef.value?.querySelector?.("[data-settings-option-selected='true']:not([disabled])") ??
    null;
  const firstEnabledOption =
    dropdownRef.value?.querySelector?.("button:not([disabled])") ?? null;
  const target = selectedOption || firstEnabledOption;
  if (typeof target?.focus === "function") {
    target.focus({ preventScroll: true });
    target.scrollIntoView?.({ block: "nearest" });
  }
}

function scrollOptionIntoView(event) {
  event?.currentTarget?.scrollIntoView?.({ block: "nearest" });
}

async function openDropdown() {
  if (props.disabled || pickerOpen.value) return;
  playSelect();
  pickerOpen.value = true;
  await focusInitialOption();
}

function closeDropdown({ restoreFocus = true } = {}) {
  const hadFocusInside =
    dropdownRef.value?.contains?.(document.activeElement) ?? false;
  pickerOpen.value = false;
  if ((restoreFocus || hadFocusInside) && !props.disabled) {
    nextTick(() => focusToggle());
  }
}

function toggleDropdown() {
  if (props.disabled) return;
  if (pickerOpen.value) {
    playSelect();
    closeDropdown();
    return;
  }
  void openDropdown();
}

function chooseOption(option) {
  if (!option || option.disabled) return;
  playSelect();
  emit("update:modelValue", option.value);
  emit("change", option.value);
  closeDropdown();
}

function onOutsidePointer(event) {
  if (!pickerOpen.value || !wrapRef.value) return;
  const target = event.target;
  const path = typeof event.composedPath === "function" ? event.composedPath() : [];
  const insideToggle =
    toggleRef.value &&
    (toggleRef.value.contains(target) || path.includes(toggleRef.value));
  const insideList =
    dropdownRef.value &&
    (dropdownRef.value.contains(target) || path.includes(dropdownRef.value));
  if (insideToggle || insideList) return;
  closeDropdown({ restoreFocus: false });
}

function onEscape(event) {
  if (event.key !== "Escape" || !pickerOpen.value) return;
  closeDropdown();
}

onMounted(() => {
  document.addEventListener("pointerdown", onOutsidePointer, true);
  document.addEventListener("keydown", onEscape);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onOutsidePointer, true);
  document.removeEventListener("keydown", onEscape);
});

watch(
  () => props.disabled,
  (disabled) => {
    if (disabled && pickerOpen.value) {
      closeDropdown({ restoreFocus: false });
    }
  }
);
</script>

<template>
  <div
    ref="wrapRef"
    class="settings-dropdown-wrap"
    :data-settings-info-key="currentInfoKey"
    :data-settings-picker-open="pickerOpen ? 'true' : null"
  >
    <button
      ref="toggleRef"
      type="button"
      class="settings-dropdown-toggle settings-select text-[18px]"
      :class="[widthClass, { 'settings-control-locked opacity-50 cursor-default': disabled }]"
      :disabled="disabled"
      :data-settings-info-key="currentInfoKey"
      :data-controller-settings-toggle="disabled ? null : 'true'"
      :data-controller-clickable="disabled ? null : 'true'"
      data-controller-size="big"
      :data-controller-node="controllerNode"
      :data-controller-up="controllerUp"
      :data-controller-down="controllerDown"
      :data-controller-left="controllerLeft"
      :data-controller-right="controllerRight"
      @click="toggleDropdown"
      @mouseenter="forceRepaint($event.currentTarget)"
    >
      <span class="settings-dropdown-label">{{ currentLabel }}</span>
      <span
        class="settings-dropdown-caret"
        :class="{ 'settings-dropdown-caret-open': pickerOpen }"
        aria-hidden="true"
      ></span>
    </button>

    <div
      v-show="pickerOpen"
      ref="dropdownRef"
      class="settings-dropdown-menu scrollbar"
      :class="widthClass"
      :data-settings-info-key="currentInfoKey"
      data-controller-dropdown-scope="true"
    >
      <button
        v-for="option in options"
        :key="String(option?.value ?? option?.label ?? '')"
        type="button"
        class="settings-dropdown-option"
        :disabled="option?.disabled"
        :data-settings-info-key="option?.infoKey ?? currentInfoKey"
        :data-settings-option-selected="
          String(option?.value ?? '') === String(modelValue ?? '') ? 'true' : null
        "
        @click="chooseOption(option)"
        @focus="scrollOptionIntoView"
        @mouseenter="forceRepaint($event.currentTarget)"
      >
        {{ option?.label ?? option?.value }}
      </button>
    </div>
  </div>
</template>
