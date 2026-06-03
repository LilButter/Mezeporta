import { spawnSync } from "node:child_process";
import { cwd, env, exit, platform } from "node:process";

const modeArg = process.argv[2] ?? "build";
const mode =
  modeArg === "dev" || modeArg === "dev-gpu"
    ? "dev"
    : modeArg === "portable"
      ? "portable"
      : "build";
const forceGpuMesa = modeArg === "dev-gpu";
const npmScript =
  mode === "dev"
    ? "tauri:dev:linux"
    : mode === "portable"
      ? "tauri:build:linux:portable"
      : "tauri:build:linux";
const EXCLUDED_DEFAULT_DISTROS = new Set(["docker-desktop", "docker-desktop-data"]);
let selectedDistro = env.MEZEPORTA_WSL_DISTRO?.trim() ?? "";

function wslArgs(commandArgs) {
  const args = [];
  if (selectedDistro) {
    args.push("-d", selectedDistro);
  }
  args.push("--", ...commandArgs);
  return args;
}

function runWsl(commandArgs, options = {}) {
  return spawnSync("wsl.exe", wslArgs(commandArgs), {
    stdio: options.stdio ?? "pipe",
    encoding: options.encoding ?? "utf8",
    shell: false,
  });
}

function normalizeWslOutput(value) {
  return String(value ?? "")
    .replace(/\u0000/g, "")
    .replace(/^\uFEFF/, "")
    .trim();
}

function listWslDistros() {
  const result = spawnSync("wsl.exe", ["-l", "-q"], {
    stdio: "pipe",
    encoding: "utf8",
    shell: false,
  });

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    throw new Error(
      normalizeWslOutput(result.stderr || result.stdout || "failed to list WSL distros")
    );
  }

  return normalizeWslOutput(result.stdout)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function resolveWslDistro() {
  if (selectedDistro) {
    return;
  }

  const distros = listWslDistros();
  const candidates = distros.filter((distro) => !EXCLUDED_DEFAULT_DISTROS.has(distro.toLowerCase()));
  selectedDistro =
    candidates.find((distro) => /^ubuntu(?:$|-)/i.test(distro)) ??
    candidates[0] ??
    "";

  if (!selectedDistro) {
    throw new Error(
      "No usable WSL distro found. Install Ubuntu or set MEZEPORTA_WSL_DISTRO to a Linux distro name."
    );
  }
}

function shellQuote(value) {
  return `'${String(value).replace(/'/g, `'\\''`)}'`;
}

function shellEnvAssignment(name, value) {
  return `${name}=${shellQuote(value)}`;
}

function shellEnvExpression(name, expression) {
  return `${name}=${expression}`;
}

function wslAudioEnvPrefix() {
  const pulseServer = env.PULSE_SERVER?.trim()
    ? shellEnvAssignment("PULSE_SERVER", env.PULSE_SERVER.trim())
    : shellEnvExpression("PULSE_SERVER", "/run/user/$(id -u)/pulse/native");
  const runtimeDir = env.XDG_RUNTIME_DIR?.trim()
    ? shellEnvAssignment("XDG_RUNTIME_DIR", env.XDG_RUNTIME_DIR.trim())
    : shellEnvExpression("XDG_RUNTIME_DIR", "/run/user/$(id -u)");

  return [
    shellEnvAssignment("ALSOFT_DRIVERS", env.ALSOFT_DRIVERS?.trim() || "pulse,null"),
    pulseServer,
    runtimeDir,
    shellEnvAssignment("SDL_AUDIODRIVER", env.SDL_AUDIODRIVER?.trim() || "pulseaudio"),
  ].join(" ");
}

function wslGraphicsEnvPrefix() {
  if (mode !== "dev" || (!forceGpuMesa && !env.MEZEPORTA_WSL_GPU_ADAPTER?.trim())) {
    return "";
  }

  const adapterName = env.MEZEPORTA_WSL_GPU_ADAPTER?.trim() || "NVIDIA";
  return [
    shellEnvAssignment("MESA_D3D12_DEFAULT_ADAPTER_NAME", adapterName),
    shellEnvAssignment("GALLIUM_DRIVER", "d3d12"),
    shellEnvAssignment("MESA_LOADER_DRIVER_OVERRIDE", "d3d12"),
    shellEnvAssignment("LIBGL_ALWAYS_SOFTWARE", "false"),
    shellEnvAssignment("LIBGL_KOPPER_DISABLE", "true"),
  ].join(" ");
}

function windowsPathToWslPath(windowsPath) {
  const result = runWsl(["wslpath", "-a", windowsPath]);
  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    throw new Error(
      String(result.stderr || result.stdout || "failed to convert workspace path with wslpath").trim()
    );
  }
  return String(result.stdout ?? "").trim();
}

function main() {
  if (platform !== "win32") {
    console.error(`Use "npm run ${npmScript}" directly on Linux.`);
    exit(1);
  }

  resolveWslDistro();
  console.log(`Running ${npmScript} in WSL distro: ${selectedDistro}`);
  if (mode === "dev" && (forceGpuMesa || env.MEZEPORTA_WSL_GPU_ADAPTER?.trim())) {
    console.log(`Using WSL Mesa D3D12 GPU adapter: ${env.MEZEPORTA_WSL_GPU_ADAPTER?.trim() || "NVIDIA"}`);
  }

  const wslCwd = windowsPathToWslPath(cwd());
  const envPrefix = [wslAudioEnvPrefix(), wslGraphicsEnvPrefix()].filter(Boolean).join(" ");
  const npmCommand = `${envPrefix ? `${envPrefix} ` : ""}npm run ${npmScript}`;
  const command = `cd ${shellQuote(wslCwd)} && ${npmCommand}`;
  const result = runWsl(["/bin/sh", "-lc", command], {
    stdio: "inherit",
    encoding: undefined,
  });

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  exit(1);
}
