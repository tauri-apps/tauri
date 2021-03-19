import toml from '@tauri-apps/toml'
import fs from 'fs'
import { CargoLock, CargoManifest } from '../types/cargo'

export function readTomlFile<T extends CargoLock | CargoManifest>(
  filepath: string
): T | null {
  try {
    const file = fs.readFileSync(filepath).toString()
    return (toml.parse(file) as unknown) as T
  } catch (_) {
    return null
  }
}
