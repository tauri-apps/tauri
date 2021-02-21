import { emit as emitEvent } from './helpers/event'

async function emit(event: string, payload?: string): Promise<void> {
  return emitEvent(event, undefined, payload)
}

export { listen } from './helpers/event'
export { emit }
