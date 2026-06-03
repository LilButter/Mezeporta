import { assetUrl } from "./store";

const SPECIAL_EVENT_DEFS = [
  {
    id: "tenro",
    label: "Tenro",
    image: "/extra/Tenro.png",
    aliases: ["tenro", "tenrou", "tenrouirai", "tower"],
    flags: ["tenroActive", "tenrouActive", "towerActive"],
  },
  {
    id: "pallone-festival",
    label: "Pallone Festival",
    image: "/extra/PalloneFestival.png",
    aliases: ["pallone", "pallonefestival", "pallonefest", "pallone_festival", "caravan"],
    flags: ["palloneActive", "palloneFestivalActive", "caravanFestivalActive"],
  },
  {
    id: "mezfes",
    label: "MezFes",
    image: "/extra/Mezfes.png",
    aliases: ["mezfes", "mezeportafestival", "mezeporta_festival"],
    flags: ["mezFesActive", "mezfesActive"],
  },
  {
    id: "hunter-festa",
    label: "Hunter Festa",
    image: "/extra/HunterFesta.png",
    aliases: ["hunterfesta", "hunter_festa", "festa"],
    flags: ["festaActive", "hunterFestaActive"],
  },
  {
    id: "hunting-tournament",
    label: "Hunting Tournament",
    image: "/extra/HuntingTournament.png",
    aliases: ["vs", "tournament", "huntingtournament", "hunting_tournament"],
    flags: ["tournamentActive", "vsActive", "huntingTournamentActive"],
  },
  {
    id: "diva",
    label: "Diva",
    image: "/extra/Diva.png",
    aliases: ["diva", "divadefense", "ud"],
    flags: ["divaActive", "divaDefenseActive", "udActive"],
  },
  {
    id: "conquest",
    label: "Conquest",
    image: "/extra/Conquest.png",
    aliases: ["conquest", "conquestwar", "rengoku", "seibattle", "seibatsu"],
    flags: ["conquestActive", "rengokuActive", "seibattleActive", "seibatsuActive"],
  },
];

function normalizeEventName(value) {
  return String(value ?? "")
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "");
}

function eventList(serverInfo) {
  return [
    ...(Array.isArray(serverInfo?.enabledSpecialEvents)
      ? serverInfo.enabledSpecialEvents
      : []),
    ...(Array.isArray(serverInfo?.events?.specialEvents)
      ? serverInfo.events.specialEvents
      : []),
  ];
}

function normalizeEventEntry(entry) {
  if (entry && typeof entry === "object") {
    return [
      entry.id,
      entry.type,
      entry.eventType,
      entry.key,
      entry.name,
      entry.label,
    ]
      .map(normalizeEventName)
      .filter(Boolean);
  }
  return [normalizeEventName(entry)].filter(Boolean);
}

function eventListIncludes(list, aliases) {
  const normalizedList = list.flatMap(normalizeEventEntry);
  return aliases.some((alias) => {
    const normalizedAlias = normalizeEventName(alias);
    return normalizedList.some(
      (entry) => entry === normalizedAlias || entry.includes(normalizedAlias)
    );
  });
}

function eventFlagActive(serverInfo, flags = []) {
  return flags.some(
    (flag) => Boolean(serverInfo?.events?.[flag]) || Boolean(serverInfo?.[flag])
  );
}

function eventBadge({ id, label, image, rows }) {
  return {
    id,
    label,
    image: assetUrl(image),
    rows,
  };
}

function isMezFesActive(mezFes) {
  if (!mezFes) return false;
  const start = Number(mezFes.start ?? 0);
  const end = Number(mezFes.end ?? 0);
  if (!Number.isFinite(start) || !Number.isFinite(end) || start <= 0 || end <= start) {
    return false;
  }
  const now = Date.now() / 1000;
  return now >= start && now <= end;
}

export function buildActiveEventBadges(serverInfo, helpers = {}) {
  const formatUnixSeconds =
    typeof helpers.formatUnixSeconds === "function"
      ? helpers.formatUnixSeconds
      : () => "N/A";
  const formatPanelNumber =
    typeof helpers.formatPanelNumber === "function"
      ? helpers.formatPanelNumber
      : (value) => String(value ?? "0");

  const badges = [];
  const activeIds = new Set();
  const mezFes = serverInfo?.mezFes;
  if (isMezFesActive(mezFes)) {
    const stalls = Array.isArray(mezFes.stalls) ? mezFes.stalls : [];
    badges.push(
      eventBadge({
        id: "mezfes",
        label: "MezFes",
        image: "/extra/Mezfes.png",
        rows: [
          ["Status", `Active (ID ${mezFes.id ?? "N/A"})`],
          ["Start", formatUnixSeconds(mezFes.start)],
          ["End", formatUnixSeconds(mezFes.end)],
          ["Solo tickets", formatPanelNumber(mezFes.soloTickets)],
          ["Group tickets", formatPanelNumber(mezFes.groupTickets)],
          ["Stalls", stalls.length ? stalls.join(", ") : "N/A"],
        ],
      })
    );
    activeIds.add("mezfes");
  }

  const list = eventList(serverInfo);
  for (const def of SPECIAL_EVENT_DEFS) {
    if (activeIds.has(def.id)) continue;
    const flagActive = eventFlagActive(serverInfo, def.flags);
    if (!flagActive && !eventListIncludes(list, def.aliases)) continue;
    badges.push(
      eventBadge({
        id: def.id,
        label: def.label,
        image: def.image,
        rows: [["Status", "Active"]],
      })
    );
    activeIds.add(def.id);
  }

  return badges;
}
