export enum ManagementType {
  Install,
  InstallDev,
  Update
}

export type Result = Map<ManagementType, string[]>
