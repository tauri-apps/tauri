// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { once, listen, emit, UnlistenFn } from './helpers/event'

export type UpdateStatus = 'PENDING' | 'ERROR' | 'DONE' | 'UPTODATE'

export interface UpdateStatusResult {
  error?: string
  status: UpdateStatus
}

export interface UpdateManifest {
  version: string
  date: string
  body: string
}

export interface UpdateResult {
  manifest?: UpdateManifest
  shouldUpdate: boolean
}

export async function installUpdate(): Promise<void> {
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
    listen('tauri://update-status', (data: { payload: any }) => {
      onStatusChange(data?.payload as UpdateStatusResult)
    })
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
    emit('tauri://update-install').catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}

export async function checkUpdate(): Promise<UpdateResult> {
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
    once('tauri://update-available', (data: { payload: any }) => {
      onUpdateAvailable(data?.payload as UpdateManifest)
    }).catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })

    // listen status change
    listen('tauri://update-status', (data: { payload: any }) => {
      onStatusChange(data?.payload as UpdateStatusResult)
    })
      .then((fn) => {
        unlistenerFn = fn
      })
      .catch((e) => {
        cleanListener()
        // dispatch the error to our checkUpdate
        throw e
      })

    // start the process
    emit('tauri://update').catch((e) => {
      cleanListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}
