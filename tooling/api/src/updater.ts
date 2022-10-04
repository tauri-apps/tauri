// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Customize the auto updater flow.
 *
 * This package is also accessible with `window.__TAURI__.updater` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 * @module
 */

import { once, listen, emit, TauriEvent } from './event'
import { UnlistenFn } from './helpers/event'

/**
 * @since 1.0.0
 */
type UpdateStatus = 'PENDING' | 'ERROR' | 'DONE' | 'UPTODATE'

/**
 * @since 1.0.0
 */
interface UpdateStatusResult {
  error?: string
  status: UpdateStatus
}

/**
 * @since 1.0.0
 */
interface UpdateManifest {
  version: string
  date: string
  body: string
}

/**
 * @since 1.0.0
 */
interface UpdateResult {
  manifest?: UpdateManifest
  shouldUpdate: boolean
}

/**
 * Listen to an updater event.
 * @example
 * ```typescript
 * import { onUpdaterEvent } from "@tauri-apps/api/updater";
 * const unlisten = await onUpdaterEvent(({ error, status }) => {
 *  console.log('Updater event', error, status);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 *
 * @since 1.0.2
 */
async function onUpdaterEvent(
  handler: (status: UpdateStatusResult) => void
): Promise<UnlistenFn> {
  return listen(TauriEvent.STATUS_UPDATE, (data: { payload: any }) => {
    handler(data?.payload as UpdateStatusResult)
  })
}

/**
 * Install the update if there's one available.
 * @example
 * ```typescript
 * import { checkUpdate, installUpdate } from '@tauri-apps/api/updater';
 * const update = await checkUpdate();
 * if (update.shouldUpdate) {
 *   console.log(`Installing update ${update.manifest?.version}, ${update.manifest?.date}, ${update.manifest.body}`);
 *   await installUpdate();
 * }
 * ```
 *
 * @return A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function installUpdate(): Promise<void> {
  let unlistenerFn: UnlistenFn | undefined

  function cleanListener(): void {
    if (unlistenerFn) {
      unlistenerFn()
    }
    unlistenerFn = undefined
  }

  return new Promise((resolve, reject) => {
    function onStatusChange(statusResult: UpdateStatusResult): void {
      if (statusResult.error) {
        cleanListener()
        return reject(statusResult.error)
      }

      // install complete
      if (statusResult.status === 'DONE') {
        cleanListener()
        return resolve()
      }
    }

    // listen status change
    onUpdaterEvent(onStatusChange)
      .then((fn) => {
        unlistenerFn = fn
      })
      .catch((e) => {
        cleanListener()
        // dispatch the error to our checkUpdate
        throw e
      })

    // start the process we dont require much security as it's
    // handled by rust
    emit(TauriEvent.INSTALL_UPDATE).catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}

/**
 * Checks if an update is available.
 * @example
 * ```typescript
 * import { checkUpdate } from '@tauri-apps/api/updater';
 * const update = await checkUpdate();
 * // now run installUpdate() if needed
 * ```
 *
 * @return Promise resolving to the update status.
 *
 * @since 1.0.0
 */
async function checkUpdate(): Promise<UpdateResult> {
  let unlistenerFn: UnlistenFn | undefined

  function cleanListener(): void {
    if (unlistenerFn) {
      unlistenerFn()
    }
    unlistenerFn = undefined
  }

  return new Promise((resolve, reject) => {
    function onUpdateAvailable(manifest: UpdateManifest): void {
      cleanListener()
      return resolve({
        manifest,
        shouldUpdate: true
      })
    }

    function onStatusChange(statusResult: UpdateStatusResult): void {
      if (statusResult.error) {
        cleanListener()
        return reject(statusResult.error)
      }

      if (statusResult.status === 'UPTODATE') {
        cleanListener()
        return resolve({
          shouldUpdate: false
        })
      }
    }

    // wait to receive the latest update
    once(TauriEvent.UPDATE_AVAILABLE, (data: { payload: any }) => {
      onUpdateAvailable(data?.payload as UpdateManifest)
    }).catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })

    // listen status change
    onUpdaterEvent(onStatusChange)
      .then((fn) => {
        unlistenerFn = fn
      })
      .catch((e) => {
        cleanListener()
        // dispatch the error to our checkUpdate
        throw e
      })

    // start the process
    emit(TauriEvent.CHECK_UPDATE).catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}

export type { UpdateStatus, UpdateStatusResult, UpdateManifest, UpdateResult }

export { onUpdaterEvent, installUpdate, checkUpdate }
