import { invoke } from '../tauri'

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
  async close() {
    return invoke('plugin:resources|close', {
      rid: this.rid
    })
  }
}

/** Extends a base class by other specifed classes */
export function applyMixins(baseClass: any, extendedClasses: any | any[]) {
  ;(Array.isArray(extendedClasses)
    ? extendedClasses
    : [extendedClasses]
  ).forEach((extendedClass) => {
    Object.getOwnPropertyNames(extendedClass.prototype).forEach((name) => {
      Object.defineProperty(
        baseClass.prototype,
        name,
        Object.getOwnPropertyDescriptor(extendedClass.prototype, name) ||
          Object.create(null)
      )
    })
  })
}
