import { FullConfig } from "@playwright/test";
import { spawn, ChildProcess } from "child_process";
import path from "path";
import fs from "fs";
import { waitForServer } from "./helpers/wait-for-server";

const PID_FILE = path.join(__dirname, ".test-pids.json");

interface ProcessPids {
  mockBackend: number;
  leptosApp: number;
}

async function globalSetup(config: FullConfig) {
  // Skip if servers are already running externally
  if (process.env.SKIP_GLOBAL_SETUP) {
    console.log("⏭️  Skipping global setup (SKIP_GLOBAL_SETUP=true)");
    return;
  }

  console.log("\n🔧 Starting mock backend...");

  const mockBackend = spawn("node", ["server.js"], {
    cwd: path.resolve(__dirname, "../../tests/mock_backend"),
    stdio: "pipe",
    detached: true,
  });

  mockBackend.stdout?.on("data", (data: Buffer) => {
    if (process.env.DEBUG) console.log(`[mock] ${data.toString().trim()}`);
  });
  mockBackend.stderr?.on("data", (data: Buffer) => {
    console.error(`[mock:err] ${data.toString().trim()}`);
  });

  await waitForServer(4000, 15_000);
  console.log("✅ Mock backend ready on :4000");

  console.log("🔧 Starting Leptos app...");

  const leptosApp = spawn("cargo", ["leptos", "serve", "--release"], {
    cwd: path.resolve(__dirname, "../"),
    stdio: "pipe",
    detached: true,
    env: {
      ...process.env,
      GRAPHQL_ENDPOINT: "http://localhost:4000/graphql",
    },
  });

  leptosApp.stdout?.on("data", (data: Buffer) => {
    if (process.env.DEBUG) console.log(`[leptos] ${data.toString().trim()}`);
  });
  leptosApp.stderr?.on("data", (data: Buffer) => {
    // cargo leptos logs to stderr
    if (process.env.DEBUG) console.log(`[leptos] ${data.toString().trim()}`);
  });

  await waitForServer(3000, 120_000); // Leptos compile can take a while
  console.log("✅ Leptos app ready on :3000");

  // Save PIDs for teardown
  const pids: ProcessPids = {
    mockBackend: mockBackend.pid!,
    leptosApp: leptosApp.pid!,
  };
  fs.writeFileSync(PID_FILE, JSON.stringify(pids));

  // Unref so this process can exit while children keep running
  mockBackend.unref();
  leptosApp.unref();
}

export default globalSetup;
