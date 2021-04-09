// SPDX-License-Identifier: Apache-2.0 OR MIT

export enum ManagementType {
  Install,
  InstallDev,
  Update
}

export type Result = Map<ManagementType, string[]>
