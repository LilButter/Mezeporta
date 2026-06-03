import { invoke } from "@tauri-apps/api";

let itemMeta = null;
let equipNames = null;
let weaponClass = null;
let lookupLoadPromise = null;

function normalizeLookupId(value) {
  const raw = String(value ?? "").trim();
  if (!raw) return "";
  const hex = raw.replace(/^0x/i, "").toUpperCase();
  return /^[0-9A-F]+$/.test(hex) ? hex.padStart(4, "0") : hex;
}

function applyItemNameOverrides(overrides) {
  if (!overrides || typeof overrides !== "object") return;
  for (const [rawId, override] of Object.entries(overrides)) {
    const id = normalizeLookupId(rawId);
    if (!id) continue;
    const existing = itemMeta[id] ?? { id, icon: "Dummy", color: "" };
    if (typeof override === "string") {
      itemMeta[id] = { ...existing, id, name: override };
    } else if (override && typeof override === "object") {
      itemMeta[id] = { ...existing, id, ...override };
    }
  }
}

function applyEquipmentNameOverrides(overrides) {
  if (!overrides || typeof overrides !== "object") return;
  for (const [slotKey, slotOverrides] of Object.entries(overrides)) {
    if (!slotOverrides || typeof slotOverrides !== "object") continue;
    equipNames[slotKey] = { ...(equipNames[slotKey] ?? {}) };
    for (const [rawId, override] of Object.entries(slotOverrides)) {
      const id = normalizeLookupId(rawId);
      if (!id) continue;
      equipNames[slotKey][id] = typeof override === "string" ? override : override?.name;
    }
  }
}

function applyLookupOverrides(overrides) {
  applyItemNameOverrides(overrides?.items);
  applyEquipmentNameOverrides(overrides?.equipment);
}

async function ensureLookupData() {
  if (itemMeta && equipNames && weaponClass) return;
  if (!lookupLoadPromise) {
    lookupLoadPromise = Promise.all([
      import("./lookup/altclient-lookup.json"),
      import("./lookup/name-overrides.json").catch(() => ({ default: {} })),
    ]).then(([lookup, overrides]) => {
      const data = lookup.default ?? {};
      itemMeta = { ...(data.itemMeta ?? {}) };
      equipNames = Object.fromEntries(
        Object.entries(data.equipNames ?? {}).map(([key, names]) => [key, { ...names }])
      );
      weaponClass = data.weaponClass ?? {};
      applyLookupOverrides(overrides.default ?? {});
    });
  }
  await lookupLoadPromise;
}

const DEFAULT_LAYOUT = {
  zenny: 0xB0,
  gzenny: 0x1FF64,
  cp: 0x212E4,
  equipBox: 0x120,
  itemBox: 0x11A60,
  currentEquip: 0x1F604,
  itemBoxSlots: 4000,
  allowEquipmentBoxDecorationFallback: false,
};

const G_SERIES_LAYOUT = {
  zenny: 0xB0,
  gzenny: 0x172C4,
  cp: 0x18644,
  equipBox: 0x120,
  itemBox: 0xBCA0,
  currentEquip: 0x16964,
  itemBoxSlots: 4000,
  allowEquipmentBoxDecorationFallback: false,
};
const F5_SAVEDATA_LAYOUT = {
  zenny: 0xB0,
  gzenny: 0x0,
  cp: 0x0,
  equipBox: 0x110,
  itemBox: 0x7E10,
  currentEquip: 0xEC54,
  itemBoxSlots: 3510,
  allowEquipmentBoxDecorationFallback: false,
};

const G1_SAVEDATA_LAYOUT = G_SERIES_LAYOUT;
const ZZ_SAVEDATA_LAYOUT = DEFAULT_LAYOUT;

