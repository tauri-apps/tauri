export interface IManager {
  type: string;
  installPackage: (packageName: string) => void;
  installDevPackage: (packageName: string) => void;
  updatePackage: (packageName: string) => void;
  getPackageVersion: (packageName: string) => string | null;
  getLatestVersion: (packageName: string) => string;

}
