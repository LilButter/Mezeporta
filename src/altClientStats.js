import { invoke } from "@tauri-apps/api";

export const EMPTY_ALT_CLIENT_STATS = {
  mezFes: null,
  featuredWeapon: null,
  events: {
    festaActive: false,
    divaActive: false,
    tournamentActive: false,
    specialEvents: [],
  },
  enabledSpecialEvents: [],
  onlinePlayers: 0,
  user: {
    gachaPremium: 0,
    gachaTrial: 0,
    frontierPoints: 0,
  },
  characters: [],
  onlineFriends: [],
};

let devAltClientStatsOverride = null;

function cloneStats(value) {
  if (!value || typeof value !== "object") return {};
  return JSON.parse(JSON.stringify(value));
}

function applyDevAltClientStatsOverride(stats) {
  if (!import.meta.env.DEV || !devAltClientStatsOverride) return stats;
  const override = cloneStats(devAltClientStatsOverride);
  return {
    ...stats,
    ...override,
    events: {
      ...(stats?.events ?? {}),
      ...(override.events ?? {}),
    },
    user: {
      ...(stats?.user ?? {}),
      ...(override.user ?? {}),
    },
    characters: Array.isArray(override.characters)
      ? override.characters
      : Array.isArray(stats?.characters)
      ? stats.characters
      : [],
    onlineFriends: Array.isArray(override.onlineFriends)
      ? override.onlineFriends
      : Array.isArray(stats?.onlineFriends)
      ? stats.onlineFriends
      : [],
  };
}

export function setDevAltClientStatsOverride(value) {
  if (!import.meta.env.DEV) return;
  devAltClientStatsOverride = value && typeof value === "object" ? cloneStats(value) : null;
}

export async function getAltClientStats() {
  try {
    const stats = await invoke("get_alt_client_stats");
    return applyDevAltClientStatsOverride(stats ?? { ...EMPTY_ALT_CLIENT_STATS });
  } catch (_error) {
    return applyDevAltClientStatsOverride({ ...EMPTY_ALT_CLIENT_STATS });
  }
}

export async function getAltClientDistributions(characterId, offset = 0, limit = 6) {
  try {
    const page = await invoke("get_alt_client_distributions", {
      characterId: Number(characterId ?? 0),
      offset: Number(offset ?? 0),
      limit: Number(limit ?? 6),
    });
    return {
      characterId: Number(page?.characterId ?? characterId ?? 0),
      offset: Number(page?.offset ?? offset ?? 0),
      limit: Number(page?.limit ?? limit ?? 6),
      total: Number(page?.total ?? 0),
      entries: Array.isArray(page?.entries) ? page.entries : [],
    };
  } catch (_error) {
    return {
      characterId: Number(characterId ?? 0),
      offset: Number(offset ?? 0),
      limit: Number(limit ?? 6),
      total: 0,
      entries: [],
    };
  }
}
