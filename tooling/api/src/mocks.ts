interface IPCMessage {
  cmd: string,
  callback: number,
  error: number,
  [key: string]: unknown
}

export function mockIPC(cb: (cmd: string, args: Record<string, unknown>) => any): void {
  if ("__TAURI_IPC__" in window)
    throw new Error("window.__TAURI_IPC__ is already defined");

  window.__TAURI_IPC__ = async ({ cmd, callback, error, ...args }: IPCMessage) => {
    try {
      // @ts-expect-error The function key is dynamic and therefore not typed
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      window[`_${callback}`](await cb(cmd, args));
    } catch (err) {
      // @ts-expect-error The function key is dynamic and therefore not typed
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      window[`_${error}`](err);
    }
  };
}

export function mockWindows(current: string, ...additionalWindows: string[]): void {
  if ("__TAURI_METADATA__" in window)
    throw new Error("window.__TAURI_METADATA__ is already defined");

  window.__TAURI_METADATA__ = {
    __windows: [current, ...additionalWindows].map((label) => ({ label })),
    __currentWindow: { label: current },
  };
}
export function clearMocks(): void {
  // @ts-expect-error
  delete window.__TAURI_IPC__
  // @ts-expect-error
  delete window.__TAURI_METADATA__
}
