import { spawnSync } from "node:child_process";
import { env, exit } from "node:process";
import { assertLinuxBuildReady } from "./check-linux-helper.mjs";

const WINDOWS_PATH_PATTERNS = [
  /^\/mnt\/[a-z]\/windows(\/|$)/i,
  /^\/mnt\/[a-z]\/program files( \(x86\))?(\/|$)/i,
  /^\/mnt\/[a-z]\/users\/[^/]+\/appdata\/local\/microsoft\/windowsapps(\/|$)/i,
];

function sanitizeLinuxPath(inputPath) {
  const segments = (inputPath ?? "").split(":").filter(Boolean);
  const filtered = [];

  for (const segment of segments) {
    if (WINDOWS_PATH_PATTERNS.some((pattern) => pattern.test(segment))) {
      continue;
    }
    if (!filtered.includes(segment)) {
      filtered.push(segment);
    }
  }

  return filtered.join(":");
}

function linuxTauriEnv() {
  return {
    ...env,
    ALSOFT_DRIVERS: env.ALSOFT_DRIVERS || "pulse,null",
    CARGO_BUILD_TARGET: "x86_64-unknown-linux-gnu",
    PATH: sanitizeLinuxPath(env.PATH),
    SDL_AUDIODRIVER: env.SDL_AUDIODRIVER || "pulseaudio",
  };
}

async function main() {
  await assertLinuxBuildReady();

  const result = spawnSync(
    "tauri",
    [
      "dev",
      "--target",
      "x86_64-unknown-linux-gnu",
      "--config",
      "src-tauri/tauri.linux.conf.json",
    ],
    {
      stdio: "inherit",
      shell: false,
      env: linuxTauriEnv(),
    }
  );

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  exit(1);
});
