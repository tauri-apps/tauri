import { invoke } from '@tauri-apps/api/core';

let enabled = false;

export function enableConsoleCapture() {
  if (enabled) return;
  enabled = true;

  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;

  console.log = (...args) => {
    originalLog(...args);
    sendLog('LOG', args);
  };

  console.warn = (...args) => {
    originalWarn(...args);
    sendLog('WARN', args);
  };

  console.error = (...args) => {
    originalError(...args);
    sendLog('ERROR', args);
  };
}

function sendLog(level: string, args: unknown[]) {
  const message = args
    .map((a) => {
      if (typeof a === 'object') {
        try {
          return JSON.stringify(a);
        } catch {
          return String(a);
        }
      }
      return String(a);
    })
    .join(' ');

  invoke('console_log', { level, message }).catch(() => {});
}

export async function flushConsoleLog(): Promise<string> {
  return await invoke('flush_console_log');
}

export async function clearConsoleLog(): Promise<string> {
  return await invoke('clear_console_log');
}