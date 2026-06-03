import { invoke as tauriInvoke } from "@tauri-apps/api/tauri";

const LOCAL_STATE_KEY = "mezeporta_web_state_v1";
const LEGACY_LOCAL_STATE_KEY = "butter_web_state_v1";
const BRIDGE_KEY = "__MEZEPORTA_BRIDGE__";
const LEGACY_BRIDGE_KEY = "__BUTTER_BRIDGE__";
const DEFAULT_STATE = {
  style: 1,
  locale: "en",
  username: "",
  password: "",
  rememberMe: false,
  gameFolder: "",
  serverlistUrl: "",
  messagelistUrl: "",
  endpoints: [
    {
      name: "Offline",
      url: "OFFLINEMODE",
      isRemote: false,
      host: "",
      launcherPort: null,
      gamePort: null,
      gamePath: null,
      version: null,
    },
  ],
  remoteEndpoints: [],
  currentEndpoint: null,
  settings: {},
  launcherPrefs: {
    preloadControllerDlls: false,
    friendSignature: "none",
    winePrefixMode: "portable",
    winePrefixCustomPath: null,
  },
  uiPrefs: {
    sfxEnabled: false,
    sfxVolume: 30,
    fontPreset: "default",
    classicLauncherWidth: 1124,
    classicLauncherHeight: 600,
    classicLauncherRecentResolutions: [],
    ps4LauncherWidth: 1280,
    ps4LauncherHeight: 720,
    ps4LauncherRecentResolutions: [],
    gameVersion: "ZZ",
    forceGameVersion: false,
    devMode: false,
    serverVersionPromptDisabled: false,
    launcherController: true,
    linuxHardwareAcceleration: true,
    offlineImages: false,
  },
};

function normalizeWinePrefixMode(value) {
  const normalized = String(value ?? "").trim().toLowerCase();
  if (["system", "custom", "proton"].includes(normalized)) return normalized;
  return "portable";
}

function normalizeWinePrefixCustomPath(value) {
  if (value === null || value === undefined) return null;
  const normalized = String(value).trim();
  return normalized ? normalized : null;
}

function normalizeLauncherPrefs(prefs = {}) {
  return {
    preloadControllerDlls: Boolean(
      prefs?.preloadControllerDlls ?? DEFAULT_STATE.launcherPrefs.preloadControllerDlls
    ),
    friendSignature: String(
      prefs?.friendSignature ?? DEFAULT_STATE.launcherPrefs.friendSignature
    ),
    winePrefixMode: normalizeWinePrefixMode(
      prefs?.winePrefixMode ?? DEFAULT_STATE.launcherPrefs.winePrefixMode
    ),
    winePrefixCustomPath: normalizeWinePrefixCustomPath(
      prefs?.winePrefixCustomPath ?? DEFAULT_STATE.launcherPrefs.winePrefixCustomPath
    ),
  };
}

function createDefaultState() {
  return {
    ...DEFAULT_STATE,
    endpoints: DEFAULT_STATE.endpoints.map((entry) => ({ ...entry })),
    remoteEndpoints: [],
    settings: {},
    launcherPrefs: normalizeLauncherPrefs(DEFAULT_STATE.launcherPrefs),
    uiPrefs: { ...DEFAULT_STATE.uiPrefs },
  };
}

function loadLocalState() {
  if (typeof window === "undefined") return createDefaultState();
  try {
    const raw =
      window.localStorage.getItem(LOCAL_STATE_KEY) ??
      window.localStorage.getItem(LEGACY_LOCAL_STATE_KEY);
    const parsed = raw ? JSON.parse(raw) : {};
    const nextState = {
      ...createDefaultState(),
      ...parsed,
      endpoints: Array.isArray(parsed?.endpoints)
        ? parsed.endpoints
        : createDefaultState().endpoints,
      remoteEndpoints: Array.isArray(parsed?.remoteEndpoints)
        ? parsed.remoteEndpoints
        : [],
      settings:
        parsed?.settings && typeof parsed.settings === "object"
          ? parsed.settings
          : {},
      launcherPrefs: normalizeLauncherPrefs(parsed?.launcherPrefs),
      uiPrefs: {
        ...DEFAULT_STATE.uiPrefs,
        ...(parsed?.uiPrefs && typeof parsed.uiPrefs === "object"
          ? parsed.uiPrefs
          : {}),
      },
    };
    if (
      !window.localStorage.getItem(LOCAL_STATE_KEY) &&
      window.localStorage.getItem(LEGACY_LOCAL_STATE_KEY)
    ) {
      saveLocalState(nextState);
    }
    return nextState;
  } catch (_e) {
    return createDefaultState();
  }
}

