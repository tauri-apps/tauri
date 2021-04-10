// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export enum ManagementType {
  Install,
  InstallDev,
  Update
}

export type Result = Map<ManagementType, string[]>
