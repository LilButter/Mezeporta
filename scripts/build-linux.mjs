import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { argv, env, exit } from "node:process";
import { fileURLToPath } from "node:url";
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

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..");
const linuxBundleDir = resolve(
  projectRoot,
  "src-tauri",
  "target",
  "x86_64-unknown-linux-gnu",
  "release",
  "bundle",
  "appimage"
);
const linuxReleaseDir = resolve(
  projectRoot,
  "src-tauri",
  "target",
  "x86_64-unknown-linux-gnu",
  "release"
);
const appDirPath = resolve(linuxBundleDir, "mezeporta.AppDir");
const appImageFileName = "mezeporta_1.5.2_amd64.AppImage";
const extractedLinuxdeployDir = resolve(linuxBundleDir, ".linuxdeploy-tools");
const portableBundleDir = resolve(
  projectRoot,
  "src-tauri",
  "target",
  "x86_64-unknown-linux-gnu",
  "release",
  "bundle",
  "portable"
);
const portableStagingDir = resolve(portableBundleDir, "Mezeporta-linux-x64-portable");
const portableTarballPath = resolve(
  portableBundleDir,
  "Mezeporta_1.5.2_linux_x64_portable.tar.gz"
);
const linuxBinaryFileNames = ["mezeporta", "app"];
const mezeDepsSourcePath = resolve(projectRoot, "src-tauri", "bin", "meze-deps.exe");
const audioSourceDir = resolve(projectRoot, "public", "audio");
const portableOnly = argv.includes("--portable-only");
const runtimeSetupScripts = [
  [
    resolve(projectRoot, "scripts", "linux", "install-deps-ubuntu.sh"),
    "mezeporta-setup-ubuntu.sh",
  ],
  [
    resolve(projectRoot, "scripts", "linux", "install-deps-arch.sh"),
    "mezeporta-setup-arch.sh",
  ],
];

function walkFiles(rootDir, visitor) {
  for (const entry of readdirSync(rootDir, { withFileTypes: true })) {
    const fullPath = resolve(rootDir, entry.name);
    if (entry.isDirectory()) {
      walkFiles(fullPath, visitor);
      continue;
    }
    visitor(fullPath, entry.name);
  }
}

function copyDirectoryRecursive(sourceDir, destinationDir) {
  mkdirSync(destinationDir, { recursive: true });

  for (const entry of readdirSync(sourceDir, { withFileTypes: true })) {
    const sourcePath = resolve(sourceDir, entry.name);
    const destinationPath = resolve(destinationDir, entry.name);
    if (entry.isDirectory()) {
      copyDirectoryRecursive(sourcePath, destinationPath);
      continue;
    }
    copyFileSync(sourcePath, destinationPath);
  }
}

function resolveLinuxBinarySourcePath() {
  for (const fileName of linuxBinaryFileNames) {
    const candidate = resolve(linuxReleaseDir, fileName);
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  const searched = linuxBinaryFileNames
    .map((fileName) => resolve(linuxReleaseDir, fileName))
    .join(", ");
  throw new Error(`Missing Linux release binary. Searched: ${searched}`);
}

function removeBundledGStreamer() {
  if (!existsSync(appDirPath)) {
    throw new Error(`Missing AppDir for Linux bundle: ${appDirPath}`);
  }

  walkFiles(appDirPath, (fullPath, fileName) => {
    if (
      /^libgstreamer.*\.so(?:\..*)?$/i.test(fileName) ||
      /^libgst.*\.so(?:\..*)?$/i.test(fileName) ||
      fileName === "gst-plugin-scanner"
    ) {
      unlinkSync(fullPath);
    }
  });
}

function resolveAppImageTool() {
  const homeDir = env.HOME;
  if (!homeDir) {
    throw new Error("HOME is not set; cannot locate cached linuxdeploy AppImage.");
  }

  const linuxdeployPath = resolve(
    homeDir,
    ".cache",
    "tauri",
    "linuxdeploy-x86_64.AppImage"
  );
  if (!existsSync(linuxdeployPath)) {
    throw new Error(
      `Missing cached linuxdeploy AppImage: ${linuxdeployPath}`
    );
  }

  chmodSync(linuxdeployPath, 0o755);
  const appImageToolPath = resolve(
    extractedLinuxdeployDir,
    "squashfs-root",
    "plugins",
    "linuxdeploy-plugin-appimage",
    "appimagetool-prefix",
    "AppRun"
  );
  if (existsSync(appImageToolPath)) {
    chmodSync(appImageToolPath, 0o755);
    return appImageToolPath;
  }

  rmSync(extractedLinuxdeployDir, { recursive: true, force: true });
  mkdirSync(extractedLinuxdeployDir, { recursive: true });

  const result = spawnSync(
    linuxdeployPath,
    [
      "--appimage-extract",
    ],
    {
      cwd: extractedLinuxdeployDir,
      stdio: "ignore",
      shell: false,
      env: {
        ...env,
      },
    }
  );

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }

  if (!existsSync(appImageToolPath)) {
    throw new Error(`Unable to locate extracted appimagetool at ${appImageToolPath}`);
  }

  chmodSync(appImageToolPath, 0o755);
  return appImageToolPath;
}

function rebuildSanitizedAppImage() {
  const appImageToolPath = resolveAppImageTool();
  rmSync(resolve(linuxBundleDir, appImageFileName), { force: true });

  const result = spawnSync(
    appImageToolPath,
    ["mezeporta.AppDir", appImageFileName],
    {
      cwd: linuxBundleDir,
      stdio: "inherit",
      shell: false,
      env: {
        ...env,
        ARCH: "x86_64",
      },
    }
  );

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }
}

