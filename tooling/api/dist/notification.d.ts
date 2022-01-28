/**
 * Options to send a notification.
 */
interface Options {
    /** Notification title. */
    title: string;
    /** Optional notification body. */
    body?: string;
    /** Optional notification icon. */
    icon?: string;
}
/** Possible permission values. */
declare type Permission = 'granted' | 'denied' | 'default';
/**
 * Checks if the permission to send notifications is granted.
 *
 * @returns
 */
declare function isPermissionGranted(): Promise<boolean | null>;
/**
 * Requests the permission to send notifications.
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 */
declare function requestPermission(): Promise<Permission>;
/**
 * Sends a notification to the user.
 *
 * @param options Notification options.
 */
declare function sendNotification(options: Options | string): void;
export type { Options, Permission };
export { sendNotification, requestPermission, isPermissionGranted };
