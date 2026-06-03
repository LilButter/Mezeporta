<script setup>
import { computed, ref, onMounted, onBeforeUnmount, watch, nextTick } from "vue";

import { openPicker, forceRepaint } from "../common";
import GearIcon from "../components/GearIcon.vue";
import {
  store,
  storeMut,
  setCurrentEndpoint,
  doLogin,
  doRegister,
  dialogEditEndpoint,
  dialogAddEndpoint,
  classicButtonUrl,
  classicAddServerButtonUrl,
  assetUrl,
} from "../store";

import { playHover, playSelect, playConfirm, bindSfx } from "../sfx";

const serverPicker = ref(false);

const usernameEl  = ref(null);
const passwordEl  = ref(null);
const loginBtn    = ref(null);
const registerBtn = ref(null);
const serverBtn   = ref(null);
const rememberEl  = ref(null);
const serverWrap  = ref(null);
const dropdownRef = ref(null);

const remoteEndpointCount = computed(() => store.remoteEndpoints?.length ?? 0);
const localEndpointCount = computed(() => store.endpoints?.length ?? 0);
const isOfflineMode = computed(() => store.currentEndpoint?.url === "OFFLINEMODE");

// Filtered endpoints that exclude the currently selected server from dropdown options
const filteredRemoteEndpoints = computed(() => 
  (store.remoteEndpoints ?? []).filter(
    endpoint => endpoint.url !== store.currentEndpoint?.url
  )
);
const filteredLocalEndpoints = computed(() => 
  (store.endpoints ?? []).filter(
    endpoint => endpoint.url !== store.currentEndpoint?.url
  )
);

function serverMainNode(remote, index) {
  return `server-${remote ? "remote" : "local"}-main-${index}`;
}

function serverGearNode(remote, index) {
  return `server-${remote ? "remote" : "local"}-gear-${index}`;
}

function endpointHasGear(endpoint) {
  const url = String(endpoint?.url ?? "").trim().toUpperCase();
  const name = String(endpoint?.name ?? "").trim().toUpperCase();
  return Boolean(url) && url !== "OFFLINEMODE" && name !== "OFFLINE-MODE";
}

const remoteGearIndexes = computed(() =>
  (store.remoteEndpoints ?? [])
    .map((endpoint, index) => (endpointHasGear(endpoint) ? index : null))
    .filter((index) => index !== null)
);

const localGearIndexes = computed(() =>
  (store.endpoints ?? [])
    .map((endpoint, index) => (endpointHasGear(endpoint) ? index : null))
    .filter((index) => index !== null)
);

function firstRemoteGearNode() {
  const firstIndex = remoteGearIndexes.value[0];
  return Number.isInteger(firstIndex) ? serverGearNode(true, firstIndex) : null;
}

function firstLocalGearNode() {
  const firstIndex = localGearIndexes.value[0];
  return Number.isInteger(firstIndex) ? serverGearNode(false, firstIndex) : null;
}

function previousRemoteGearNode(index) {
  const currentIndex = remoteGearIndexes.value.indexOf(index);
  if (currentIndex <= 0) return null;
  return serverGearNode(true, remoteGearIndexes.value[currentIndex - 1]);
}

function previousLocalGearNode(index) {
  const currentIndex = localGearIndexes.value.indexOf(index);
  if (currentIndex <= 0) return null;
  return serverGearNode(false, localGearIndexes.value[currentIndex - 1]);
}

function nextRemoteGearNode(index) {
  const currentIndex = remoteGearIndexes.value.indexOf(index);
  if (currentIndex === -1 || currentIndex + 1 >= remoteGearIndexes.value.length) {
    return null;
  }
  return serverGearNode(true, remoteGearIndexes.value[currentIndex + 1]);
}

function nextLocalGearNode(index) {
  const currentIndex = localGearIndexes.value.indexOf(index);
  if (currentIndex === -1 || currentIndex + 1 >= localGearIndexes.value.length) {
    return null;
  }
  return serverGearNode(false, localGearIndexes.value[currentIndex + 1]);
}

