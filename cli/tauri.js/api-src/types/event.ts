export interface Event<T> {
  type: string
  payload: T
}

export type EventCallback<T> = (event: Event<T>) => void
