import { spawnSync } from "node:child_process";
import { env, exit } from "node:process";

const npmExecPath = env.npm_execpath;

if (!npmExecPath) {
  throw new Error("npm_execpath is not set");
}

const result = spawnSync(process.execPath, [npmExecPath, "exec", "--", "vite", "build"], {
  stdio: "inherit",
  shell: false,
  env: {
    ...env,
    BROWSERSLIST_IGNORE_OLD_DATA: env.BROWSERSLIST_IGNORE_OLD_DATA ?? "1",
  },
});

if (result.error) {
  throw result.error;
}

if (typeof result.status === "number") {
  exit(result.status);
}
