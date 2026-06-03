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
  <div class="ps4-login-panel">
    <div class="ps4-login-frame">
      <template v-if="isOfflineMode">
        <div class="ps4-login-offline-shell">
          <div class="ps4-login-offline-art-row">
            <div class="ps4-login-offline-art-shell">
              <img
                :src="assetUrl('/extra/AddServerToStart.png')"
                class="ps4-login-offline-art"
                draggable="false"
                alt=""
              />
              <div class="ps4-login-offline-art-text font-main">
                {{ $t("offline-add-server-hint") }}
              </div>
            </div>
          </div>

          <div class="ps4-login-button-row">
            <button
              ref="loginBtn"
              class="ps4-start-button font-main"
              data-controller-node="login"
              data-controller-size="big"
              data-controller-priority="15"
              :disabled="store.authLoading"
              @click="onPrimaryClick"
            >
              <span class="ps4-start-button-label">{{ $t("server-add-dialog-label") }}</span>
            </button>
          </div>

          <div class="ps4-login-offline-server-row">
            <div class="ps4-login-field ps4-login-field-full">
              <span class="ps4-login-field-label">{{ $t("server-select-label") }}</span>
              <div
                ref="serverWrap"
                class="ps4-server-picker-wrap"
                :data-server-picker-open="serverPicker ? 'true' : null"
              >
                <button
                  ref="serverBtn"
                  type="button"
                  class="ps4-server-picker"
                  data-controller-node="server"
                  data-controller-server-toggle="true"
                  data-controller-size="big"
                  data-controller-priority="12"
                  :class="{ 'box-disabled': store.authLoading }"
                  @click="store.authLoading ? null : openPickerRef()"
                >
                  <span class="truncate">{{ store.currentEndpoint.name }}</span>
                  <div :class="serverPicker ? 'arrow-up' : 'arrow-down'"></div>
                </button>

                <div
                  v-show="serverPicker"
                  ref="dropdownRef"
                  class="ps4-server-dropdown scrollbar"
                  data-controller-dropdown-scope="true"
                >
                  <div
                    v-if="filteredRemoteEndpoints.length"
                    class="ps4-server-dropdown-group"
                  >
                    <div
                      v-for="(endpoint, i) in filteredRemoteEndpoints"
                      :key="endpoint.url || endpoint.name || i"
                      class="ps4-server-option"
                      :class="{ 'ps4-server-option-no-gear': !endpointHasGear(endpoint) }"
                    >
                      <button
                        type="button"
                        class="ps4-server-option-main"
                        :data-controller-node="serverMainNode(true, i)"
                        :data-controller-up="previousServerMainNode(true, i)"
                        :data-controller-down="nextServerMainNode(true, i)"
                        :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(true, i) : null"
                        @mouseenter="forceRepaint($event.currentTarget)"
                        @click="chooseEndpoint(endpoint)"
                      >
                        {{ endpoint.name }}
                      </button>
                      <button
                        v-if="endpointHasGear(endpoint)"
                        type="button"
                        class="ps4-server-option-gear"
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

                  <div v-if="filteredLocalEndpoints.length" class="ps4-server-dropdown-group">
                    <div
                      v-for="(endpoint, i) in filteredLocalEndpoints"
                      :key="endpoint.url || endpoint.name || i"
                      class="ps4-server-option"
                      :class="{ 'ps4-server-option-no-gear': !endpointHasGear(endpoint) }"
                    >
                      <button
                        type="button"
                        class="ps4-server-option-main"
                        :data-controller-node="serverMainNode(false, i)"
                        :data-controller-up="previousServerMainNode(false, i)"
                        :data-controller-down="nextServerMainNode(false, i)"
                        :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(false, i) : null"
                        @mouseenter="forceRepaint($event.currentTarget)"
                        @click="chooseEndpoint(endpoint)"
                      >
                        {{ endpoint.name }}
                      </button>
                      <button
                        v-if="endpointHasGear(endpoint)"
                        type="button"
                        class="ps4-server-option-gear"
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

                  <button
                    type="button"
                    class="ps4-server-option-add no-button-image ps4-server-option-no-gear"
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
      <div class="ps4-login-top-row">
        <label class="ps4-login-field">
          <span class="ps4-login-field-label">{{ $t("username-label") }}</span>
          <input
            ref="usernameEl"
            v-model="storeMut.username"
            type="text"
            id="username_input"
            class="ps4-login-input"
            data-controller-node="username"
            data-controller-size="big"
            data-controller-priority="10"
            spellcheck="false"
            :disabled="store.authLoading"
            @focus="onInputFocus"
            @blur="onInputBlur"
          />
        </label>

        <label class="ps4-login-field">
          <span class="ps4-login-field-label">{{ $t("password-label") }}</span>
          <input
            ref="passwordEl"
            v-model="storeMut.password"
            type="password"
            id="password_input"
            class="ps4-login-input"
            data-controller-node="password"
            data-controller-size="big"
            data-controller-priority="11"
            :disabled="store.authLoading"
            @focus="onInputFocus"
            @blur="onInputBlur"
          />
        </label>
      </div>

      <div class="ps4-login-subrow">
        <label
          ref="rememberEl"
          class="ps4-remember-row"
          data-controller-node="remember"
          data-controller-size="big"
          data-controller-clickable="true"
          data-controller-priority="13"
          tabindex="0"
          :class="store.authLoading ? 'disabled' : 'cursor-pointer'"
          @click="store.authLoading ? null : onRememberClick()"
        >
          <img
            :src="assetUrl('/ps4/checkbox.png')"
            draggable="false"
            class="h-[12px] w-[11px] object-none"
            :class="storeMut.rememberMe ? 'object-top' : 'object-bottom'"
          />
          <span>{{ $t("remember-me-label") }}</span>
        </label>
      </div>

      <div class="ps4-login-middle-row">
        <div class="ps4-login-field">
          <span class="ps4-login-field-label">{{ $t("server-select-label") }}</span>
          <div
            ref="serverWrap"
            class="ps4-server-picker-wrap"
            :data-server-picker-open="serverPicker ? 'true' : null"
          >
            <button
              ref="serverBtn"
              type="button"
              class="ps4-server-picker"
              data-controller-node="server"
              data-controller-server-toggle="true"
              data-controller-size="big"
              data-controller-priority="12"
              :class="{ 'box-disabled': store.authLoading }"
              @click="store.authLoading ? null : openPickerRef()"
            >
              <span class="truncate">{{ store.currentEndpoint.name }}</span>
              <div :class="serverPicker ? 'arrow-up' : 'arrow-down'"></div>
            </button>

            <div
              v-show="serverPicker"
              ref="dropdownRef"
              class="ps4-server-dropdown scrollbar"
              data-controller-dropdown-scope="true"
            >
              <div
                v-if="filteredRemoteEndpoints.length"
                class="ps4-server-dropdown-group"
              >
                <div
                  v-for="(endpoint, i) in filteredRemoteEndpoints"
                  :key="endpoint.url || endpoint.name || i"
                  class="ps4-server-option"
                  :class="{ 'ps4-server-option-no-gear': !endpointHasGear(endpoint) }"
                >
                  <button
                    type="button"
                    class="ps4-server-option-main"
                    :data-controller-node="serverMainNode(true, i)"
                    :data-controller-up="previousServerMainNode(true, i)"
                    :data-controller-down="nextServerMainNode(true, i)"
                    :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(true, i) : null"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="chooseEndpoint(endpoint)"
                  >
                    <span class="truncate block">{{ endpoint.name }}</span>
                  </button>
                  <button
                    v-if="endpointHasGear(endpoint)"
                    type="button"
                    class="ps4-server-option-gear"
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

              <div v-if="filteredLocalEndpoints.length" class="ps4-server-dropdown-group">
                <div
                  v-for="(endpoint, i) in filteredLocalEndpoints"
                  :key="endpoint.url || endpoint.name || i"
                  class="ps4-server-option"
                  :class="{ 'ps4-server-option-no-gear': !endpointHasGear(endpoint) }"
                >
                  <button
                    type="button"
                    class="ps4-server-option-main"
                    :data-controller-node="serverMainNode(false, i)"
                    :data-controller-up="previousServerMainNode(false, i)"
                    :data-controller-down="nextServerMainNode(false, i)"
                    :data-controller-right="endpointHasGear(endpoint) ? serverGearNode(false, i) : null"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="chooseEndpoint(endpoint)"
                  >
                    <span class="truncate block">{{ endpoint.name }}</span>
                  </button>
                  <button
                    v-if="endpointHasGear(endpoint)"
                    type="button"
                    class="ps4-server-option-gear"
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

              <button
                type="button"
                class="ps4-server-option-add no-button-image ps4-server-option-no-gear"
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
        <div class="ps4-login-side-actions">
          <button
            ref="registerBtn"
            class="ps4-register-link"
            data-controller-node="register"
            data-controller-size="big"
            data-controller-priority="14"
            :disabled="store.authLoading"
            @click="onRegisterClick"
          >
            {{ $t("register-button") }}
          </button>
        </div>
      </div>

      <div class="ps4-login-button-row">
        <button
          ref="loginBtn"
          class="ps4-start-button font-main"
          data-controller-node="login"
          data-controller-size="big"
          data-controller-priority="15"
          :disabled="store.authLoading"
          @click="onPrimaryClick"
        >
          <span class="ps4-start-button-label">{{ $t("login-button") }}</span>
        </button>
      </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.ps4-login-offline-shell {
  transform: translateY(-13px);
}

