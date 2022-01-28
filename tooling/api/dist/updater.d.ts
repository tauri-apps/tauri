declare type UpdateStatus = 'PENDING' | 'ERROR' | 'DONE' | 'UPTODATE';
interface UpdateStatusResult {
    error?: string;
    status: UpdateStatus;
}
interface UpdateManifest {
    version: string;
    date: string;
    body: string;
}
interface UpdateResult {
    manifest?: UpdateManifest;
    shouldUpdate: boolean;
}
/**
 * Install the update if there's one available.
 *
 * @return A promise indicating the success or failure of the operation.
 */
declare function installUpdate(): Promise<void>;
/**
 * Checks if an update is available.
 *
 * @return Promise resolving to the update status.
 */
declare function checkUpdate(): Promise<UpdateResult>;
export type { UpdateStatus, UpdateStatusResult, UpdateManifest, UpdateResult };
export { installUpdate, checkUpdate };
