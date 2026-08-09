// Client-side bridge to the dedicated Worker hosting the wasm core (see
// src/workers/dbWorker.ts). One Worker per page load, request/response
// correlated by id -- mirrors the shape Tauri's invoke() already gives
// src/lib/tauri.ts, so webApi.ts can implement the same `api` interface.
import DbWorker from "../workers/dbWorker?worker";

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
}

let worker: Worker | null = null;
const pending = new Map<string, PendingRequest>();

function getWorker(): Worker {
  if (worker) return worker;
  worker = new DbWorker();
  worker.onmessage = (ev: MessageEvent) => {
    const { id, result, error } = ev.data as {
      id: string;
      result?: unknown;
      error?: string;
    };
    const req = pending.get(id);
    if (!req) return;
    pending.delete(id);
    if (error) req.reject(new Error(error));
    else req.resolve(result);
  };
  return worker;
}

/** Invoke a wasm-bindgen export by its exact (camelCase) JS name. */
export function callDb<T = unknown>(cmd: string, args: unknown[] = []): Promise<T> {
  return new Promise((resolve, reject) => {
    const id = crypto.randomUUID();
    pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
    getWorker().postMessage({ id, cmd, args });
  });
}