.ps4-login-offline-art-row {
  display: flex;
  justify-content: center;
  width: 349px;
  margin-left: 83px;
  margin-bottom: 6px;
}

.ps4-login-offline-art-shell {
  position: relative;
  width: 278px;
  height: 121px;
}

.ps4-login-offline-art {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.ps4-login-offline-art-text {
  position: absolute;
  left: 15px;
  top: 15px;
  width: 148px;
  color: #f3eee3;
  text-align: center;
  font-size: 18px;
  line-height: 1.18;
  text-shadow: 0 2px 2px rgba(0, 0, 0, 0.9);
  pointer-events: none;
  white-space: normal;
}

.ps4-login-offline-server-row {
  display: flex;
  justify-content: center;
  width: 349px;
  margin-left: 83px;
  margin-top: 10px;
}

.ps4-login-field-full {
  width: 349px;
}

/* Remove background image from Add Server button in dropdown */
.ps4-server-option-add::before {
  display: none !important;
}

.ps4-server-option-main.controller-nav-focused,
.ps4-server-option-main:hover:not(:disabled),
.ps4-server-option-main:focus-visible:not(:disabled),
.ps4-server-option-gear.controller-nav-focused,
.ps4-server-option-gear:hover:not(:disabled),
.ps4-server-option-gear:focus-visible:not(:disabled),
.ps4-server-option-add.controller-nav-focused,
.ps4-server-option-add:hover:not(:disabled),
.ps4-server-option-add:focus-visible:not(:disabled) {
  color: var(--controller-focus-color);
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}

/* Offline mode login button uses ButtonALT.png */
.ps4-login-offline-shell .ps4-start-button {
  --button-image: url('/ps4/ButtonALT4.png');
}
</style>
