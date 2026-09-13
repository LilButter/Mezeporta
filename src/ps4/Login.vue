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

import { playHover, playSelect, playConfirm, playQuickSelect, bindSfx } from "../sfx";

const serverPicker = ref(false);

const usernameEl  = ref(null);
const passwordEl  = ref(null);
const loginBtn    = ref(null);
const registerBtn = ref(null);
const serverBtn   = ref(null);
const rememberEl  = ref(null);
const serverWrap  = ref(null);
const dropdownRef = ref(null);

const isOfflineMode = computed(() => store.currentEndpoint?.url === "OFFLINEMODE");

function isOfflineEndpoint(endpoint) {
  const url = String(endpoint?.url ?? "").trim().toUpperCase();
  const name = String(endpoint?.name ?? "").trim().toUpperCase();
  return url === "OFFLINEMODE" || name === "OFFLINE-MODE";
}

const offlineEndpoint = computed(() => {
  const fromRemote = (store.remoteEndpoints ?? []).find(isOfflineEndpoint);
  if (fromRemote) return fromRemote;
  const fromLocal = (store.endpoints ?? []).find(isOfflineEndpoint);
  if (fromLocal) return fromLocal;
  return { name: "Offline-Mode", url: "OFFLINEMODE" };
});

function isCurrentEndpoint(endpoint) {
  if (!endpoint || !store.currentEndpoint) return false;
  if (endpoint.url === "OFFLINEMODE" && store.currentEndpoint.url === "OFFLINEMODE") return true;
  return endpoint.url === store.currentEndpoint.url && endpoint.name === store.currentEndpoint.name;
}

function endpointHasGear(endpoint) {
  const url = String(endpoint?.url ?? "").trim().toUpperCase();
  const name = String(endpoint?.name ?? "").trim().toUpperCase();
  return Boolean(url) && url !== "OFFLINEMODE" && name !== "OFFLINE-MODE";
}

const canEditCurrentEndpoint = computed(() => {
  if (!store.currentEndpoint) return false;
  return endpointHasGear(store.currentEndpoint);
});

const selectableServers = computed(() => {
  const list = [];
  (store.remoteEndpoints ?? []).forEach((endpoint, index) => {
    if (!isOfflineEndpoint(endpoint) && !isCurrentEndpoint(endpoint)) {
      list.push({
        endpoint,
        isRemote: true,
        originalIndex: index,
        hasGear: endpointHasGear(endpoint),
      });
    }
  });
  (store.endpoints ?? []).forEach((endpoint, index) => {
    if (!isOfflineEndpoint(endpoint) && !isCurrentEndpoint(endpoint)) {
      list.push({
        endpoint,
        isRemote: false,
        originalIndex: index,
        hasGear: endpointHasGear(endpoint),
      });
    }
  });

  if (!isOfflineMode.value) {
    list.push({
      endpoint: offlineEndpoint.value,
      isRemote: false,
      originalIndex: -1,
      hasGear: false,
    });
  }

  return list;
});

const selectableGearIndexes = computed(() =>
  selectableServers.value
    .map((item, index) => (item.hasGear ? index : null))
    .filter((index) => index !== null)
);

function serverItemNode(index) {
  return `server-item-main-${index}`;
}

function serverItemGearNode(index) {
  return `server-item-gear-${index}`;
}

function previousServerItemNode(index) {
  return index > 0 ? serverItemNode(index - 1) : null;
}

function nextServerItemNode(index) {
  if (index + 1 < selectableServers.value.length) {
    return serverItemNode(index + 1);
  }
  return canEditCurrentEndpoint.value ? "server-edit" : "server-add";
}

function previousServerItemGearNode(index) {
  const currentIndex = selectableGearIndexes.value.indexOf(index);
  if (currentIndex <= 0) return null;
  return serverItemGearNode(selectableGearIndexes.value[currentIndex - 1]);
}

function nextServerItemGearNode(index) {
  const currentIndex = selectableGearIndexes.value.indexOf(index);
  if (currentIndex === -1 || currentIndex + 1 >= selectableGearIndexes.value.length) {
    return "server-add";
  }
  return serverItemGearNode(selectableGearIndexes.value[currentIndex + 1]);
}

