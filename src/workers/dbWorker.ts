// Dedicated Worker hosting the wasm core. Runs inside a Worker (not the main
// thread) because OPFS's FileSystemSyncAccessHandle API -- what the sahpool
// VFS uses for persistence -- is only available in worker contexts, not on
// the main thread, in any current browser. Confirmed against the real crate
// during the Phase 1 spike; see streamed-yawning-coral.md.
import init, * as wasmApi from "../wasm-core/pkg/samsmrti_lib.js";

type WasmApi = Record<string, (...args: unknown[]) => unknown>;

interface WorkerRequest {
  id: string;
  cmd: string;
  args?: unknown[];
}

interface WorkerResponse {
  id: string;
  result?: unknown;
  error?: string;
}

// Narrow surface of DedicatedWorkerGlobalScope this file needs -- avoids
// depending on the "WebWorker" lib in tsconfig (the app project targets DOM,
// not worker globals).
interface WorkerScope {
  postMessage(message: WorkerResponse): void;
  onmessage: ((ev: MessageEvent<WorkerRequest>) => void) | null;
}

const workerSelf = self as unknown as WorkerScope;

const ready: Promise<void> = (async () => {
  await init();
  await wasmApi.initOpfs();
})();

workerSelf.onmessage = async (ev: MessageEvent<WorkerRequest>) => {
  const { id, cmd, args } = ev.data;
  try {
    await ready;
    const fn = (wasmApi as unknown as WasmApi)[cmd];
    if (typeof fn !== "function") {
      throw new Error(`Unknown db worker command: ${cmd}`);
    }
    const result = fn(...(args ?? []));
    const response: WorkerResponse = { id, result };
    workerSelf.postMessage(response);
  } catch (e) {
    const response: WorkerResponse = {
      id,
      error: e instanceof Error ? e.message : String(e),
    };
    workerSelf.postMessage(response);
  }
};
