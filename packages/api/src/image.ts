// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Resource, invoke } from './core'

/// Image dimensions type.
export interface ImageSize {
  /// Image width.
  width: number
  /// Image height.
  height: number
}

/** An RGBA Image in row-major order from top to bottom. */
export class Image extends Resource {
  /**
   * Creates an Image from a resource ID. For internal use only.
   *
   * @ignore
   */
  constructor(rid: number) {
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
  async rgba(): Promise<Uint8Array> {
    return invoke<number[]>('plugin:image|rgba', {
      rid: this.rid
    }).then((buffer) => new Uint8Array(buffer))
  }

  /** Returns the size of this image.  */
  async size(): Promise<ImageSize> {
    return invoke<ImageSize>('plugin:image|size', { rid: this.rid })
  }
}

/**
 * Transforms image from various types into a type acceptable by Rust.
 *
 * See [tauri::image::JsImage](https://docs.rs/tauri/2/tauri/image/enum.JsImage.html) for more information.
 * Note the API signature is not stable and might change.
 */
export function transformImage<T>(
  image: string | Image | Uint8Array | ArrayBuffer | number[] | null
): T {
  const ret =
    image == null
      ? null
      : typeof image === 'string'
        ? image
        : image instanceof Image
          ? image.rid
          : image

  return ret as T
}