const LAYOUT_BY_VERSION = {
  S6: F5_SAVEDATA_LAYOUT,
  S7K: F5_SAVEDATA_LAYOUT,
  F4: F5_SAVEDATA_LAYOUT,
  F5: F5_SAVEDATA_LAYOUT,
  G1: G1_SAVEDATA_LAYOUT,
  G2: G1_SAVEDATA_LAYOUT,
  "G3.1": G1_SAVEDATA_LAYOUT,
  "G3.2": G1_SAVEDATA_LAYOUT,
  GG: G1_SAVEDATA_LAYOUT,
  "G5.2": G1_SAVEDATA_LAYOUT,
  G6: ZZ_SAVEDATA_LAYOUT,
  G7: ZZ_SAVEDATA_LAYOUT,
  "G9.1": ZZ_SAVEDATA_LAYOUT,
  "G10.1": ZZ_SAVEDATA_LAYOUT,
  Z1: ZZ_SAVEDATA_LAYOUT,
  Z2T: ZZ_SAVEDATA_LAYOUT,
  ZZ: ZZ_SAVEDATA_LAYOUT,
};

const EQUIP_READ_ORDER = [0, 2, 3, 4, 5, 1];
const EQUIP_TYPE_INFO = {
  "00": { slot: "Legs", key: "legs", icon: "/extra/equip/armor/Legs.png" },
  "02": { slot: "Head", key: "head", icon: "/extra/equip/armor/Head.png" },
  "03": { slot: "Chest", key: "chest", icon: "/extra/equip/armor/Chest.png" },
  "04": { slot: "Arms", key: "arms", icon: "/extra/equip/armor/Arms.png" },
  "05": { slot: "Waist", key: "waist", icon: "/extra/equip/armor/Waist.png" },
  "06": { slot: "Weapon", key: "melee", icon: "/extra/equip/weapon/GS.png" },
  "07": { slot: "Weapon", key: "ranged", icon: "/extra/equip/weapon/Bow.png" },
};

const WEAPON_CLASS_ICON = {
  GS: "/extra/equip/weapon/GS.png",
  LS: "/extra/equip/weapon/LS.png",
  Hammer: "/extra/equip/weapon/Hammer.png",
  Lance: "/extra/equip/weapon/Lance.png",
  SnS: "/extra/equip/weapon/SnS.png",
  DS: "/extra/equip/weapon/DS.png",
  HH: "/extra/equip/weapon/HH.png",
  Gunlance: "/extra/equip/weapon/Gunlance.png",
  Bow: "/extra/equip/weapon/Bow.png",
  HBG: "/extra/equip/weapon/HBG.png",
  LBG: "/extra/equip/weapon/LBG.png",
  Tonfa: "/extra/equip/weapon/Tonfa.png",
  Swaxe: "/extra/equip/weapon/Swaxe.png",
  Magspike: "/extra/equip/weapon/Magspike.png",
};

const parsedSavedataCache = new Map();

function normalizeCharacterId(characterId) {
  const numeric = Number(characterId);
  if (!Number.isFinite(numeric) || numeric <= 0) return null;
  return Math.trunc(numeric);
}

function endpointCacheKey(endpoint) {
  if (!endpoint) return "";
  return [
    endpoint.name ?? "",
    endpoint.url ?? "",
    endpoint.launcherPort ?? "",
    endpoint.gamePort ?? "",
    endpoint.version ?? "",
  ].join("|");
}

function readU32LE(bytes, offset) {
  if (!Number.isFinite(offset) || offset < 0 || offset + 3 >= bytes.length) return 0;
  return (
    bytes[offset] |
    (bytes[offset + 1] << 8) |
    (bytes[offset + 2] << 16) |
    (bytes[offset + 3] << 24)
  ) >>> 0;
}

function readU16LE(bytes, offset) {
  if (!Number.isFinite(offset) || offset < 0 || offset + 1 >= bytes.length) return 0;
  return bytes[offset] | (bytes[offset + 1] << 8);
}

function hexByte(value) {
  return Number(value).toString(16).toUpperCase().padStart(2, "0");
}

function readIdHex(bytes, offset) {
  if (!Number.isFinite(offset) || offset < 0 || offset + 1 >= bytes.length) return "0000";
  return `${hexByte(bytes[offset])}${hexByte(bytes[offset + 1])}`;
}

