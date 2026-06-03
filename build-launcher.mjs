import { spawnSync } from "node:child_process";
import { createInterface } from "node:readline/promises";
import { stdin as input, stdout as output, exit, platform } from "node:process";

function npmCommand() {
  return platform === "win32" ? "npm.cmd" : "npm";
}

function runNpmScript(scriptName) {
  const result = spawnSync(npmCommand(), ["run", scriptName], {
    stdio: "inherit",
    shell: false,
  });
  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }
}

function resolveTarget(argv) {
  const targetArg = argv.find((arg) => arg.startsWith("--target="));
  if (targetArg) {
    return targetArg.slice("--target=".length).trim().toLowerCase();
  }

  const targetIndex = argv.indexOf("--target");
  if (targetIndex >= 0) {
    return String(argv[targetIndex + 1] ?? "").trim().toLowerCase();
  }

  return null;
}

async function promptTarget() {
  const rl = createInterface({ input, output });
  try {
    output.write("Choose build target:\n");
    output.write("1) Windows\n");
    output.write("2) Linux\n");
    const answer = String(await rl.question("> ")).trim();
    if (answer === "1" || /^win(dows)?$/i.test(answer)) return "windows";
    if (answer === "2" || /^linux$/i.test(answer)) return "linux";
    throw new Error("Invalid selection. Choose 1 for Windows or 2 for Linux.");
  } finally {
    rl.close();
  }
}

async function main() {
  const target = resolveTarget(process.argv.slice(2)) ?? (await promptTarget());

  if (target === "windows") {
    runNpmScript("tauri:build");
    return;
  }

  if (target === "linux") {
    if (platform !== "linux") {
      console.error(
        "Linux bundles must be built inside Linux or WSL. Run this script from a Linux shell."
      );
      exit(1);
    }
    runNpmScript("tauri:build:linux");
    return;
  }

  console.error(`Unsupported target "${target}". Use "windows" or "linux".`);
  exit(1);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  exit(1);
});
