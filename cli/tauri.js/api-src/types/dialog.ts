export interface OpenDialogOptions {
  filter?: string
  defaultPath?: string
  multiple?: boolean
  directory?: boolean
}

export type SaveDialogOptions = Pick<OpenDialogOptions, 'filter' | 'defaultPath'>