function saveLocalState(state) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(LOCAL_STATE_KEY, JSON.stringify(state));
    window.localStorage.removeItem(LEGACY_LOCAL_STATE_KEY);
  } catch (_e) {
    // ignore storage errors
  }
}

const localEventListeners = new Set();

// Electron's contextBridge uses structured clone between renderer and preload.
// Vue reactive proxies (and some complex types) are not cloneable, so sanitize
// on the renderer side before crossing the boundary.
function sanitizeForBridge(value, seen = new WeakMap()) {
  if (value == null) return value;

  const t = typeof value;
  if (t === "string" || t === "number" || t === "boolean") return value;
  if (t === "bigint") {
    const asNum = Number(value);
    return Number.isFinite(asNum) ? asNum : value.toString();
  }
  if (t === "function" || t === "symbol" || t === "undefined") return null;

  if (value instanceof Date) return value.toISOString();
  if (value instanceof Error) {
    return { name: value.name, message: value.message, stack: value.stack };
  }

  if (typeof value === "object") {
    if (seen.has(value)) return null;
    seen.set(value, true);

    if (Array.isArray(value)) {
      return value.map((v) => sanitizeForBridge(v, seen));
    }

    if (value instanceof Map) {
      const obj = {};
      for (const [k, v] of value.entries()) obj[String(k)] = sanitizeForBridge(v, seen);
      return obj;
    }
    if (value instanceof Set) {
      return Array.from(value.values()).map((v) => sanitizeForBridge(v, seen));
    }

    if (ArrayBuffer.isView(value)) {
      return Array.from(value);
    }
    if (value instanceof ArrayBuffer) {
      return Array.from(new Uint8Array(value));
    }

    const out = {};
    for (const key of Object.keys(value)) {
      out[key] = sanitizeForBridge(value[key], seen);
    }
    return out;
  }

  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_e) {
    return String(value);
  }
}

function emitLocalEvent(payload) {
  for (const handler of localEventListeners) {
    try {
      handler(payload);
    } catch (_e) {
      // ignore listener errors
    }
  }
}

function ensureCurrentEndpoint(state) {
  if (state.currentEndpoint) return;
  state.currentEndpoint =
    state.remoteEndpoints?.[0] ??
    state.endpoints?.[0] ?? {
      name: "Offline",
      url: "OFFLINEMODE",
      isRemote: false,
      host: "",
      launcherPort: null,
      gamePort: null,
      gamePath: null,
      version: null,
    };
}

