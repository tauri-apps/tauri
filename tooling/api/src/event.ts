// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { emit as emitEvent } from './helpers/event'

async function emit(event: string, payload?: string): Promise<void> {
  return emitEvent(event, undefined, payload)
}

export { listen, once } from './helpers/event'
export { emit }
