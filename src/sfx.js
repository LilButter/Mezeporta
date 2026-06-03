import { invoke } from "@tauri-apps/api";
import { store } from "./store";

const FILE_BASENAMES = Object.freeze({
  hover: "hover",
  select: "select",
  confirm: "confirm",
  start: "start",
  login: "login",
});

const POOL_SIZE = 4;
const SOURCE_CACHE = Object.create(null);
const SOURCE_PROMISES = Object.create(null);
const AUDIO_POOLS = Object.create(null);
const LINUX_SFX_WARNED = new Set();

let sfxUnlocked = false;
let lastHoverTs = 0;
let runtimePlatform =
  typeof navigator !== "undefined" && /linux/i.test(`${navigator.platform} ${navigator.userAgent}`)
    ? "linux"
    : "unknown";

function currentVolume() {
  return (store.settings?.sfxVolume ?? 70) / 100;
}

function canUseHtmlAudio() {
  return typeof Audio !== "undefined";
}

function getProbeAudio() {
  if (!canUseHtmlAudio()) return null;
  return new Audio();
}

function getPreferredExtensions() {
  const fallback = runtimePlatform === "linux" ? ["ogg", "mp3"] : ["mp3", "ogg"];
  const probe = getProbeAudio();
  if (!probe?.canPlayType) return fallback;

  const scores = [
    {
      ext: "mp3",
      score:
        probe.canPlayType("audio/mpeg") === "probably"
          ? 2
          : probe.canPlayType("audio/mpeg") === "maybe"
          ? 1
          : 0,
    },
    {
      ext: "ogg",
      score:
        probe.canPlayType('audio/ogg; codecs="vorbis"') === "probably"
          ? 2
          : probe.canPlayType('audio/ogg; codecs="vorbis"') === "maybe"
          ? 1
          : 0,
    },
  ];

  const preferredTieBreaker = runtimePlatform === "linux" ? ["ogg", "mp3"] : ["mp3", "ogg"];
  scores.sort((left, right) => {
    if (right.score !== left.score) return right.score - left.score;
    return preferredTieBreaker.indexOf(left.ext) - preferredTieBreaker.indexOf(right.ext);
  });

  const supported = scores.filter((entry) => entry.score > 0).map((entry) => entry.ext);
  return supported.length ? supported : fallback;
}

function candidateAudioPaths(name) {
  const base = FILE_BASENAMES[name];
  if (!base) return [];
  return getPreferredExtensions().map((ext) => `/audio/${base}.${ext}`);
}

function markUnlocked() {
  sfxUnlocked = true;
}

function clearAudioCaches() {
  for (const key of Object.keys(SOURCE_CACHE)) {
    delete SOURCE_CACHE[key];
  }
  for (const key of Object.keys(SOURCE_PROMISES)) {
    delete SOURCE_PROMISES[key];
  }
  for (const key of Object.keys(AUDIO_POOLS)) {
    delete AUDIO_POOLS[key];
  }
}

function createAudioElement(url) {
  const audio = new Audio(url);
  audio.preload = "auto";
  audio.volume = currentVolume();
  return audio;
}

async function playLinuxUiSfx(name, { waitForEnd = false } = {}) {
  try {
    await invoke("play_linux_ui_sfx", {
      name,
      waitForEnd,
      volume: currentVolume(),
    });
    return true;
  } catch (error) {
    if (import.meta.env?.DEV && !LINUX_SFX_WARNED.has(name)) {
      LINUX_SFX_WARNED.add(name);
      console.warn(`Linux UI SFX "${name}" failed:`, error);
    }
    return false;
  }
}

function syncAudioVolume(audio) {
  audio.volume = currentVolume();
}

function probeAudioUrl(url) {
  return new Promise((resolve) => {
    if (!canUseHtmlAudio()) {
      resolve(false);
      return;
    }

    const audio = createAudioElement(url);
    let settled = false;
    const timeout = window.setTimeout(() => finish(false), 2500);

    function cleanup() {
      audio.oncanplaythrough = null;
      audio.onloadeddata = null;
      audio.onerror = null;
      window.clearTimeout(timeout);
    }

    function finish(ok) {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(ok);
    }

    audio.oncanplaythrough = () => finish(true);
    audio.onloadeddata = () => finish(true);
    audio.onerror = () => finish(false);

    try {
      audio.load();
    } catch (_error) {
      finish(false);
    }
  });
}

async function resolveAudioSource(name) {
  if (SOURCE_CACHE[name]) return SOURCE_CACHE[name];
  if (SOURCE_PROMISES[name]) return SOURCE_PROMISES[name];

  const promise = (async () => {
    for (const path of candidateAudioPaths(name)) {
      if (await probeAudioUrl(path)) {
        SOURCE_CACHE[name] = path;
        return path;
      }
    }
    return null;
  })();

  SOURCE_PROMISES[name] = promise;
  try {
    return await promise;
  } finally {
    delete SOURCE_PROMISES[name];
  }
}

async function ensureAudioPool(name) {
  if (AUDIO_POOLS[name]) return AUDIO_POOLS[name];
  const url = await resolveAudioSource(name);
  if (!url) return null;

  const audios = Array.from({ length: POOL_SIZE }, () => createAudioElement(url));
  AUDIO_POOLS[name] = {
    index: 0,
    url,
    audios,
  };
  return AUDIO_POOLS[name];
}

