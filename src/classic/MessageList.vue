<script setup>
import { computed } from "@vue/reactivity";
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

const listClass = computed(() =>
  props.important ? "news-important" : "news-default"
);
const dividerSrc = computed(() =>
  props.important ? "/classic/msg-line-important.png" : "/classic/msg-line-base.png"
);

async function onMessageClick(link) {
  if (!link) return;
  playSelect();
  const canOpen = await confirmExternalLinkOpen(link);
  if (!canOpen) return;
  open(link).catch((error) => console.error("open failed:", error));
}

function onNewBadgeError(event) {
  const fallback = "/classic/new.gif";
  if (event?.target && !event.target.src.endsWith(fallback)) {
    event.target.src = fallback;
  }
}

function scrollMessageIntoView(event, index) {
  const row = event?.currentTarget;
  row?.scrollIntoView?.({ block: "nearest" });

  const container = row?.closest?.("[data-controller-node='message_box']");
  if (!(container instanceof HTMLElement)) return;

  if (props.nodePrefix === "message-entry-ann" && index === 0) {
    container.scrollTop = 0;
  }
}
</script>

<template>
  <div class="leading-4 font-main">
    <div
      v-if="messages.length"
      class="col-span-3 w-full text-xl mb-1 sidescrollfix"
      :class="listClass"
    >
      <img
        :src="assetUrl(dividerSrc)"
        draggable="false"
        class="message-divider"
      />
      <div class="messages-header font-old relative bottom-[25px] left-[18px]">
        {{ title }}
      </div>
    </div>

    <template v-for="(message, index) in messages" :key="`${message.date}-${index}`">
      <div
        class="message-row-grid relative left-[-11px] py-[5px]"
        :class="[listClass, { 'pt-0': index === 0, 'mt-[-0.1em]': index === 0 }]"
        :data-controller-node="nodePrefix ? `${nodePrefix}-${index}` : null"
        data-controller-clickable="true"
        data-controller-size="big"
        tabindex="0"
        @focus="scrollMessageIntoView($event, index)"
        @click="onMessageClick(message.link)"
      >
        <div class="ml-[18px] message-row">
          {{ formatDate(message.date) }}
        </div>
        <div class="cursor-pointer news-button message-row">
          {{ message.message }}
        </div>
        <div class="flex items-start new-gif">
          <img
            v-if="important"
            :src="assetUrl('/classic/new.gif')"
            @error="onNewBadgeError"
            class="mt-1.5"
            draggable="false"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.message-row-grid.controller-nav-focused,
.message-row-grid:hover,
.message-row-grid:focus-visible {
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}
</style>
