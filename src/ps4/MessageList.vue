<script setup>
import { open } from "@tauri-apps/api/shell";
import { formatDate } from "../common";
import { playSelect } from "../sfx";
import { assetUrl, confirmExternalLinkOpen } from "../store";

const props = defineProps({
  title: String,
  important: Boolean,
  messages: Array,
  nodePrefix: String,
});

async function onMessageClick(link) {
  if (!link) return;
  playSelect();
  const canOpen = await confirmExternalLinkOpen(link);
  if (!canOpen) return;
  open(link).catch((e) => console.error("open failed:", e));
}

function scrollMessageIntoView(event, index) {
  const row = event?.currentTarget;
  row?.scrollIntoView?.({ block: "nearest" });

  const container = row?.closest?.("[data-controller-node='message_box']");
  if (!(container instanceof HTMLElement)) return;

  if (props.nodePrefix === "message-entry-ann" && index === 0) {
    container.scrollTop = 0;
    requestAnimationFrame(() => {
      if (container.isConnected) {
        container.scrollTop = 0;
      }
    });
  }
}
</script>

<template>
  <section class="ps4-message-section">
    <div class="ps4-message-title-shell">
      <img
        :src="assetUrl(important ? '/ps4/msg-line-important.png' : '/ps4/msg-line-base.png')"
        class="ps4-message-divider"
        draggable="false"
      />
      <div class="ps4-message-title" :class="{ 'ps4-message-title-important': important }">
        {{ title }}
      </div>
    </div>

    <div v-if="!messages?.length" class="ps4-message-empty">
      {{ $t("none-label", "None") }}
    </div>

    <template v-for="(message, index) in messages" :key="`${message.date}-${index}`">
      <div
        class="ps4-message-row"
        :class="{ 'ps4-message-row-important': important }"
        :data-controller-node="nodePrefix ? `${nodePrefix}-${index}` : null"
        data-controller-clickable="true"
        data-controller-size="big"
        tabindex="0"
        @focus="scrollMessageIntoView($event, index)"
        @click="onMessageClick(message.link)"
      >
        <div class="ps4-message-date">
          {{ formatDate(message.date) }}
        </div>
        <div class="ps4-message-link">
          {{ message.message }}
        </div>
        <div class="ps4-message-badge">
          <img
            v-if="important"
            :src="assetUrl('/ps4/New.png')"
            class="ps4-message-new"
            draggable="false"
          />
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.ps4-message-row.controller-nav-focused,
.ps4-message-row:hover,
.ps4-message-row:focus-visible {
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}
</style>