function mapVersionToken(value) {
  const token = String(value ?? "")
    .replace(/\s*\(debug\s+only\)\s*$/i, "")
    .trim()
    .toUpperCase();
  if (!token) return "";

  if (token === "S6" || token === "SEASON6" || token === "SEASON6.0") {
    return "S6";
  }
  if (token === "S7K" || token === "S7" || token === "SEASON7" || token === "SEASON7.0") {
    return "S7K";
  }

  if (token === "F4" || token === "FW4" || token === "FW.4" || token === "FW_4") {
    return "F4";
  }
  if (token === "F5" || token === "FW5" || token === "FW.5" || token === "FW_5") {
    return "F5";
  }
  if (token.startsWith("FW") || /^F\d/.test(token)) {
    const match = token.match(/(\d+(?:\.\d+)?)/);
    const parsed = Number(match?.[1]);
    if (!Number.isFinite(parsed)) return "F5";
    if (parsed >= 5) return "F5";
    if (parsed >= 4) return "F4";
  }

  if (token === "GG" || token === "G4") return "GG";

  if (token.startsWith("G")) {
    const match = token.match(/^G(\d+)(?:\.(\d+))?$/);
    if (match) {
      const major = Number(match[1] ?? "0");
      const minor = Number(match[2] ?? "0");
      if (major === 1) return "G1";
      if (major === 2) return "G2";
      if (major === 3) return minor >= 2 ? "G3.2" : "G3.1";
      if (major === 4) return "GG";
      if (major === 5) return "G5.2";
      if (major === 6) return "G6";
      if (major === 7) return "G7";
      if (major === 9) return "G9.1";
      if (major === 10) return "G10.1";
      if (Number.isFinite(major) && major <= 10) return "G1";
      return "ZZ";
    }
  }

  if (token === "ZZ") return "ZZ";
  if (token === "Z2" || token === "Z2.2" || token === "Z2T" || token === "Z2TW") return "Z2T";
  if (token.startsWith("Z")) {
    const match = token.match(/^Z(\d+)(?:\.(\d+))?$/);
    if (match) {
      const major = Number(match[1] ?? "0");
      if (Number.isFinite(major) && major >= 3) return "ZZ";
      if (Number.isFinite(major) && major >= 1) return "Z1";
    }
    return "ZZ";
  }

  return "";
}
function normalizeVersion(savedataVersion) {
  return mapVersionToken(savedataVersion) || "ZZ";
}

function resolveLayoutVersionToken(version) {
  if (["S6", "S7K", "F4", "F5"].includes(version)) return "F5";
  if (["G1", "G2", "G3.1", "G3.2", "GG", "G5.2"].includes(version)) return "G1";
  if (["G6", "G7", "G9.1", "G10.1", "Z1", "Z2T", "ZZ"].includes(version)) return "ZZ";
  return "ZZ";
}

function decodeBase64ToBytes(encoded) {
  const binary = atob(String(encoded ?? ""));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
function isCmpCompressedSavedata(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.length < 16) return false;
  return (
    bytes[0] === 0x63 && // c
    bytes[1] === 0x6d && // m
    bytes[2] === 0x70 && // p
    bytes[3] === 0x20 && // space
    bytes[4] === 0x32 && // 2
    bytes[5] === 0x30 && // 0
    bytes[6] === 0x31 && // 1
    bytes[7] === 0x31 && // 1
    bytes[8] === 0x30 && // 0
    bytes[9] === 0x31 && // 1
    bytes[10] === 0x31 && // 1
    bytes[11] === 0x33 && // 3
    bytes[12] === 0x20 && // space
    bytes[13] === 0x20 && // space
    bytes[14] === 0x20 && // space
    bytes[15] === 0x00
  );
}

function decompressCmpSavedata(bytes) {
  if (!isCmpCompressedSavedata(bytes)) return bytes;

  const out = [];
  for (let i = 16; i < bytes.length; i += 1) {
    const value = bytes[i];
    if (value !== 0x00) {
      out.push(value);
      continue;
    }

    if (i + 1 >= bytes.length) {
      break;
    }

    const nullCount = bytes[i + 1];
    i += 1;

    for (let j = 0; j < nullCount; j += 1) {
      out.push(0x00);
    }
  }

  return Uint8Array.from(out);
}

function resolveItemMeta(itemId) {
  const key = String(itemId ?? "0000").toUpperCase();
  const meta = itemMeta?.[key];
  if (meta) return meta;
  return {
    id: key,
    name: `Unknown ${key}`,
    icon: "Dummy",
    color: "",
  };
}

