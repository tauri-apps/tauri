import { LiteralUnion } from 'type-fest';
interface Event<T> {
    /** Event name */
    event: EventName;
    /** Event identifier used to unlisten */
    id: number;
    /** Event payload */
    payload: T;
}
declare type EventName = LiteralUnion<'tauri://update' | 'tauri://update-available' | 'tauri://update-install' | 'tauri://update-status' | 'tauri://resize' | 'tauri://move' | 'tauri://close-requested' | 'tauri://focus' | 'tauri://blur' | 'tauri://scale-change' | 'tauri://menu' | 'tauri://file-drop' | 'tauri://file-drop-hover' | 'tauri://file-drop-cancelled', string>;
declare type EventCallback<T> = (event: Event<T>) => void;
declare type UnlistenFn = () => void;
/**
 * Listen to an event from the backend.
 *
 * @param event Event name
 * @param handler Event handler callback
 * @return A promise resolving to a function to unlisten to the event.
 */
declare function listen<T>(event: EventName, handler: EventCallback<T>): Promise<UnlistenFn>;
/**
 * Listen to an one-off event from the backend.
 *
 * @param event Event name
 * @param handler Event handler callback
 * @returns A promise resolving to a function to unlisten to the event.
 */
declare function once<T>(event: EventName, handler: EventCallback<T>): Promise<UnlistenFn>;
/**
 * Emits an event to the backend.
 *
 * @param event Event name
 * @param [payload] Event payload
 * @returns
 */
declare function emit(event: string, payload?: unknown): Promise<void>;
export type { Event, EventName, EventCallback, UnlistenFn };
export { listen, once, emit };
