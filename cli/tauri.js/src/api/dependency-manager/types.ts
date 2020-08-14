export enum ManagementType {
  Install,
  Update
}

export type Result = Map<ManagementType, string[]>;
