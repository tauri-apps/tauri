// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export enum ManagementType {
  Install,
  InstallDev,
  Update
}

export type Result = Map<ManagementType, string[]>

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions
export type Answer = { answer: boolean }
