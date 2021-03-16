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
  const allUnlisteners: UnlistenFn[] = []

  function cleanAllListener(): void {
    allUnlisteners.forEach((unlistenFn) => {
      unlistenFn()
    })
  }

  return new Promise((resolve, reject) => {
    function onStatusChange(statusResult: UpdateStatusResult): void {
      console.log({ statusResult })
      if (statusResult.error) {
        cleanAllListener()
        return reject(statusResult.error)
      }

      // install complete
      if (statusResult.status === 'DONE') {
        cleanAllListener()
        return resolve()
      }
    }

    // listen status change
    listen('tauri://update-status', (data: { payload: any }) => {
      onStatusChange(data?.payload as UpdateStatusResult)
    })
      .then((unlistenFn) => {
        allUnlisteners.push(unlistenFn)
      })
      .catch((e) => {
        cleanAllListener()
        // dispatch the error to our checkUpdate
        throw e
      })

    // start the process we dont require much security as it's
    // handled by rust
    emit('tauri://update-install').catch((e) => {
      cleanAllListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}

export async function checkUpdate(): Promise<UpdateResult> {
  const allUnlisteners: UnlistenFn[] = []

  function cleanAllListener(): void {
    allUnlisteners.forEach((unlistenFn) => {
      unlistenFn()
    })
  }

  return new Promise((resolve, reject) => {
    function onUpdateAvailable(manifest: UpdateManifest): void {
      cleanAllListener()
      return resolve({
        manifest,
        shouldUpdate: true
      })
    }

    function onStatusChange(statusResult: UpdateStatusResult): void {
      if (statusResult.error) {
        cleanAllListener()
        return reject(statusResult.error)
      }

      if (statusResult.status === 'UPTODATE') {
        cleanAllListener()
        return resolve({
          shouldUpdate: false
        })
      }
    }

    // wait to receive the latest update
    once('tauri://update-available', (data: { payload: any }) => {
      onUpdateAvailable(data?.payload as UpdateManifest)
    }).catch((e) => {
      cleanAllListener()
      // dispatch the error to our checkUpdate
      throw e
    })

    // listen status change
    listen('tauri://update-status', (data: { payload: any }) => {
      onStatusChange(data?.payload as UpdateStatusResult)
    })
      .then((unlistenFn) => {
        allUnlisteners.push(unlistenFn)
      })
      .catch((e) => {
        cleanAllListener()
        // dispatch the error to our checkUpdate
        throw e
      })

    // start the process
    emit('tauri://update').catch((e) => {
      cleanAllListener()
      // dispatch the error to our checkUpdate
      throw e
    })
  })
}