async function localInvoke(cmd, args = {}) {
  const state = loadLocalState();
  ensureCurrentEndpoint(state);
  switch (cmd) {
    case "initial_data": {
      return {
        style: state.style,
        locale: state.locale,
        username: state.username,
        password: state.password,
        rememberMe: state.rememberMe,
        gameFolder: state.gameFolder,
        serverlistUrl: state.serverlistUrl,
        messagelistUrl: state.messagelistUrl,
        endpoints: state.endpoints,
        remoteEndpoints: state.remoteEndpoints,
        currentEndpoint: state.currentEndpoint,
        currentFolder: "",
        lastCharId: null,
        settings: state.settings ?? {},
        uiPrefs: state.uiPrefs ?? {},
        launcherPrefs: normalizeLauncherPrefs(state.launcherPrefs),
        banners: [],
        links: [],
        characters: [],
        messages: [],
        remoteMessages: [],
        background: null,
        cog: null,
        capcom: null,
        button: null,
        launcher_header: null,
        headers: null,
        dialog: null,
        server_patch: null,
        ps4: null,
      };
    }
    case "set_style":
      state.style = args?.style ?? state.style;
      saveLocalState(state);
      return { ok: true };
    case "set_locale":
      state.locale = args?.locale ?? state.locale;
      saveLocalState(state);
      return { ok: true };
    case "set_game_folder":
      state.gameFolder = args?.gameFolder ?? "";
      saveLocalState(state);
      return { ok: true };
    case "set_serverlist_url":
      state.serverlistUrl = args?.serverlistUrl ?? "";
      saveLocalState(state);
      return { ok: true };
    case "set_messagelist_url":
      state.messagelistUrl = args?.messagelistUrl ?? "";
      saveLocalState(state);
      return { ok: true };
    case "set_setting": {
      const { setting, value } = args ?? {};
      if (setting) {
        state.settings = { ...(state.settings ?? {}), [setting]: value };
        saveLocalState(state);
      }
      return { ok: true };
    }
    case "set_launcher_pref": {
      const payload = args?.prefs ?? args ?? {};
      state.launcherPrefs = {
        ...normalizeLauncherPrefs(state.launcherPrefs),
        ...normalizeLauncherPrefs({
          preloadControllerDlls:
            payload?.preloadControllerDlls === undefined
              ? state.launcherPrefs?.preloadControllerDlls
              : payload?.preloadControllerDlls,
          friendSignature:
            payload?.friendSignature === undefined
              ? state.launcherPrefs?.friendSignature
              : payload?.friendSignature,
          winePrefixMode:
            payload?.winePrefixMode === undefined
              ? state.launcherPrefs?.winePrefixMode
              : payload?.winePrefixMode,
          winePrefixCustomPath:
            payload?.winePrefixCustomPath === undefined
              ? state.launcherPrefs?.winePrefixCustomPath
              : payload?.winePrefixCustomPath,
        }),
      };
      saveLocalState(state);
      return { ok: true };
    }
    case "set_ui_pref": {
      const name = args?.name;
      const value = args?.value;
      if (name) {
        state.uiPrefs = { ...(state.uiPrefs ?? {}) };
        state.uiPrefs[name] = value;
        if (name === "gameVersion" && state.currentEndpoint) {
          state.currentEndpoint = {
            ...state.currentEndpoint,
            version: value,
          };
        }
        saveLocalState(state);
      }
      return { ok: true };
    }
    case "get_linux_prefix_status":
      return {
        prefixPath: "",
        ready: false,
        missingTools: [],
      };
    case "install_linux_portable_prefix":
      return {
        prefixPath: "",
        ready: false,
        missingTools: [],
      };
    case "detect_game_version":
      return null;
    case "patcher_swap_info":
      return {
        activeServer: null,
        activeHasManifest: false,
        targetHasManifest: false,
        targetHasCache: false,
      };
    case "patcher_swap_to_cached":
      return { ok: true };
    case "reset_game_files":
      return { ok: true };
    case "set_endpoints": {
      const endpoints = Array.isArray(args?.endpoints) ? args.endpoints : [];
      state.endpoints = endpoints;
      ensureCurrentEndpoint(state);
      saveLocalState(state);
      return state.currentEndpoint;
    }
    case "set_remote_endpoints": {
      const endpoints = Array.isArray(args?.endpoints) ? args.endpoints : [];
      state.remoteEndpoints = endpoints;
      ensureCurrentEndpoint(state);
      saveLocalState(state);
      return state.currentEndpoint;
    }
    case "set_current_endpoint": {
      state.currentEndpoint = args?.currentEndpoint ?? state.currentEndpoint;
      ensureCurrentEndpoint(state);
      saveLocalState(state);
      emitLocalEvent({
        type: "userdata",
        payload: {
          userdata: {
            username: state.username,
            rememberMe: state.rememberMe,
          },
          password: state.password,
        },
      });
      emitLocalEvent({
        type: "endpoints",
        payload: {
          endpoints: state.endpoints,
          remoteEndpoints: state.remoteEndpoints,
        },
      });
      return {
        banners: [],
        messages: [],
        links: [],
        background: null,
        cog: null,
        capcom: null,
        button: null,
        launcher_header: null,
        headers: null,
        dialog: null,
        server_patch: null,
        ps4: null,
      };
    }
    case "shutdown_launcher": {
      await getBridge()?.windowClose?.();
      return { ok: true };
    }
    default:
      throw new Error(`unimplemented method: ${cmd}`);
  }
}

