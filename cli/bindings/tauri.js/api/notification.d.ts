export interface Options {
    title: string;
    body?: string;
    icon?: string;
}
export declare type PartialOptions = Omit<Options, 'title'>;
export declare type Permission = 'granted' | 'denied' | 'default';
declare function isPermissionGranted(): Promise<boolean | null>;
declare function requestPermission(): Promise<Permission>;
declare function sendNotification(options: Options | string): void;
export { sendNotification, requestPermission, isPermissionGranted };