const ITEM_ICON_ALIASES = {
  Barrel10: "Barrel0",
  Binoculars0: "Dummy",
  Binoculars1: "Dummy",
  Body: "Body0",
  Body11: "Body0",
  Box1: "Box0",
  Cactus1: "Dummy",
  "Cat-Jewel1": "Dummy",
  Claw11: "Claw",
  Drink9: "Drink",
  Dung1: "Dung10",
  Egg11: "Egg0",
  Egg7: "Egg0",
  "Empty-Shell1": "Empty-Shell",
  Fruit1: "Dummy",
  Green1: "Dummy",
  Green2: "Dummy",
  Grey1: "Dummy",
  Grey10: "Dummy",
  HH1: "Dummy",
  House1: "Dummy",
  "Light-Blue1": "Dummy",
  Map0: "Dummy",
  Map1: "Dummy",
  Music1: "Dummy",
  "Pet-Food1": "Dummy",
  Pickaxe1: "Pickaxe0",
  Pink1: "Dummy",
  Question3: "Question",
  Root1: "Dummy",
  Sphere7: "Sphere",
  "Sword-Crystal1": "Dummy",
  "Tower-Sigil1": "Dummy",
  Tower1: "Dummy",
  Trap1: "Trap",
  Whetstone1: "Whetstone0",
  White0: "Dummy",
  White1: "Dummy",
  Yellow1: "Dummy",
};

function itemIconPath(meta) {
  const icon = String(meta?.icon ?? "Dummy").trim() || "Dummy";
  const color = String(meta?.color ?? "").trim();
  const key = `${icon}${color}`;
  return `/extra/items/${ITEM_ICON_ALIASES[key] ?? key}.png`;
}

export async function getItemDisplayMeta(itemId) {
  await ensureLookupData();
  const meta = resolveItemMeta(itemId);
  return {
    id: meta.id,
    name: meta.name,
    icon: itemIconPath(meta),
  };
}

function resolveEquipNameByKey(key, equipId) {
  const names = equipNames?.[key] ?? {};
  return names[equipId] ?? `Unknown ${equipId}`;
}

function resolveEquipName(typeCode, equipId) {
  const info = EQUIP_TYPE_INFO[typeCode];
  if (!info) return `Unknown ${equipId}`;
  return resolveEquipNameByKey(info.key, equipId);
}

function resolveWeaponClass(typeCode, equipId) {
  if (typeCode === "06") {
    return weaponClass?.melee?.[equipId] ?? "GS";
  }
  if (typeCode === "07") {
    return weaponClass?.ranged?.[equipId] ?? "Bow";
  }
  return "";
}

function resolveEquipIcon(typeCode, equipId) {
  const info = EQUIP_TYPE_INFO[typeCode];
  if (!info) return "/extra/equip/armor/Head.png";

  if (typeCode === "06" || typeCode === "07") {
    const weaponType = resolveWeaponClass(typeCode, equipId);
    return WEAPON_CLASS_ICON[weaponType] ?? info.icon;
  }

  return info.icon;
}

function parseCurrencies(bytes, layout) {
  const caravanPoints = layout.cp > 0 ? readU32LE(bytes, layout.cp) : 0;
  return {
    zenny: readU32LE(bytes, layout.zenny),
    gzenny: layout.gzenny > 0 ? readU32LE(bytes, layout.gzenny) : 0,
    caravanPoints,
  };
}

const KNOWN_EQUIP_TYPES = new Set(["00", "02", "03", "04", "05", "06", "07"]);
const WEAPON_EQUIP_TYPES = new Set(["06", "07"]);
const ARMOR_EQUIP_TYPES = new Set(["00", "02", "03", "04", "05"]);

