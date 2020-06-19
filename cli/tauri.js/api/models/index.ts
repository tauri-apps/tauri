// http

export enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

export enum BodyType {
  Form = 1,
  File = 2,
  Auto = 3
}

export type Body = object | string | BinaryType

export type HttpVerb = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS' | 'CONNECT' | 'TRACE'

export interface HttpOptions {
  method: HttpVerb
  url: string
  headers?: Record<string, any>
  propertys?: Record<string, any>
  body?: Body
  followRedirects: boolean
  maxRedirections: boolean
  connectTimeout: number
  readTimeout: number
  timeout: number
  allowCompression: boolean
  responseType?: ResponseType
  bodyType: BodyType
}

export type PartialOptions = Omit<HttpOptions, 'method' | 'url'>

// events

export interface Event {
  type: string
  payload: unknown
}

export type EventCallback = (event: Event) => void

//
export interface OpenDialogOptions {
  filter?: string
  defaultPath?: string
  multiple?: boolean
  directory?: boolean
}

export type SaveDialogOptions = Pick<OpenDialogOptions, 'filter' | 'defaultPath'>

export enum BaseDirectory {
  Audio = 1,
  Cache,
  Config,
  Data,
  LocalData,
  Desktop,
  Document,
  Download,
  Executable,
  Font,
  Home,
  Picture,
  Public,
  Runtime,
  Template,
  Video,
  Resource,
  App,
}

export interface FsOptions {
  dir?: BaseDirectory
}

export interface FsFileOption {
  path: string
  contents: string
}

export interface FileEntry {
  path: string
  // TODO why not camelCase ?
  is_dir: boolean
  name: string
}
