const cache = new Map();
const manifest = new Map();
const pending = new Map();

const MANIFEST_STORAGE_KEY = "mezeporta.imageCacheManifest.v1";
const MANIFEST_MAX_ENTRIES = 400;
let manifestLoaded = false;
let manifestPersistTimeout = null;

function appendSignature(url, signature) {
  if (!url || !signature) return url;
  const encoded = encodeURIComponent(signature);
  const delimiter = url.includes("?") ? "&" : "?";
  return `${url}${delimiter}v=${encoded}`;
}

function canUseStorage() {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}

function loadManifest() {
  if (manifestLoaded) return;
  manifestLoaded = true;
  if (!canUseStorage()) return;
  try {
    const raw = window.localStorage.getItem(MANIFEST_STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw);
    const entries = Array.isArray(parsed?.entries) ? parsed.entries : [];
    for (const row of entries) {
      if (!Array.isArray(row) || row.length < 2) continue;
      const [url, signature, checkedAt] = row;
      if (typeof url !== "string" || typeof signature !== "string") continue;
      manifest.set(url, {
        signature,
        checkedAt: Number.isFinite(checkedAt) ? checkedAt : 0,
      });
    }
  } catch (_) {
    // ignore malformed manifest
  }
}

function persistManifest() {
  if (!canUseStorage()) return;
  const entries = Array.from(manifest.entries())
    .sort((a, b) => (b[1]?.checkedAt || 0) - (a[1]?.checkedAt || 0))
    .slice(0, MANIFEST_MAX_ENTRIES)
    .map(([url, entry]) => [url, entry.signature, entry.checkedAt || 0]);
  try {
    window.localStorage.setItem(
      MANIFEST_STORAGE_KEY,
      JSON.stringify({
        version: 1,
        entries,
      })
    );
  } catch (_) {
    // ignore storage quota errors
  }
}

function scheduleManifestPersist() {
  if (!canUseStorage()) return;
  if (manifestPersistTimeout) return;
  manifestPersistTimeout = window.setTimeout(() => {
    manifestPersistTimeout = null;
    persistManifest();
  }, 150);
}

function getEntry(url) {
  loadManifest();
  const inMemory = cache.get(url);
  if (inMemory) return inMemory;
  const fromManifest = manifest.get(url);
  if (fromManifest) {
    cache.set(url, fromManifest);
  }
  return fromManifest;
}

function updateEntry(url, signature) {
  const nextEntry = { signature, checkedAt: Date.now() };
  cache.set(url, nextEntry);
  manifest.set(url, nextEntry);
  scheduleManifestPersist();
}

function canFetchSignature(url) {
  if (typeof window === "undefined" || typeof window.location === "undefined") {
    return true;
  }
  try {
    const parsedUrl = new URL(url, window.location.href);
    if (parsedUrl.protocol === "data:" || parsedUrl.protocol === "blob:") {
      return false;
    }
    const currentUrl = new URL(window.location.href);
    const isClientImagesPath = parsedUrl.pathname
      .toLowerCase()
      .startsWith("/clientimages/");
    if (/^https?:$/i.test(parsedUrl.protocol) && isClientImagesPath) {
      return true;
    }
    return parsedUrl.origin === currentUrl.origin;
  } catch (_) {
    return true;
  }
}

async function fetchSignature(url) {
  if (!canFetchSignature(url)) return null;

  try {
    const response = await fetch(url, { method: "HEAD", cache: "no-store" });
    if (response.ok) {
      const etag = response.headers.get("etag");
      const lastModified = response.headers.get("last-modified");
      if (etag) return `etag:${etag}`;
      if (lastModified) return `last-modified:${lastModified}`;
    }
  } catch (_) {
    // fall back to hashing
  }

  try {
    const response = await fetch(url, { cache: "no-store" });
    if (!response.ok) return null;
    const buffer = await response.arrayBuffer();
    const digest = await crypto.subtle.digest("SHA-256", buffer);
    const hash = Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
    return `sha256:${hash}`;
  } catch (_) {
    return null;
  }
}

export function getCachedImageUrl(url) {
  if (!url) return null;
  const entry = getEntry(url);
  return appendSignature(url, entry?.signature);
}

export async function refreshCachedImageUrl(url) {
  if (!url) return null;
  if (pending.has(url)) return pending.get(url);

  const promise = (async () => {
    const signature = await fetchSignature(url);
    if (!signature) return getCachedImageUrl(url) || url;
    const entry = getEntry(url);
    if (!entry || entry.signature !== signature || !entry.checkedAt) {
      updateEntry(url, signature);
    }
    return appendSignature(url, signature);
  })();

  pending.set(url, promise);
  try {
    return await promise;
  } finally {
    pending.delete(url);
  }
}

export function applyCachedImage(url, setter) {
  if (!url) {
    setter(null);
    return;
  }
  const cached = getCachedImageUrl(url) || url;
  setter(cached);
  refreshCachedImageUrl(url).then((next) => {
    if (next && next !== cached) {
      setter(next);
    }
  });
}

export function clearImageCache(options = {}) {
  const clearManifest = Boolean(options?.clearManifest);
  cache.clear();
  pending.clear();
  if (clearManifest) {
    manifest.clear();
    persistManifest();
  }
}

loadManifest();