function previousServerMainNode(remote, index) {
  if (remote) return index > 0 ? serverMainNode(true, index - 1) : null;
  if (index > 0) return serverMainNode(false, index - 1);
  return remoteEndpointCount.value > 0
    ? serverMainNode(true, remoteEndpointCount.value - 1)
    : null;
}

function nextServerMainNode(remote, index) {
  if (remote) {
    if (index + 1 < remoteEndpointCount.value) return serverMainNode(true, index + 1);
    if (localEndpointCount.value > 0) return serverMainNode(false, 0);
    return "server-add";
  }
  if (index + 1 < localEndpointCount.value) return serverMainNode(false, index + 1);
  return "server-add";
}

function previousServerGearNode(remote, index) {
  if (remote) return previousRemoteGearNode(index);
  const previousLocalNode = previousLocalGearNode(index);
  if (previousLocalNode) return previousLocalNode;
  const remoteNodes = remoteGearIndexes.value;
  return remoteNodes.length > 0
    ? serverGearNode(true, remoteNodes[remoteNodes.length - 1])
    : null;
}

function nextServerGearNode(remote, index) {
  if (remote) {
    const nextRemoteNode = nextRemoteGearNode(index);
    if (nextRemoteNode) return nextRemoteNode;
    return firstLocalGearNode();
  }
  const nextLocalNode = nextLocalGearNode(index);
  if (nextLocalNode) return nextLocalNode;
  return null;
}

function addServerUpNode() {
  if (localEndpointCount.value > 0) return serverMainNode(false, localEndpointCount.value - 1);
  if (remoteEndpointCount.value > 0) return serverMainNode(true, remoteEndpointCount.value - 1);
  return null;
}

let lastKeyTs = 0;
const MOD_KEYS = new Set(["Shift", "Control", "Alt", "Meta", "CapsLock"]);

function typeSfx(e) {
  // ignore pure modifier keys or auto-repeat of any key
  if (MOD_KEYS.has(e.key) || e.repeat) return;

  const now = performance.now();
  if (now - lastKeyTs < 45) return;
  lastKeyTs = now;
  playHover();
  forceRepaint(e.target);
}

// focus sound: only once when changing focus
const lastFocusedEl = ref(null);
function onInputFocus(e) {
  if (e.target?.dataset?.controllerSuppressFocusSfx === "true") {
    lastFocusedEl.value = e.target;
    forceRepaint(e.target);
    return;
  }
  if (lastFocusedEl.value !== e.target) {
    playSelect();
    lastFocusedEl.value = e.target;
    forceRepaint(e.target);
  }
}
function onInputBlur(e) {
  if (lastFocusedEl.value === e.target) lastFocusedEl.value = null;
}

function onLoginClick()    { playConfirm(); doLogin();    }
function onRegisterClick() { playConfirm(); doRegister(); }

function onRememberClick() {
  playSelect();
  if (!store.authLoading) storeMut.rememberMe = !storeMut.rememberMe;
}

function onPrimaryClick() {
  if (isOfflineMode.value) {
    addEndpoint();
    return;
  }
  onLoginClick();
}

async function focusFirstServerOption() {
  await nextTick();
  const firstOption = dropdownRef.value?.querySelector?.("button");
  if (typeof firstOption?.focus === "function") {
    firstOption.focus({ preventScroll: true });
  }
}

function restoreServerButtonFocus() {
  const element = serverBtn.value;
  if (!(element instanceof HTMLElement) || typeof element.focus !== "function") return;
  element.focus({ preventScroll: true });
  element.dispatchEvent(
    new CustomEvent("controller-nav-sync-focus", {
      bubbles: true,
      detail: { element },
    })
  );
}

async function openPickerRef() {
  playSelect(); // open/close sound
  openPicker(serverPicker);
  if (serverPicker.value) {
    await focusFirstServerOption();
  } else {
    await nextTick();
    restoreServerButtonFocus();
  }
}

function closePicker() {
  const hadFocusInside = dropdownRef.value?.contains?.(document.activeElement) ?? false;
  serverPicker.value = false;
  if (hadFocusInside) {
    nextTick(() => restoreServerButtonFocus());
  }
}

// choose endpoint
async function chooseEndpoint(endpoint) {
  playSelect();
  await setCurrentEndpoint(endpoint);
  serverPicker.value = false;
  nextTick(() => restoreServerButtonFocus());
}