function actionRowUpNode() {
  if (selectableServers.value.length > 0) {
    return serverItemNode(selectableServers.value.length - 1);
  }
  return null;
}

function actionRowGearUpNode() {
  if (selectableServers.value.length > 0) {
    const lastIdx = selectableServers.value.length - 1;
    if (selectableServers.value[lastIdx].hasGear) {
      return serverItemGearNode(lastIdx);
    }
    return serverItemNode(lastIdx);
  }
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
  playQuickSelect(); // open/close sound
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

function editCurrentEndpoint() {
  playSelect();
  serverPicker.value = false;
  if (!store.currentEndpoint) return;
  const remoteIdx = (store.remoteEndpoints ?? []).findIndex(
    (ep) => ep.url === store.currentEndpoint?.url && ep.name === store.currentEndpoint?.name
  );
  if (remoteIdx !== -1) {
    dialogEditEndpoint(remoteIdx, true);
    return;
  }
  const localIdx = (store.endpoints ?? []).findIndex(
    (ep) => ep.url === store.currentEndpoint?.url && ep.name === store.currentEndpoint?.name
  );
  if (localIdx !== -1) {
    dialogEditEndpoint(localIdx, false);
    return;
  }
}

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
                    v-if="selectableServers.length"
                    class="ps4-server-dropdown-group"
                  >
                    <div
                      v-for="(item, i) in selectableServers"
                      :key="item.endpoint.url || item.endpoint.name || i"
                      class="ps4-server-option"
                      :class="{ 'ps4-server-option-no-gear': !item.hasGear }"
                    >
                      <button
                        type="button"
                        class="ps4-server-option-main"
                        :data-controller-node="serverItemNode(i)"
                        :data-controller-up="previousServerItemNode(i)"
                        :data-controller-down="nextServerItemNode(i)"
                        :data-controller-right="item.hasGear ? serverItemGearNode(i) : null"
                        @mouseenter="forceRepaint($event.currentTarget)"
                        @click="chooseEndpoint(item.endpoint)"
                      >
                        <span class="truncate">{{ item.endpoint.name }}</span>
                      </button>
                      <button
                        v-if="item.hasGear"
                        type="button"
                        class="ps4-server-option-gear"
                        :data-controller-node="serverItemGearNode(i)"
                        :data-controller-up="previousServerItemGearNode(i)"
                        :data-controller-down="nextServerItemGearNode(i)"
                        :data-controller-left="serverItemNode(i)"
                        @mouseenter="forceRepaint($event.currentTarget)"
                        @click="editEndpoint(item.originalIndex, item.isRemote)"
                      >
                        <GearIcon />
                      </button>
                    </div>
                  </div>

                  <div class="ps4-server-dropdown-group ps4-server-actions-row">
                    <button
                      v-if="canEditCurrentEndpoint"
                      type="button"
                      class="ps4-server-action-btn ps4-server-action-edit no-button-image"
                      data-controller-node="server-edit"
                      :data-controller-up="actionRowUpNode()"
                      data-controller-right="server-add"
                      @mouseenter="forceRepaint($event.currentTarget)"
                      @click="editCurrentEndpoint"
                    >
                      <span class="ps4-action-text ps4-action-short truncate">{{ $t("server-edit-short-label") }}</span>
                      <span class="ps4-action-text ps4-action-full truncate">{{ $t("server-edit-label") }}</span>
                    </button>
                    <button
                      type="button"
                      class="ps4-server-action-btn ps4-server-action-add no-button-image"
                      :class="{ 'ps4-server-action-solo': !canEditCurrentEndpoint }"
                      data-controller-node="server-add"
                      :data-controller-up="canEditCurrentEndpoint ? actionRowGearUpNode() : actionRowUpNode()"
                      :data-controller-left="canEditCurrentEndpoint ? 'server-edit' : null"
                      @mouseenter="forceRepaint($event.currentTarget)"
                      @click="addEndpoint"
                    >
                      <span v-if="canEditCurrentEndpoint" class="ps4-action-text ps4-action-short truncate">{{ $t("server-add-short-label") }}</span>
                      <span class="ps4-action-text ps4-action-full truncate">{{ $t("server-add-label") }}</span>
                    </button>
                  </div>
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
                v-if="selectableServers.length"
                class="ps4-server-dropdown-group"
              >
                <div
                  v-for="(item, i) in selectableServers"
                  :key="item.endpoint.url || item.endpoint.name || i"
                  class="ps4-server-option"
                  :class="{ 'ps4-server-option-no-gear': !item.hasGear }"
                >
                  <button
                    type="button"
                    class="ps4-server-option-main"
                    :data-controller-node="serverItemNode(i)"
                    :data-controller-up="previousServerItemNode(i)"
                    :data-controller-down="nextServerItemNode(i)"
                    :data-controller-right="item.hasGear ? serverItemGearNode(i) : null"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="chooseEndpoint(item.endpoint)"
                  >
                    <span class="truncate">{{ item.endpoint.name }}</span>
                  </button>
                  <button
                    v-if="item.hasGear"
                    type="button"
                    class="ps4-server-option-gear"
                    :data-controller-node="serverItemGearNode(i)"
                    :data-controller-up="previousServerItemGearNode(i)"
                    :data-controller-down="nextServerItemGearNode(i)"
                    :data-controller-left="serverItemNode(i)"
                    @mouseenter="forceRepaint($event.currentTarget)"
                    @click="editEndpoint(item.originalIndex, item.isRemote)"
                  >
                    <GearIcon />
                  </button>
                </div>
              </div>

              <div class="ps4-server-dropdown-group ps4-server-actions-row">
                <button
                  v-if="canEditCurrentEndpoint"
                  type="button"
                  class="ps4-server-action-btn ps4-server-action-edit no-button-image"
                  data-controller-node="server-edit"
                  :data-controller-up="actionRowUpNode()"
                  data-controller-right="server-add"
                  @mouseenter="forceRepaint($event.currentTarget)"
                  @click="editCurrentEndpoint"
                >
                  <span class="ps4-action-text ps4-action-short truncate">{{ $t("server-edit-short-label") }}</span>
                  <span class="ps4-action-text ps4-action-full truncate">{{ $t("server-edit-label") }}</span>
                </button>
                <button
                  type="button"
                  class="ps4-server-action-btn ps4-server-action-add no-button-image"
                  :class="{ 'ps4-server-action-solo': !canEditCurrentEndpoint }"
                  data-controller-node="server-add"
                  :data-controller-up="canEditCurrentEndpoint ? actionRowGearUpNode() : actionRowUpNode()"
                  :data-controller-left="canEditCurrentEndpoint ? 'server-edit' : null"
                  @mouseenter="forceRepaint($event.currentTarget)"
                  @click="addEndpoint"
                >
                  <span v-if="canEditCurrentEndpoint" class="ps4-action-text ps4-action-short truncate">{{ $t("server-add-short-label") }}</span>
                  <span class="ps4-action-text ps4-action-full truncate">{{ $t("server-add-label") }}</span>
                </button>
              </div>
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
.ps4-server-option-add::before,
.ps4-server-action-btn::before {
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
.ps4-server-option-add:focus-visible:not(:disabled),
.ps4-server-action-btn.controller-nav-focused,
.ps4-server-action-btn:hover:not(:disabled),
.ps4-server-action-btn:focus-visible:not(:disabled) {
  color: var(--controller-focus-color);
  background: rgba(255, 255, 255, 0.16);
  outline: none !important;
  box-shadow: none !important;
  border-color: transparent !important;
}

.ps4-server-action-btn.ps4-server-action-edit {
  border-right: 1px solid rgba(255, 255, 255, 0.12) !important;
}

/* Offline mode login button uses ButtonALT.png */
.ps4-login-offline-shell .ps4-start-button {
  --button-image: url('/ps4/ButtonALT4.png');
}
</style>
