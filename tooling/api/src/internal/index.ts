// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '../primitives'

/**
 * A rust-backed resource.
 *
 * The resource lives in the main process and does not exist
 * in the Javascript world, and thus will not be cleaned up automatiacally
 * except on application exit. If you want to clean it up early, call {@linkcode Resource.close}
 */
export class Resource {
  #rid: number

  get rid(): number {
    return this.#rid
  }

  constructor(rid: number) {
    this.#rid = rid
  }

  /**
   * Destroys and cleans up this resource from memory.
   * **You should not call any method on this object anymore and should drop any reference to it.**
   */
  async close(): Promise<void> {
    return invoke('plugin:resources|close', {
      rid: this.rid
    })
  }
}

/** Extends a base class by other specifed classes */
export function applyMixins(
  baseClass: { prototype: unknown },
  extendedClasses: unknown
): void {
  ;(Array.isArray(extendedClasses)
    ? extendedClasses
    : [extendedClasses]
  ).forEach((extendedClass: { prototype: unknown }) => {
    Object.getOwnPropertyNames(extendedClass.prototype).forEach((name) => {
      Object.defineProperty(
        baseClass.prototype,
        name,
        // eslint-disable-next-line
        Object.getOwnPropertyDescriptor(extendedClass.prototype, name) ??
          Object.create(null)
      )
    })
  })
}