function nextAudioFromPool(pool) {
  const audio = pool.audios[pool.index];
  pool.index = (pool.index + 1) % pool.audios.length;
  return audio;
}

function resetAudioForPlayback(audio) {
  syncAudioVolume(audio);
  try {
    audio.pause();
  } catch (_error) {
    // ignore pause errors on fresh elements
  }
  try {
    audio.currentTime = 0;
  } catch (_error) {
    // ignore seek errors when metadata is not ready yet
  }
}

async function playAndWaitForEnd(audio) {
  try {
    await audio.play();
  } catch (_error) {
    return false;
  }

  return new Promise((resolve) => {
    let settled = false;

    function cleanup() {
      audio.onended = null;
      audio.onerror = null;
      audio.onpause = null;
    }

    function finish(ok) {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(ok);
    }

    audio.onended = () => finish(true);
    audio.onerror = () => finish(false);
    audio.onpause = () => {
      if (audio.ended) return;
      finish(false);
    };
  });
}

async function playHtmlAudioName(name, { requiresUnlock = false, waitForEnd = false } = {}) {
  if (!canUseHtmlAudio()) return false;
  if (requiresUnlock && !sfxUnlocked) return false;

  if (waitForEnd) {
    const url = await resolveAudioSource(name);
    if (!url) return false;
    const audio = createAudioElement(url);
    resetAudioForPlayback(audio);
    return playAndWaitForEnd(audio);
  }

  const pool = await ensureAudioPool(name);
  if (!pool) return false;
  const audio = nextAudioFromPool(pool);
  resetAudioForPlayback(audio);
  try {
    const playPromise = audio.play();
    if (playPromise && typeof playPromise.then === "function") {
      await playPromise;
    }
    return true;
  } catch (_error) {
    return false;
  }
}

async function playName(name, { requiresUnlock = false, waitForEnd = false } = {}) {
  if (!store.settings?.sfxEnabled) return false;
  if (runtimePlatform === "linux") {
    if (await playHtmlAudioName(name, { requiresUnlock, waitForEnd })) {
      return true;
    }
    const played = await playLinuxUiSfx(name, { waitForEnd });
    if (played) {
      return true;
    }
    return false;
  }
  return playHtmlAudioName(name, { requiresUnlock, waitForEnd });
}

if (typeof window !== "undefined") {
  const unlock = () => {
    markUnlocked();
    window.removeEventListener("pointerdown", unlock);
    window.removeEventListener("keydown", unlock);
    window.removeEventListener("touchstart", unlock);
    preloadSfx();
  };

  window.addEventListener("pointerdown", unlock, { once: true });
  window.addEventListener("keydown", unlock, { once: true });
  window.addEventListener("touchstart", unlock, { once: true });
}

export function setLinuxAudioRuntimeStatus(status = {}) {
  if (status?.platform === "linux") {
    runtimePlatform = "linux";
    return;
  }
  if (status?.platform) {
    runtimePlatform = "other";
  }
}

export function forceSfxUnlock() {
  markUnlocked();
  preloadSfx();
}

export function play(name, opts = { requiresUnlock: false }) {
  void playName(name, opts);
}

export async function playStartAndWait() {
  if (!store.settings?.sfxEnabled) return;
  await playName("start", { waitForEnd: true });
}

export const playHover = () => {
  if (!store.settings?.sfxEnabled) return;
  const now = performance.now();
  if (now < lastHoverTs) {
    lastHoverTs = 0;
  }
  if (now - lastHoverTs < 60) return;
  lastHoverTs = now;
  void playName("hover", { requiresUnlock: true });
};

export const playSelect = () => {
  void playName("select");
};

export const playConfirm = () => {
  void playName("confirm");
};

export const playStart = () => {
  void playName("start");
};

export const playLogin = () => {
  return playName("login");
};

export function preloadLoginSfx() {
  if (!store.settings?.sfxEnabled) return;
  if (runtimePlatform === "linux") return;
  if (!canUseHtmlAudio()) return;
  void ensureAudioPool("login");
}

export function preloadSfx() {
  if (!store.settings?.sfxEnabled) return;
  if (runtimePlatform === "linux") return;
  if (!canUseHtmlAudio()) return;
  for (const key of Object.keys(FILE_BASENAMES)) {
    void ensureAudioPool(key);
  }
}

export function bindSfx(el, opts = { hover: true, click: "select" }) {
  const onHover = opts.hover ? () => playHover() : null;
  const onClick = opts.click ? () => play(opts.click) : null;

  if (onHover) {
    el.addEventListener("pointerenter", onHover);
    el.addEventListener("mouseenter", onHover);
  }
  if (onClick) {
    el.addEventListener("click", onClick);
  }

  return () => {
    if (onHover) {
      el.removeEventListener("pointerenter", onHover);
      el.removeEventListener("mouseenter", onHover);
    }
    if (onClick) {
      el.removeEventListener("click", onClick);
    }
  };
}

export function resetSfxState() {
  clearAudioCaches();
  sfxUnlocked = false;
  lastHoverTs = 0;
}
