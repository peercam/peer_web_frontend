import { FullConfig } from "@playwright/test";
import path from "path";
import fs from "fs";
import kill from "tree-kill";

const PID_FILE = path.join(__dirname, ".test-pids.json");

function killProcess(pid: number): Promise<void> {
  return new Promise((resolve) => {
    kill(pid, "SIGTERM", (err) => {
      if (err) {
        // Process may already be dead — that's fine
        console.warn(`⚠️  Could not kill PID ${pid}: ${err.message}`);
      }
      resolve();
    });
  });
}

async function globalTeardown(config: FullConfig) {
  // Skip if we didn't start the servers
  if (process.env.SKIP_GLOBAL_SETUP) {
    console.log("⏭️  Skipping global teardown (SKIP_GLOBAL_SETUP=true)");
    return;
  }

  if (!fs.existsSync(PID_FILE)) {
    console.warn("⚠️  PID file not found — processes may already be stopped");
    return;
  }

  const pids = JSON.parse(fs.readFileSync(PID_FILE, "utf-8"));

  console.log("\n🧹 Stopping Leptos app...");
  await killProcess(pids.leptosApp);

  console.log("🧹 Stopping mock backend...");
  await killProcess(pids.mockBackend);

  fs.unlinkSync(PID_FILE);
  console.log("✅ All processes stopped");
}

export default globalTeardown;
