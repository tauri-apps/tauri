// SPDX-License-Identifier: Apache-2.0 OR MIT

import { emit as emitEvent } from './helpers/event'

async function emit(event: string, payload?: string): Promise<void> {
  return emitEvent(event, undefined, payload)
}

export { listen, once } from './helpers/event'
export { emit }
