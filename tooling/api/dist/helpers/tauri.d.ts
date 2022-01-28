declare type TauriModule = 'App' | 'Fs' | 'Path' | 'Os' | 'Window' | 'Shell' | 'Event' | 'Internal' | 'Dialog' | 'Cli' | 'Notification' | 'Http' | 'GlobalShortcut' | 'Process' | 'Clipboard';
interface TauriCommand {
    __tauriModule: TauriModule;
    [key: string]: unknown;
}
declare function invokeTauriCommand<T>(command: TauriCommand): Promise<T>;
export type { TauriModule, TauriCommand };
export { invokeTauriCommand };
