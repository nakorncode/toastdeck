import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const cargoBin = path.join(os.homedir(), ".cargo", "bin");
const env = { ...process.env };
const pathKey = process.platform === "win32" ? "Path" : "PATH";
env[pathKey] = `${cargoBin}${path.delimiter}${env[pathKey] ?? ""}`;

const AUMID = "NakornCode.ToastDesk.v2";
const PROBE = "ToastDesk native-loop";

const cargo = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "native::tests::push_probe_toast",
    "--",
    "--exact",
    "--ignored",
    "--nocapture",
  ],
  { cwd: root, env, encoding: "utf8" },
);

if (cargo.status !== 0) {
  console.log(
    JSON.stringify(
      {
        ok: false,
        failures: ["cargo test native::tests::push_probe_toast failed"],
        stdout: cargo.stdout,
        stderr: cargo.stderr,
      },
      null,
      2,
    ),
  );
  process.exit(1);
}

const history = spawnSync(
  "powershell",
  [
    "-NoProfile",
    "-Command",
    `[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
$history = [Windows.UI.Notifications.ToastNotificationManager]::History.GetHistory('${AUMID}')
$xml = @($history | ForEach-Object { $_.Content.GetXml() })
$match = @($xml | Where-Object { $_ -like '*${PROBE}*' })
Write-Output ("COUNT=" + $history.Count)
Write-Output ("MATCH=" + $match.Count)
if ($match.Count -gt 0) { $match | Select-Object -First 1 }`,
  ],
  { encoding: "utf8" },
);

const output = `${history.stdout ?? ""}\n${history.stderr ?? ""}`;
const count = Number((output.match(/^COUNT=(\d+)/m) ?? [])[1] ?? "0");
const match = Number((output.match(/^MATCH=(\d+)/m) ?? [])[1] ?? "0");
const failures = [];
if (history.status !== 0) {
  failures.push("could not read ToastNotificationManager history");
}
if (match < 1) {
  failures.push(`no history toast for ${AUMID} matching ${PROBE} (count=${count})`);
}

const report = {
  ok: failures.length === 0,
  failures,
  count,
  match,
  historyOutput: output.trim(),
};
console.log(JSON.stringify(report, null, 2));
process.exit(report.ok ? 0 : 1);