// edit/add endpoints
function editEndpoint(i, remote) { playSelect(); dialogEditEndpoint(i, remote); }
function addEndpoint()           { playSelect(); dialogAddEndpoint(); }

let unbinds = [];
let interactiveUnbinds = [];

function cleanupInteractiveBinds() {
  interactiveUnbinds.forEach((unbind) => unbind && unbind());
  interactiveUnbinds = [];
}

function bindInteractiveSfx() {
  cleanupInteractiveBinds();
  if (loginBtn.value) {
    interactiveUnbinds.push(bindSfx(loginBtn.value, { hover: true, click: null }));
  }
  if (registerBtn.value) {
    interactiveUnbinds.push(bindSfx(registerBtn.value, { hover: true, click: null }));
  }
  if (rememberEl.value) {
    interactiveUnbinds.push(bindSfx(rememberEl.value, { hover: true, click: null }));
  }
}

onMounted(() => {
  bindInteractiveSfx();

  // key sounds
  if (usernameEl.value) usernameEl.value.addEventListener("keydown", typeSfx);
  if (passwordEl.value) passwordEl.value.addEventListener("keydown", typeSfx);

  const onOutsidePointer = (event) => {
    if (!serverPicker.value || !serverWrap.value) return;
    const target = event.target;
    const path = typeof event.composedPath === "function" ? event.composedPath() : [];
    const insideButton = serverBtn.value
      && (serverBtn.value.contains(target) || path.includes(serverBtn.value));
    const insideList = dropdownRef.value
      && (dropdownRef.value.contains(target) || path.includes(dropdownRef.value));
    if (insideButton || insideList) return;
    closePicker();
  };
  const onEscape = (event) => {
    if (event.key !== "Escape" || !serverPicker.value) return;
    closePicker();
  };
  document.addEventListener("pointerdown", onOutsidePointer, true);
  document.addEventListener("keydown", onEscape);
  unbinds.push(() => document.removeEventListener("pointerdown", onOutsidePointer, true));
  unbinds.push(() => document.removeEventListener("keydown", onEscape));
});

watch([loginBtn, registerBtn, rememberEl], async () => {
  await nextTick();
  bindInteractiveSfx();
});

watch(
  () => storeMut.username,
  async () => {
    await nextTick();
    if (document.activeElement !== usernameEl.value) {
      forceRepaint(usernameEl.value);
    }
  }
);

watch(
  () => storeMut.password,
  async () => {
    await nextTick();
    if (document.activeElement !== passwordEl.value) {
      forceRepaint(passwordEl.value);
    }
  }
);

onBeforeUnmount(() => {
  cleanupInteractiveBinds();
  unbinds.forEach(u => u && u());
  if (usernameEl.value) usernameEl.value.removeEventListener("keydown", typeSfx);
  if (passwordEl.value) passwordEl.value.removeEventListener("keydown", typeSfx);
});
</script>

