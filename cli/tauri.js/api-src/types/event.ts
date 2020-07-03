export interface Event {
  type: string
  payload: unknown
}

export type EventCallback = (event: Event) => void
