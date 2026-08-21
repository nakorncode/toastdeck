import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const cargoBin = path.join(os.homedir(), ".cargo", "bin");
const tauriJs = path.join(root, "node_modules", "@tauri-apps", "cli", "tauri.js");

function envWithCargoOnPath(source) {
  const env = { ...source };
  const current = Object.entries(env).find(([key]) => key.toUpperCase() === "PATH")?.[1] ?? "";

  for (const key of Object.keys(env)) {
    if (key.toUpperCase() === "PATH") {
      delete env[key];
    }
  }

  const pathKey = process.platform === "win32" ? "Path" : "PATH";
  env[pathKey] = `${cargoBin}${path.delimiter}${current}`;
  return env;
}

const child = spawn(process.execPath, [tauriJs, ...process.argv.slice(2)], {
  cwd: root,
  env: envWithCargoOnPath(process.env),
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.exit(1);
  }
  process.exit(code ?? 1);
});