function getBridge() {
  if (typeof window === "undefined") return null;
  if (window[BRIDGE_KEY]?.invoke) return window[BRIDGE_KEY];
  if (window[LEGACY_BRIDGE_KEY]?.invoke) {
    window[BRIDGE_KEY] = window[LEGACY_BRIDGE_KEY];
    return window[BRIDGE_KEY];
  }
  const hasTauriIpc = typeof window.__TAURI_IPC__ === "function";
  if (window.__TAURI__?.invoke || hasTauriIpc) {
    return {
      invoke: window.__TAURI__?.invoke ?? tauriInvoke,
      windowMinimize: async () => {},
      windowClose: async () => {},
      clipboardWriteText: async (text) => {
        if (navigator?.clipboard?.writeText) {
          await navigator.clipboard.writeText(String(text ?? ""));
        }
      },
      openExternal: async (url) => {
        if (typeof url === "string" && url) window.open(url, "_blank");
      },
      openDialog: async () => null,
      confirmDialog: async (message) => window.confirm(message ?? ""),
      setLauncherPreference: async (prefs) =>
        (window.__TAURI__?.invoke ?? tauriInvoke)("set_launcher_pref", {
          prefs,
        }),
      setUiPreference: async (name, value) =>
        (window.__TAURI__?.invoke ?? tauriInvoke)("set_ui_pref", { name, value }),
      releaseUiResources: async () => {},
      suspendUiWindow: async () => {},
      resumeUiWindow: async () => {},
      onEvent: () => () => {},
    };
  }
  if (!window[BRIDGE_KEY]) {
    window[BRIDGE_KEY] = {
      invoke: localInvoke,
      windowMinimize: async () => {},
      windowClose: async () => {},
      clipboardWriteText: async (text) => {
        if (navigator?.clipboard?.writeText) {
          await navigator.clipboard.writeText(String(text ?? ""));
        }
      },
      openExternal: async (url) => {
        if (typeof url === "string" && url) window.open(url, "_blank");
      },
      openDialog: async () => null,
      confirmDialog: async (message) => window.confirm(message ?? ""),
      setLauncherPreference: async (prefs) => {
        await localInvoke("set_launcher_pref", { prefs });
      },
      setUiPreference: async (name, value) => {
        await localInvoke("set_ui_pref", { name, value });
      },
      releaseUiResources: async () => {},
      suspendUiWindow: async () => {},
      resumeUiWindow: async () => {},
      onEvent: (cb) => {
        localEventListeners.add(cb);
        return () => localEventListeners.delete(cb);
      },
    };
  }
  if (!window[LEGACY_BRIDGE_KEY]) {
    window[LEGACY_BRIDGE_KEY] = window[BRIDGE_KEY];
  }
  return window[BRIDGE_KEY];
}

export function invoke(cmd, args = {}) {
  const bridge = getBridge();
  if (bridge?.invoke) return bridge.invoke(cmd, sanitizeForBridge(args));
  return Promise.reject(new Error("Backend bridge not available."));
}

export function windowMinimize() {
  return getBridge()?.windowMinimize?.();
}
export function windowClose() {
  return getBridge()?.windowClose?.();
}

export function clipboardWriteText(text) {
  return getBridge()?.clipboardWriteText?.(sanitizeForBridge(text));
}

export function openExternal(url) {
  return getBridge()?.openExternal?.(sanitizeForBridge(url));
}

export function openDialog(options) {
  return getBridge()?.openDialog?.(sanitizeForBridge(options));
}

export function onBackendEvent(cb) {
  return getBridge()?.onEvent?.(cb);
}

export function setLauncherPreference(prefs) {
  return getBridge()?.setLauncherPreference?.(sanitizeForBridge(prefs));
}

export function setUiPreference(name, value) {
  const bridge = getBridge();
  if (bridge?.setUiPreference) {
    return bridge.setUiPreference(
      sanitizeForBridge(name),
      sanitizeForBridge(value)
    );
  }
  if (bridge?.invoke) {
    return bridge.invoke("set_ui_pref", {
      name: sanitizeForBridge(name),
      value: sanitizeForBridge(value),
    });
  }
  return undefined;
}

export function releaseUiResources() {
  return getBridge()?.releaseUiResources?.();
}

export function suspendUiWindow() {
  return getBridge()?.suspendUiWindow?.();
}

export function resumeUiWindow() {
  return getBridge()?.resumeUiWindow?.();
}

export function confirmDialog(message, options) {
  return getBridge()?.confirmDialog?.(
    sanitizeForBridge(message),
    sanitizeForBridge(options)
  );
}