<template>
  <div class="flex flex-col items-center w-full mt-2 px-12">
    <template v-if="isOfflineMode">
      <div class="offline-start-shell flex flex-col items-center h-[270px] pt-3">
        <div class="offline-start-art-shell mb-3">
          <img
            :src="assetUrl('/extra/AddServerToStart.png')"
            class="offline-start-art-image"
            draggable="false"
            alt=""
          />
          <div class="offline-start-art-text font-main">
            {{ $t("offline-add-server-hint") }}
          </div>
        </div>

        <div class="flex gap-4 text-2xl mb-3">
                        <button
                          ref="loginBtn"
                          class="font-main w-[220px] h-[68px] state-bg shadow shadow-md shadow-black rounded-md uppercase text-[20px]"
                          data-controller-node="login"
                          data-controller-size="big"
                          data-controller-priority="15"
                          :disabled="store.authLoading"
                          @click="onPrimaryClick"
                          :style="{ backgroundImage: `url('${classicAddServerButtonUrl}')` }"
                        >
                          {{ $t("server-add-dialog-label") }}
                        </button>
                      </div>

        <div class="flex flex-col">
          <label>{{ $t("server-select-label") }}</label>
          <div
            ref="serverWrap"
            class="h-[50x] min-w-[250px] z-[1]"
            :data-server-picker-open="serverPicker ? 'true' : null"
          >
            <div
              ref="serverBtn"
              class="box-text cursor-pointer flex items-center"
              data-controller-node="server"
              data-controller-server-toggle="true"
              data-controller-size="big"
              data-controller-clickable="true"
              data-controller-priority="12"
              :class="{ 'box-disabled': store.authLoading }"
              tabindex="0"
              @click="store.authLoading ? null : openPickerRef()"
            >
              <div class="grow">
                <span>{{ store.currentEndpoint.name }}</span>
              </div>
              <div :class="serverPicker ? 'arrow-up' : 'arrow-down'"></div>
            </div>

            <div
              v-show="serverPicker"
              ref="dropdownRef"
              class="absolute z-[2] mt-[-1px] bg-[#000000f0] border-[1px] border-t-0 border-white/20 w-[250px] cursor-pointer pt-0.5 max-h-[250px] overflow-auto scrollbar"
              data-controller-dropdown-scope="true"
            >
              <div
                v-if="store.remoteEndpoints"
                class="border-b-[1px] border-white/20"
              >
                <div
                   v-for="(endpoint, i) in filteredRemoteEndpoints"
                  :key="endpoint.url || endpoint.name || i"
                  class="text-sm flex items-center h-[20px]"
                >
                  <button
                    type="button"
                    class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                    :data-controller-node="serverMainNode(true, i)"
                    :data-controller-up="previousServerMainNode(true, i)"
                    :data-controller-down="nextServerMainNode(true, i)"
                    :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(true, i) : null"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="chooseEndpoint(endpoint)"
                  >
                    {{ endpoint.name }}
                  </button>
                  <div
                    v-if="endpointHasGear(endpoint)"
                    class="h-full w-[24px] flex-shrink-0 border-l-[1px] border-white/20"
                  >
                    <button
                      type="button"
                      class="server-picker-option-gear px-1.5 flex items-center justify-center bg-transparent border-0 w-full h-full"
                      :data-controller-node="serverGearNode(true, i)"
                      :data-controller-up="previousServerGearNode(true, i)"
                      :data-controller-down="nextServerGearNode(true, i)"
                      :data-controller-left="serverMainNode(true, i)"
                      @mouseenter="forceRepaint($event.currentTarget)"
                      @click="editEndpoint(i, true)"
                    >
                      <GearIcon />
                    </button>
                  </div>
                </div>
              </div>

              <div v-if="store.endpoints" class="border-b-[1px] border-white/20">
                <div
                   v-for="(endpoint, i) in filteredLocalEndpoints"
                  :key="endpoint.url || endpoint.name || i"
                  class="text-sm flex items-center h-[20px]"
                >
                  <button
                    type="button"
                    class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                    :data-controller-node="serverMainNode(false, i)"
                    :data-controller-up="previousServerMainNode(false, i)"
                    :data-controller-down="nextServerMainNode(false, i)"
                    :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(false, i) : null"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="chooseEndpoint(endpoint)"
                  >
                    {{ endpoint.name }}
                  </button>
                  <div
                    v-if="endpointHasGear(endpoint)"
                    class="h-full w-[24px] flex-shrink-0 border-l-[1px] border-white/20"
                  >
                    <button
                      type="button"
                      class="server-picker-option-gear px-1.5 flex items-center justify-center bg-transparent border-0 w-full h-full"
                      :data-controller-node="serverGearNode(false, i)"
                      :data-controller-up="previousServerGearNode(false, i)"
                      :data-controller-down="nextServerGearNode(false, i)"
                      :data-controller-left="serverMainNode(false, i)"
                      @mouseenter="forceRepaint($event.currentTarget)"
                      @click="editEndpoint(i, false)"
                    >
                      <GearIcon />
                    </button>
                  </div>
                </div>
              </div>

              <div class="text-sm flex items-center h-[20px]">
                <button
                  type="button"
                  class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                  data-controller-node="server-add"
                  :data-controller-up="addServerUpNode()"
                  @mouseenter="forceRepaint($event.currentTarget)"
                  @click="addEndpoint"
                >
                  {{ $t("server-add-label") }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
    <div class="min-w-[250px] flex flex-col">
      <label for="username_input">{{ $t("username-label") }}</label>
      <input
        ref="usernameEl"
        v-model="storeMut.username"
        type="text"
        id="username_input"
        class="box-text"
        data-controller-node="username"
        data-controller-size="big"
        data-controller-priority="10"
        spellcheck="false"
        :disabled="store.authLoading"
		@focus="onInputFocus"
		@blur="onInputBlur"
      />
    </div>

    <div v-if="!isOfflineMode" class="min-w-[250px] flex flex-col">
      <label for="password_input">{{ $t("password-label") }}</label>
      <input
        ref="passwordEl"
        v-model="storeMut.password"
        type="password"
        id="password_input"
        class="box-text"
        data-controller-node="password"
        data-controller-size="big"
        data-controller-priority="11"
        :disabled="store.authLoading"
		@focus="onInputFocus"
		@blur="onInputBlur"
      />
    </div>

    <div class="flex flex-col">
      <label>{{ $t("server-select-label") }}</label>
      <div
        ref="serverWrap"
        class="h-[50x] min-w-[250px] z-[1]"
        :data-server-picker-open="serverPicker ? 'true' : null"
      >
        <div
          ref="serverBtn"
		class="box-text cursor-pointer flex items-center"
          data-controller-node="server"
          data-controller-server-toggle="true"
          data-controller-size="big"
          data-controller-clickable="true"
          data-controller-priority="12"
          :class="{ 'box-disabled': store.authLoading }"
          tabindex="0"
          @click="store.authLoading ? null : openPickerRef()"
        >
          <div class="grow">
            <span>{{ store.currentEndpoint.name }}</span>
          </div>
          <div :class="serverPicker ? 'arrow-up' : 'arrow-down'"></div>
        </div>

        <div
          v-show="serverPicker"
          ref="dropdownRef"
          class="absolute z-[2] mt-[-1px] bg-[#000000f0] border-[1px] border-t-0 border-white/20 w-[250px] cursor-pointer pt-0.5 max-h-[250px] overflow-auto scrollbar"
          data-controller-dropdown-scope="true"
        >
          <div
            v-if="filteredRemoteEndpoints.length"
            class="border-b-[1px] border-white/20"
          >
            <div
              v-for="(endpoint, i) in filteredRemoteEndpoints"
              :key="endpoint.url || endpoint.name || i"
              class="text-sm flex items-center h-[20px]"
            >
              <button
                type="button"
                class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                :data-controller-node="serverMainNode(true, i)"
                :data-controller-up="previousServerMainNode(true, i)"
                :data-controller-down="nextServerMainNode(true, i)"
                :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(true, i) : null"
                @mouseenter="forceRepaint($event.currentTarget)"
                @click="chooseEndpoint(endpoint)"
              >
                {{ endpoint.name }}
              </button>
              <div
                v-if="endpointHasGear(endpoint)"
                class="h-full w-[24px] flex-shrink-0 border-l-[1px] border-white/20"
              >
                <button
                  type="button"
                  class="server-picker-option-gear px-1.5 flex items-center justify-center bg-transparent border-0 w-full h-full"
                  :data-controller-node="serverGearNode(true, i)"
                  :data-controller-up="previousServerGearNode(true, i)"
                  :data-controller-down="nextServerGearNode(true, i)"
                  :data-controller-left="serverMainNode(true, i)"
                  @mouseenter="forceRepaint($event.currentTarget)"
                  @click="editEndpoint(i, true)"
                >
                  <GearIcon />
                </button>
              </div>
            </div>
          </div>

          <div v-if="filteredLocalEndpoints.length" class="border-b-[1px] border-white/20">
            <div
              v-for="(endpoint, i) in filteredLocalEndpoints"
              :key="endpoint.url || endpoint.name || i"
              class="text-sm flex items-center h-[20px]"
            >
              <button
                type="button"
                class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                :data-controller-node="serverMainNode(false, i)"
                :data-controller-up="previousServerMainNode(false, i)"
                :data-controller-down="nextServerMainNode(false, i)"
                :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(false, i) : null"
                @mouseenter="forceRepaint($event.currentTarget)"
                @click="chooseEndpoint(endpoint)"
              >
                {{ endpoint.name }}
              </button>
              <div
                v-if="endpointHasGear(endpoint)"
                class="h-full w-[24px] flex-shrink-0 border-l-[1px] border-white/20"
              >
                <button
                  type="button"
                  class="server-picker-option-gear px-1.5 flex items-center justify-center bg-transparent border-0 w-full h-full"
                  :data-controller-node="serverGearNode(false, i)"
                  :data-controller-up="previousServerGearNode(false, i)"
                  :data-controller-down="nextServerGearNode(false, i)"
                  :data-controller-left="serverMainNode(false, i)"
                  @mouseenter="forceRepaint($event.currentTarget)"
                  @click="editEndpoint(i, false)"
                >
                  <GearIcon />
                </button>
              </div>
            </div>
          </div>

          <div class="text-sm flex items-center h-[20px]">
                          <button
                            type="button"
                            class="server-picker-option px-2 grow text-left flex items-center bg-transparent border-0 w-full h-full block"
                            data-controller-node="server-add"
                            :data-controller-up="addServerUpNode()"
                            @mouseenter="forceRepaint($event.currentTarget)"
                            @click="addEndpoint"
                          >
                            {{ $t("server-add-label") }}
                          </button>
                        </div>
        </div>
      </div>
    </div>

    <div class="flex gap-4 mt-6 text-2xl">
      <button
        ref="loginBtn"
        class="font-main w-[160px] h-[56px] state-bg shadow shadow-md shadow-black rounded-md uppercase"
        data-controller-node="login"
        data-controller-size="big"
        data-controller-priority="15"
        :disabled="store.authLoading"
        @click="onPrimaryClick"
        :style="{ backgroundImage: `url('${classicButtonUrl}')` }"
      >
        {{ $t("login-button") }}
      </button>

      <button
        ref="registerBtn"
        class="font-main w-[160px] h-[56px] state-bg shadow shadow-md shadow-black rounded-md uppercase"
        data-controller-node="register"
        data-controller-size="big"
        data-controller-priority="14"
        :disabled="store.authLoading"
        @click="onRegisterClick"
        :style="{ backgroundImage: `url('${classicButtonUrl}')` }"
      >
        {{ $t("register-button") }}
      </button>
    </div>

    <label
      ref="rememberEl"
	  class="flex gap-2 items-center hover:brightness-150 mt-2"
      data-controller-node="remember"
      data-controller-size="big"
      data-controller-clickable="true"
      data-controller-priority="13"
      tabindex="0"
      :class="store.authLoading ? 'disabled' : 'cursor-pointer'"
      @click="store.authLoading ? null : onRememberClick()"
    >
      <img
        :src="assetUrl('/classic/checkbox.png')"
        draggable="false"
        class="h-[12px] w-[11px] object-none"
        :class="storeMut.rememberMe ? 'object-top' : 'object-bottom'"
      />
      <span class="text-sm">{{ $t("remember-me-label") }}</span>
    </label>
    </template>
  </div>
</template>

<style scoped>
.offline-start-shell {
  transform: translateY(-5px);
}

.offline-start-art-shell {
  position: relative;
  width: 278px;
  height: 121px;
}

.offline-start-art-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.offline-start-art-text {
  position: absolute;
  left: 15px;
  top: 11px;
  width: 148px;
  color: #f3eee3;
  text-align: center;
  font-size: 18px;
  line-height: 1.18;
  text-shadow: 0 2px 2px rgba(0, 0, 0, 0.9);
  pointer-events: none;
  white-space: normal;
}

.server-picker-option.controller-nav-focused,
.server-picker-option:hover:not(:disabled),
.server-picker-option:focus-visible:not(:disabled) {
  color: var(--controller-focus-color);
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}

.server-picker-option-gear.controller-nav-focused,
.server-picker-option-gear:hover:not(:disabled),
.server-picker-option-gear:focus-visible:not(:disabled) {
  color: var(--controller-focus-color);
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}
</style>