function copyRuntimeSetupScripts() {
  mkdirSync(linuxBundleDir, { recursive: true });

  for (const [source, fileName] of runtimeSetupScripts) {
    if (!existsSync(source)) {
      throw new Error(`Missing runtime setup script: ${source}`);
    }
    const destination = resolve(linuxBundleDir, fileName);
    copyFileSync(source, destination);
    chmodSync(destination, 0o755);
  }
}

function copyRawLinuxHelperLayout() {
  const destinationDir = resolve(linuxReleaseDir, "bin");
  mkdirSync(destinationDir, { recursive: true });
  copyFileSync(mezeDepsSourcePath, resolve(destinationDir, "meze-deps.exe"));
}

function copyRawLinuxAudioLayout() {
  const destinationDir = resolve(linuxReleaseDir, "audio");
  rmSync(destinationDir, { recursive: true, force: true });
  copyDirectoryRecursive(audioSourceDir, destinationDir);
}

function writePortableLauncherScript() {
  const launcherPath = resolve(portableStagingDir, "run-mezeporta.sh");
  writeFileSync(
    launcherPath,
    `#!/usr/bin/env sh
set -eu
HERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$HERE"
exec "$HERE/mezeporta-bin" "$@"
`,
    { mode: 0o755 }
  );
  chmodSync(launcherPath, 0o755);
}

function writePortableReadme() {
  writeFileSync(
    resolve(portableStagingDir, "README-Linux-Portable.txt"),
    `Mezeporta Linux portable package

Extract these files into the folder where the launcher should live, then run:

./run-mezeporta.sh

This archive is intentionally flat so it can be extracted directly beside the game files instead of creating another nested launcher folder.
The launcher creates its runtime data under ./Mezeporta, so do not rename any extracted file to Mezeporta.

Included layout:
- run-mezeporta.sh: launcher script
- mezeporta-bin: Linux launcher executable
- Mezeporta/bin/meze-deps.exe: helper used by Linux/Wine patch flows
- Mezeporta/audio/: UI sound effects for Linux builds
- mezeporta-setup-ubuntu.sh / mezeporta-setup-arch.sh: optional runtime dependency installers
`
  );
}

function createPortableTarball(sanitizedPath) {
  const linuxBinarySourcePath = resolveLinuxBinarySourcePath();
  if (!existsSync(mezeDepsSourcePath)) {
    throw new Error(`Missing Linux helper binary: ${mezeDepsSourcePath}`);
  }
  if (!existsSync(audioSourceDir)) {
    throw new Error(`Missing Linux audio source directory: ${audioSourceDir}`);
  }

  rmSync(portableStagingDir, { recursive: true, force: true });
  mkdirSync(resolve(portableStagingDir, "Mezeporta", "bin"), { recursive: true });

  copyFileSync(linuxBinarySourcePath, resolve(portableStagingDir, "mezeporta-bin"));
  chmodSync(resolve(portableStagingDir, "mezeporta-bin"), 0o755);
  copyFileSync(
    mezeDepsSourcePath,
    resolve(portableStagingDir, "Mezeporta", "bin", "meze-deps.exe")
  );
  copyDirectoryRecursive(audioSourceDir, resolve(portableStagingDir, "Mezeporta", "audio"));

  for (const [source, fileName] of runtimeSetupScripts) {
    if (!existsSync(source)) {
      throw new Error(`Missing runtime setup script: ${source}`);
    }
    const destination = resolve(portableStagingDir, fileName);
    copyFileSync(source, destination);
    chmodSync(destination, 0o755);
  }

  writePortableLauncherScript();
  writePortableReadme();

  rmSync(portableTarballPath, { force: true });
  const archiveEntries = readdirSync(portableStagingDir);
  const result = spawnSync(
    "tar",
    ["-czf", portableTarballPath, ...archiveEntries],
    {
      cwd: portableStagingDir,
      stdio: "inherit",
      shell: false,
      env: {
        ...env,
        PATH: sanitizedPath,
      },
    }
  );

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }

  console.log(`Created portable Linux tarball: ${portableTarballPath}`);
}

async function main() {
  const sanitizedPath = sanitizeLinuxPath(env.PATH);

  if (portableOnly) {
    copyRawLinuxHelperLayout();
    copyRawLinuxAudioLayout();
    createPortableTarball(sanitizedPath);
    return;
  }

  await assertLinuxBuildReady();

  const result = spawnSync(
    "tauri",
    [
      "build",
      "--target",
      "x86_64-unknown-linux-gnu",
      "--bundles",
      "appimage,deb",
      "--config",
      "src-tauri/tauri.linux.conf.json",
    ],
    {
      stdio: "inherit",
      shell: false,
      env: {
        ...env,
        PATH: sanitizedPath,
      },
    }
  );

  if (result.error) {
    throw result.error;
  }
  if (typeof result.status === "number" && result.status !== 0) {
    exit(result.status);
  }

  removeBundledGStreamer();
  rebuildSanitizedAppImage();
  copyRuntimeSetupScripts();
  copyRawLinuxHelperLayout();
  copyRawLinuxAudioLayout();
  createPortableTarball(sanitizedPath);
  rmSync(extractedLinuxdeployDir, { recursive: true, force: true });
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  exit(1);
});