function parseEquipEntryAtBase(bytes, base, order = 0) {
  if (base < 0 || base + 15 >= bytes.length) return null;

  const typeCode = hexByte(bytes[base + 5]);
  const equipId = readIdHex(bytes, base + 6);
  const upgradeLevel = readU16LE(bytes, base + 8);

  const decorations = [];
  for (let slot = 0; slot < 3; slot += 1) {
    const decoId = readIdHex(bytes, base + 10 + slot * 2);
    if (decoId === "0000") continue;
    const meta = resolveItemMeta(decoId);
    decorations.push({
      id: decoId,
      name: meta.name,
      icon: "/extra/items/Jewel0.png",
    });
  }

  const info = EQUIP_TYPE_INFO[typeCode] ?? null;
  const weaponType = resolveWeaponClass(typeCode, equipId);
  const isWeapon = WEAPON_EQUIP_TYPES.has(typeCode);
  const name = equipId === "0000"
    ? "No Equipment"
    : (() => {
        if (isWeapon) return resolveEquipName(typeCode, equipId);
        if (info?.key) return resolveEquipNameByKey(info.key, equipId);
        return resolveEquipName(typeCode, equipId);
      })();
  const icon = isWeapon
    ? resolveEquipIcon(typeCode, equipId)
    : (info?.icon ?? resolveEquipIcon(typeCode, equipId));

  return {
    order,
    slotLabel: info?.slot ?? "Unknown",
    typeCode,
    id: equipId,
    name,
    icon,
    upgradeLevel,
    weaponType: weaponType || null,
    decorations,
  };
}

function parseEquipEntriesAtOffset(bytes, offset) {
  const entries = [];
  for (let index = 0; index < 6; index += 1) {
    const entry = parseEquipEntryAtBase(bytes, offset + index * 16, index);
    if (!entry) break;
    entries.push(entry);
  }
  return entries;
}

function parseEquipmentBox(bytes, layout) {
  const start = Number(layout?.equipBox ?? 0);
  const end = Number(layout?.itemBox ?? bytes.length);
  if (!Number.isFinite(start) || start <= 0 || start >= bytes.length) return [];

  const maxEnd =
    Number.isFinite(end) && end > start ? Math.min(end, bytes.length) : bytes.length;
  const entries = [];
  for (let base = start; base + 15 < maxEnd; base += 16) {
    const entry = parseEquipEntryAtBase(bytes, base, entries.length);
    if (!entry || entry.id === "0000") break;
    entries.push(entry);
  }
  return entries;
}

function equipDecorationsEqual(aDecorations, bDecorations) {
  const left = Array.isArray(aDecorations) ? aDecorations : [];
  const right = Array.isArray(bDecorations) ? bDecorations : [];
  if (left.length !== right.length) return false;
  return left.every((entry, index) => entry?.id === right[index]?.id);
}

function findMatchingEquipBoxEntry(entry, equipmentBoxEntries) {
  if (!entry || !Array.isArray(equipmentBoxEntries) || !equipmentBoxEntries.length) {
    return null;
  }
  const candidates = equipmentBoxEntries
    .filter((candidate) => candidate?.id === entry.id && candidate?.typeCode === entry.typeCode)
    .filter((candidate) => Array.isArray(candidate.decorations) && candidate.decorations.length);
  if (!candidates.length) return null;

  const sameLevel = candidates.find((candidate) =>
    candidate.upgradeLevel === entry.upgradeLevel
  );
  const sameLevelCandidates = candidates.filter((candidate) =>
    candidate.upgradeLevel === entry.upgradeLevel
  );
  if (sameLevelCandidates.length === 1) return sameLevel;

  return candidates.length === 1 ? candidates[0] : null;
}

function mergeCurrentEquipDecorations(entries, equipmentBoxEntries, allowFallback = false) {
  if (!Array.isArray(entries) || !entries.length) return [];
  if (!allowFallback) return entries;
  return entries.map((entry) => {
    if (!entry || (Array.isArray(entry.decorations) && entry.decorations.length)) return entry;
    const matchingBoxEntry = findMatchingEquipBoxEntry(entry, equipmentBoxEntries);
    if (
      !matchingBoxEntry ||
      !Array.isArray(matchingBoxEntry.decorations) ||
      !matchingBoxEntry.decorations.length ||
      equipDecorationsEqual(entry.decorations, matchingBoxEntry.decorations)
    ) {
      return entry;
    }
    return {
      ...entry,
      decorations: matchingBoxEntry.decorations,
    };
  });
}

