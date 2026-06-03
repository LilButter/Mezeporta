import { access } from "node:fs/promises";
import path from "node:path";
import { constants } from "node:fs";
import { cwd, exit, platform } from "node:process";
import { pathToFileURL } from "node:url";

const helperPath = path.join(cwd(), "src-tauri", "bin", "meze-deps.exe");

export async function assertLinuxBuildReady() {
  if (platform !== "linux") {
    console.error(
      "Linux bundles must be built inside Linux or WSL. This command only runs on Linux hosts."
    );
    exit(1);
  }

  try {
    await access(helperPath, constants.R_OK);
  } catch (_error) {
    console.error(
      `Missing required helper: ${helperPath}\nStage the precompiled helper at src-tauri/bin/meze-deps.exe before running Linux dev/build.`
    );
    exit(1);
  }
}

async function main() {
  await assertLinuxBuildReady();
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    exit(1);
  });
}
