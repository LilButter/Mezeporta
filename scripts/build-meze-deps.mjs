import { mkdir, copyFile } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { cwd, exit, platform } from "node:process";

const repoRoot = cwd();
const crateRoot = path.join(repoRoot, "src-tauri", "meze-butter");
const targetDir = path.join(crateRoot, "target", "i686-pc-windows-msvc", "release");
const builtHelper = path.join(targetDir, "meze-deps.exe");
const stagedDir = path.join(repoRoot, "src-tauri", "bin");
const stagedHelper = path.join(stagedDir, "meze-deps.exe");

function cargoCommand() {
  return platform === "win32" ? "cargo.exe" : "cargo";
}

function runCargoBuild() {
  const result = spawnSync(
    cargoCommand(),
    ["build", "--release", "--target", "i686-pc-windows-msvc", "--bin", "meze-deps"],
    {
      cwd: crateRoot,
      stdio: "inherit",
      shell: false,
    }
  );

  if (result.error) {
    const message =
      result.error.code === "ENOENT"
        ? "Cargo was not found. Install the Rust toolchain for building meze-deps.exe."
        : result.error.message;
    throw new Error(message);
  }

  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }
}

async function main() {
  runCargoBuild();
  await mkdir(stagedDir, { recursive: true });
  await copyFile(builtHelper, stagedHelper);
  console.log(`Staged ${stagedHelper}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  exit(1);
});
