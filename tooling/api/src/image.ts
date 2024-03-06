// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Resource, invoke } from './core'

/** An RGBA Image in row-major order from top to bottom. */
export class Image extends Resource {
  private constructor(rid: number) {
    super(rid)
  }

  /** Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height. */
  static async new(
    rgba: number[] | Uint8Array | ArrayBuffer,
    width: number,
    height: number
  ): Promise<Image> {
    return invoke<number>('plugin:image|new', {
      rgba: transformImage(rgba),
      width,
      height
    }).then((rid) => new Image(rid))
  }

  /**
   * Creates a new image using the provided bytes by inferring the file format.
   * If the format is known, prefer [@link Image.fromPngBytes] or [@link Image.fromIcoBytes].
   *
   * Only `ico` and `png` are supported (based on activated feature flag).
   *
   * Note that you need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  static async fromBytes(
    bytes: number[] | Uint8Array | ArrayBuffer
  ): Promise<Image> {
    return invoke<number>('plugin:image|from_bytes', {
      bytes: transformImage(bytes)
    }).then((rid) => new Image(rid))
  }

  /**
   * Creates a new image using the provided png bytes.
   *
   * Note that you need the `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  static async fromPngBytes(
    bytes: number[] | Uint8Array | ArrayBuffer
  ): Promise<Image> {
    return invoke<number>('plugin:image|from_png_bytes', {
      bytes: transformImage(bytes)
    }).then((rid) => new Image(rid))
  }

  /**
   * Creates a new image using the provided ico bytes.
   *
   * Note that you need the `image-ico` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-ico"] }
   * ```
   */
  static async fromIcoBytes(
    bytes: number[] | Uint8Array | ArrayBuffer
  ): Promise<Image> {
    return invoke<number>('plugin:image|from_ico_bytes', {
      bytes: transformImage(bytes)
    }).then((rid) => new Image(rid))
  }

  /**
   * Creates a new image using the provided path.
   *
   * Only `ico` and `png` are supported (based on activated feature flag).
   *
   * Note that you need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  static async fromPath(path: string): Promise<Image> {
    return invoke<number>('plugin:image|from_path', { path }).then(
      (rid) => new Image(rid)
    )
  }

  /** Returns the RGBA data for this image, in row-major order from top to bottom.  */
  async rgba(): Promise<ArrayBuffer | number[]> {
    return invoke<ArrayBuffer | number[]>('plugin:image|rgba', {
      rid: this.rid
    })
  }

  /** Returns the width of this image.  */
  async width() {
    return invoke<number>('plugin:image|width', { rid: this.rid })
  }

  /** Returns the height of this image. */
  async height() {
    return invoke<number>('plugin:image|height', { rid: this.rid })
  }
}

/**
 * Transforms image from various types into a type acceptable by Rust. Intended for internal use only.
 *
 * @ignore
 */
export function transformImage<T>(
  image: string | Image | Uint8Array | ArrayBuffer | number[] | null
): T {
  const ret =
    image == null
      ? null
      : typeof image === 'string'
        ? image
        : image instanceof Uint8Array
          ? Array.from(image)
          : image instanceof ArrayBuffer
            ? Array.from(new Uint8Array(image))
            : image instanceof Image
              ? image.rid
              : image

  return ret as T
}