function scoreEquipCandidate(entries) {
  if (!Array.isArray(entries) || entries.length !== 6) return Number.NEGATIVE_INFINITY;

  let score = 0;
  let weaponCount = 0;
  let armorCount = 0;
  let filledArmorCount = 0;
  let waistFilled = false;

  for (const entry of entries) {
    if (!entry) return Number.NEGATIVE_INFINITY;
    const typeCode = String(entry.typeCode ?? "");
    const hasItem = String(entry.id ?? "0000") !== "0000";

    if (KNOWN_EQUIP_TYPES.has(typeCode)) {
      score += 2;
    } else {
      score -= 5;
      continue;
    }

    if (WEAPON_EQUIP_TYPES.has(typeCode)) {
      weaponCount += 1;
      score += hasItem ? 6 : -3;
    } else if (ARMOR_EQUIP_TYPES.has(typeCode)) {
      armorCount += 1;
      if (hasItem) filledArmorCount += 1;
      if (typeCode === "05" && hasItem) waistFilled = true;
    }
  }

  score += weaponCount === 1 ? 10 : -Math.abs(weaponCount - 1) * 6;
  if (armorCount >= 4) score += 4;
  if (filledArmorCount >= 3) score += 4;
  if (waistFilled) score += 4;

  return score;
}

function resolveCurrentEquipOffset(bytes, layout, versionToken) {
  const fallback = Number(layout?.currentEquip ?? 0);
  if (!Number.isFinite(fallback) || fallback < 0) return 0;
  if (versionToken !== "F5") return fallback;

  const maxStart = bytes.length - 96;
  if (maxStart < 0) return fallback;
  const base = Math.max(0, Math.min(fallback, maxStart));

  let bestOffset = base;
  let bestScore = scoreEquipCandidate(parseEquipEntriesAtOffset(bytes, base));
  const searchStart = Math.max(0, base - 0x500);
  const searchEnd = Math.min(maxStart, base + 0x500);

  for (let offset = searchStart; offset <= searchEnd; offset += 16) {
    if (offset === base) continue;
    const candidate = parseEquipEntriesAtOffset(bytes, offset);
    const distancePenalty = Math.abs(offset - base) / 64;
    const score = scoreEquipCandidate(candidate) - distancePenalty;
    if (score > bestScore) {
      bestScore = score;
      bestOffset = offset;
    }
  }

  return bestOffset;
}

function parseCurrentEquip(bytes, layout, versionToken, equipmentBoxEntries = []) {
  const equipOffset = resolveCurrentEquipOffset(bytes, layout, versionToken);
  const raw = parseEquipEntriesAtOffset(bytes, equipOffset);

  const ordered = EQUIP_READ_ORDER
    .map((entryIndex) => raw[entryIndex])
    .filter(Boolean);

  return mergeCurrentEquipDecorations(
    normalizeEquipEntries(ordered, versionToken),
    equipmentBoxEntries,
    Boolean(layout?.allowEquipmentBoxDecorationFallback)
  );
}

function toForcedSlotEntry(entry, slotLabel, key, icon) {
  if (!entry) return null;
  const next = { ...entry, slotLabel, icon };
  next.name = entry.id === "0000" ? "No Equipment" : resolveEquipNameByKey(key, entry.id);
  return next;
}

function normalizeEquipEntries(entries, versionToken) {
  if (!Array.isArray(entries) || !entries.length) return [];

  const bySlot = new Map();
  const legEntries = [];
  for (const entry of entries) {
    if (!entry) continue;
    if (entry.slotLabel === "Legs") {
      legEntries.push(entry);
    }
    if (!bySlot.has(entry.slotLabel)) {
      bySlot.set(entry.slotLabel, entry);
    }
  }

  // F4/F5 can report both legs + waist with the same armor type in some saves.
  if (versionToken === "F5" && !bySlot.has("Waist") && legEntries.length >= 2) {
    const emptyLeg = legEntries.find((entry) => entry.id === "0000");
    const filledLeg = legEntries.find((entry) => entry.id !== "0000");
    if (emptyLeg && filledLeg) {
      bySlot.set("Legs", filledLeg);
      bySlot.set(
        "Waist",
        toForcedSlotEntry(
          emptyLeg,
          "Waist",
          "waist",
          "/extra/equip/armor/Waist.png"
        )
      );
    } else {
      const waistSource = legEntries.find((entry) => entry.order === 0) ?? legEntries[0];
      bySlot.set(
        "Waist",
        toForcedSlotEntry(
          waistSource,
          "Waist",
          "waist",
          "/extra/equip/armor/Waist.png"
        )
      );
      if (waistSource?.id !== "0000") {
        const legsSource = legEntries.find((entry) => entry !== waistSource);
        if (legsSource) bySlot.set("Legs", legsSource);
      }
    }
  }

  const orderedSlots = ["Head", "Chest", "Arms", "Waist", "Legs", "Weapon"];
  return orderedSlots
    .map((slotLabel) => bySlot.get(slotLabel))
    .filter(Boolean);
}

