import { writeText } from "@tauri-apps/api/clipboard";

export const CLASSIC_STYLE = 1;
export const PS4_STYLE = 3;

export const LAUNCHER_WINDOW_DEFAULTS = Object.freeze({
  [CLASSIC_STYLE]: Object.freeze({
    width: 1124,
    height: 600,
    resizable: true,
    aspectNumerator: 281,
    aspectDenominator: 150,
  }),
  [PS4_STYLE]: Object.freeze({
    width: 1280,
    height: 720,
    resizable: true,
    aspectNumerator: 16,
    aspectDenominator: 9,
  }),
});

const LAUNCHER_WINDOW_PREF_KEYS = Object.freeze({
  [CLASSIC_STYLE]: Object.freeze({
    width: "classicLauncherWidth",
    height: "classicLauncherHeight",
  }),
  [PS4_STYLE]: Object.freeze({
    width: "ps4LauncherWidth",
    height: "ps4LauncherHeight",
  }),
});

const LAUNCHER_WINDOW_RECENT_PREF_KEYS = Object.freeze({
  [CLASSIC_STYLE]: "classicLauncherRecentResolutions",
  [PS4_STYLE]: "ps4LauncherRecentResolutions",
});

const LAUNCHER_WINDOW_CUSTOM_MODE_PREF_KEYS = Object.freeze({
  [CLASSIC_STYLE]: "classicLauncherCustomResolution",
  [PS4_STYLE]: "ps4LauncherCustomResolution",
});

export function getLauncherWindowDefaults(style) {
  return LAUNCHER_WINDOW_DEFAULTS[style] ?? LAUNCHER_WINDOW_DEFAULTS[CLASSIC_STYLE];
}

export function getLauncherResolutionPrefKeys(style) {
  return LAUNCHER_WINDOW_PREF_KEYS[style] ?? null;
}

export function getLauncherRecentResolutionPrefKey(style) {
  return LAUNCHER_WINDOW_RECENT_PREF_KEYS[style] ?? null;
}

export function getLauncherCustomResolutionPrefKey(style) {
  return LAUNCHER_WINDOW_CUSTOM_MODE_PREF_KEYS[style] ?? null;
}

export function isLauncherWindowResizable(style) {
  return Boolean(getLauncherWindowDefaults(style)?.resizable);
}

export function getLauncherWindowAspect(style) {
  const config = getLauncherWindowDefaults(style);
  if (!config?.resizable || !config?.aspectNumerator || !config?.aspectDenominator) {
    return null;
  }
  return {
    numerator: config.aspectNumerator,
    denominator: config.aspectDenominator,
  };
}

export function formatResolutionValue(width, height) {
  return `${Number(width)}x${Number(height)}`;
}

export function parseResolutionValue(value) {
  const [widthText, heightText] = String(value ?? "").split("x");
  const width = Number.parseInt(widthText, 10);
  const height = Number.parseInt(heightText, 10);
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
    return null;
  }
  return {
    width,
    height,
    value: formatResolutionValue(width, height),
    label: formatResolutionValue(width, height),
  };
}

export const LOGIN_PAGE = 0;
export const PATCHER_PAGE = 1;
export const CHARACTERS_PAGE = 2;
export const SETTINGS_PAGE = 3;

export const DELETE_DIALOG = 0;
export const SERVERS_DIALOG = 1;
export const PATCHER_DIALOG = 2;
export const SERVER_SWITCH_DIALOG = 3;
export const VERSION_SWITCH_DIALOG = 4;
export const EXTERNAL_LINK_DIALOG = 5;
export const RESET_PATCH_DIALOG = 6;
export const RESET_PATCH_SUCCESS_DIALOG = 7;
export const BAN_DIALOG = 8;
export const LINUX_PREFIX_DIALOG = 9;

export const CHECKING_PATCHER = 0;
export const DOWNLOADING_PATCHER = 1;
export const RESTORING_PATCHER = 2;
export const PATCHING_PATCHER = 3;
export const DONE_PATCHER = 4;
export const ERROR_PATCHER = 5;

export const DEFAULT_SERVERLIST_URL =
  "NOT USED UNLESS SPECIFIED BY ADMIN=SERVERIP/serverlist.json";
export const DEFAULT_MESSAGELIST_URL =
  "NOT USED UNLESS SPECIFIED BY ADMIN=SERVERIP/messagelist.json";

export const GAME_VERSIONS = ["S6", "S7K", "F4", "F5", "G1", "G2", "G3", "G3.1", "G3.2", "GG", "G5", "G5.1", "G5.2", "G6", "G7", "G9.1", "G10.1", "Z1", "Z2", "Z2T", "ZZ"];

export async function requestHandler(cb, error, loading) {
  if (loading) loading.value = true;
  error.value = "";
  try {
    let result = await cb();
    if (loading) loading.value = false;
    return result;
  } catch (e) {
    if (e === "") return;
    error.value = e;
    if (loading) loading.value = false;
    throw e;
  }
}

export function formatDate(ts) {
  let d = new Date(ts * 1000);
  return d.toISOString().slice(0, 10);
}

export function openPicker(picker) {
  if (picker.value) return;
  picker.value = true;
  function closePicker() {
    picker.value = false;
    document.removeEventListener("click", closePicker);
  }
  setTimeout(() => {
    document.addEventListener("click", closePicker), 0;
  });
}

export function closeDropdown(cb) {
  document.activeElement.blur();
  cb();
}

export function forceRepaint(el) {
  if (!el) return;
  const prevTransform = el.style.transform;
  el.style.transform = "translateZ(0)";
  requestAnimationFrame(() => {
    el.style.transform = prevTransform;
  });
}

const cidChars = [
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "J",
  "K",
  "L",
  "M",
  "N",
  "P",
  "Q",
  "R",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
];

export function getCid(id) {
  let cid = [];
  for (let i = 5; i >= 0; i--) {
    const x = 32 ** i;
    cid.push(cidChars[Math.floor(id / x)]);
    id = id % x;
  }
  cid.reverse();
  return cid.join("");
}

export function copyCid(id) {
  const cid = getCid(id);
  writeText(cid).catch((e) => console.log("ERROR", e));
}








