<script setup>
const props = defineProps({
  name: String,
  infoKey: {
    type: String,
    default: null,
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
</script>

<template>
  <div class="settings-item-shell">
    <div
      class="settings-item-row flex flex-wrap items-center justify-center gap-3 min-h-[45px] text-center"
      :data-settings-info-key="infoKey"
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
      <div class="flex items-center justify-center">
        <slot></slot>
      </div>
  </div>
    <slot name="extended"></slot>
  </div>
</template>

<style scoped>
.settings-locked-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.65));
}
</style>