function toDisplayItemSlot(slotIndex, versionToken) {
  if (versionToken === "F5") {
    // F4/F5 saves have two internal item-box entries before visible in-game slots.
    return Math.max(1, slotIndex - 1);
  }
  return slotIndex + 1;
}
function parseItemBox(bytes, layout, versionToken) {
  const configuredSlots = Number(layout?.itemBoxSlots ?? 4000);
  const availableSlots = Math.max(0, Math.floor((bytes.length - layout.itemBox) / 8));
  const maxSlots = Math.max(0, Math.min(configuredSlots, availableSlots));
  const items = [];

  for (let slot = 0; slot < maxSlots; slot += 1) {
    const base = layout.itemBox + slot * 8;
    if (base + 7 >= bytes.length) break;

    const itemId = readIdHex(bytes, base + 4);
    const quantity = readU16LE(bytes, base + 6);

    if (itemId === "0000" || quantity <= 0) continue;

    const meta = resolveItemMeta(itemId);
    items.push({
      slot: toDisplayItemSlot(slot, versionToken),
      id: itemId,
      name: meta.name,
      quantity,
      icon: itemIconPath(meta),
    });
  }

  return items;
}

function parseSavedataBytes(bytes, savedataVersion) {
  const normalizedVersion = normalizeVersion(savedataVersion);
  const layoutVersion = resolveLayoutVersionToken(normalizedVersion);
  const layout = LAYOUT_BY_VERSION[normalizedVersion] ?? DEFAULT_LAYOUT;
  const rawBytes = decompressCmpSavedata(bytes);
  const equipmentBoxEntries = layout.allowEquipmentBoxDecorationFallback
    ? parseEquipmentBox(rawBytes, layout)
    : [];

  return {
    version: normalizedVersion,
    currencies: parseCurrencies(rawBytes, layout),
    gear: parseCurrentEquip(rawBytes, layout, layoutVersion, equipmentBoxEntries),
    itemBox: parseItemBox(rawBytes, layout, layoutVersion),
  };
}

export function clearAltSavedataPanelCache() {
  parsedSavedataCache.clear();
}

export async function getAltCharacterSavedataView(
  characterId,
  endpoint,
  savedataVersion = null
) {
  const normalizedId = normalizeCharacterId(characterId);
  if (normalizedId === null || !endpoint) return null;

  const normalizedVersion = String(savedataVersion ?? "").trim();
  const cacheKey = `${endpointCacheKey(endpoint)}::${normalizedId}::${normalizedVersion}`;
  if (parsedSavedataCache.has(cacheKey)) {
    return parsedSavedataCache.get(cacheKey);
  }

  await ensureLookupData();
  try {
    const cachePayload = await invoke("read_alt_character_savedata_cache", {
      characterId: normalizedId,
      savedataVersion: normalizedVersion || null,
    });

    const encoded = String(cachePayload?.savedata ?? "").trim();
    const savedataVersion = String(cachePayload?.gsv ?? "").trim();

    if (!encoded || !savedataVersion) {
      return null;
    }

    const bytes = decodeBase64ToBytes(encoded);
    const parsed = parseSavedataBytes(bytes, savedataVersion);
    parsedSavedataCache.set(cacheKey, parsed);
    return parsed;
  } catch (_error) {
    return null;
  }
}



