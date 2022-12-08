import { invoke } from '@tauri-apps/api/tauri'

export const NAME = 'Tauri'

/**
 * Greets someone.
 * @param {string} name
 * @returns
 */
export async function greet(name) {
  return invoke('greet', { name })
}
