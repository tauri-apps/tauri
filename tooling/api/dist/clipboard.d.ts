/**
 * Writes a plain text to the clipboard.
 *
 * @returns A promise indicating the success or failure of the operation.
 */
declare function writeText(text: string): Promise<void>;
/**
 * Gets the clipboard content as plain text.
 *
 * @returns A promise resolving to the clipboard content as plain text.
 */
declare function readText(): Promise<string | null>;
export { writeText, readText };
