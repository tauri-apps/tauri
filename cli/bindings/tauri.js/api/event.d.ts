export interface Event<T> {
    type: string;
    payload: T;
}
export declare type EventCallback<T> = (event: Event<T>) => void;
/**
 * listen to an event from the backend
 *
 * @param event the event name
 * @param handler the event handler callback
 */
declare function listen<T>(event: string, handler: EventCallback<T>, once?: boolean): void;
/**
 * emits an event to the backend
 *
 * @param event the event name
 * @param [payload] the event payload
 */
declare function emit(event: string, payload?: string): void;
export { listen, emit };
