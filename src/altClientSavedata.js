import { invoke } from "@tauri-apps/api";

const prefetchInFlight = new Map();
const versionCheckInFlight = new Map();

function normalizeCharacterId(characterId) {
  const numeric = Number(characterId);
  if (!Number.isFinite(numeric) || numeric <= 0) return null;
  return Math.trunc(numeric);
}

export async function hasAltCharacterSavedataVersion(characterId, savedataVersion = null) {
  const normalizedId = normalizeCharacterId(characterId);
  if (normalizedId === null) return false;

  const normalizedVersion = String(savedataVersion ?? "").trim();
  const key = `${normalizedId}::${normalizedVersion}`;
  if (versionCheckInFlight.has(key)) {
    return versionCheckInFlight.get(key);
  }

  const request = invoke("has_alt_character_savedata_version", {
    characterId: normalizedId,
    savedataVersion: normalizedVersion || null,
  })
    .then((value) => Boolean(value))
    .catch(() => false)
    .finally(() => {
      versionCheckInFlight.delete(key);
    });

  versionCheckInFlight.set(key, request);
  return request;
}

export async function prefetchAltCharacterSavedata(characterId, savedataVersion = null) {
  const normalizedId = normalizeCharacterId(characterId);
  if (normalizedId === null) return false;

  const normalizedVersion = String(savedataVersion ?? "").trim();
  const key = `${normalizedId}::${normalizedVersion}`;
  if (prefetchInFlight.has(key)) {
    return prefetchInFlight.get(key);
  }

  const request = invoke("prefetch_alt_character_savedata", {
    characterId: normalizedId,
    savedataVersion: normalizedVersion || null,
  })
    .then((value) => Boolean(value))
    .catch(() => false)
    .finally(() => {
      prefetchInFlight.delete(key);
    });

  prefetchInFlight.set(key, request);
  return request;
}
