import net from "net";

/**
 * Wait until a TCP server is accepting connections on the given port.
 * Retries every `intervalMs` until `timeoutMs` is reached.
 */
export async function waitForServer(
  port: number,
  timeoutMs = 30_000,
  intervalMs = 500
): Promise<void> {
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    const isUp = await new Promise<boolean>((resolve) => {
      const socket = net.createConnection({ port, host: "127.0.0.1" });
      socket.once("connect", () => {
        socket.destroy();
        resolve(true);
      });
      socket.once("error", () => {
        socket.destroy();
        resolve(false);
      });
    });

    if (isUp) return;
    await new Promise((r) => setTimeout(r, intervalMs));
  }

  throw new Error(`Server on port ${port} did not start within ${timeoutMs}ms`);
}
