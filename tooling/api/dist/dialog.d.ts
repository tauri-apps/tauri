/** Extension filters for the file dialog. */
interface DialogFilter {
    /** Filter name. */
    name: string;
    /**
     * Extensions to filter, without a `.` prefix.
     * @example
     * ```typescript
     * extensions: ['svg', 'png']
     * ```
     */
    extensions: string[];
}
/** Options for the open dialog. */
interface OpenDialogOptions {
    /** The title of the dialog window. */
    title?: string;
    /** The filters of the dialog. */
    filters?: DialogFilter[];
    /** Initial directory or file path. */
    defaultPath?: string;
    /** Whether the dialog allows multiple selection or not. */
    multiple?: boolean;
    /** Whether the dialog is a directory selection or not. */
    directory?: boolean;
}
/** Options for the save dialog. */
interface SaveDialogOptions {
    /** The title of the dialog window. */
    title?: string;
    /** The filters of the dialog. */
    filters?: DialogFilter[];
    /**
     * Initial directory or file path.
     * If it's a directory path, the dialog interface will change to that folder.
     * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
     */
    defaultPath?: string;
}
/**
 * Open a file/directory selection dialog
 *
 * @returns A promise resolving to the selected path(s)
 */
declare function open(options?: OpenDialogOptions): Promise<string | string[]>;
/**
 * Open a file/directory save dialog.
 *
 * @returns A promise resolving to the selected path.
 */
declare function save(options?: SaveDialogOptions): Promise<string>;
/**
 * Shows a message dialog with an `Ok` button.
 *
 * @param {string} message The message to show.
 *
 * @return {Promise<void>} A promise indicating the success or failure of the operation.
 */
declare function message(message: string): Promise<void>;
/**
 * Shows a question dialog with `Yes` and `No` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|undefined} title The dialog's title. Defaults to the application name.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Yes` was clicked or not.
 */
declare function ask(message: string, title?: string): Promise<boolean>;
/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|undefined} title The dialog's title. Defaults to the application name.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Ok` was clicked or not.
 */
declare function confirm(message: string, title?: string): Promise<boolean>;
export type { DialogFilter, OpenDialogOptions, SaveDialogOptions };
export { open, save, message, ask, confirm };
