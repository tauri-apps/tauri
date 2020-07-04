export interface Options {
  title: string
  body?: string
  icon?: string
}

export type PartialOptions = Omit<Options, 'title'>
export type Permission = 'granted' | 'denied' | 'default'
