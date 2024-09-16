// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * A size represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalSize {
  type = 'Logical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }

  /**
   * Converts the logical size to a physical one.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const size = new LogicalSize(400, 500);
   * const physical = size.toPhysical(factor);
   * ```
   *
   * @since 2.0.0
   */
  toPhysical(scaleFactor: number): PhysicalSize {
    return new PhysicalSize(this.width * scaleFactor, this.height * scaleFactor)
  }
}

/**
 * A size represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalSize {
  type = 'Physical'
  width: number
  height: number

  constructor(width: number, height: number) {
    this.width = width
    this.height = height
  }

  /**
   * Converts the physical size to a logical one.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const size = await appWindow.innerSize();
   * const logical = size.toLogical(factor);
   * ```
   */
  toLogical(scaleFactor: number): LogicalSize {
    return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor)
  }
}

/**
 *  A position represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalPosition {
  type = 'Logical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }

  /**
   * Converts the logical position to a physical one.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const position = new LogicalPosition(400, 500);
   * const physical = position.toPhysical(factor);
   * ```
   *
   * @since 2.0.0
   */
  toPhysical(scaleFactor: number): PhysicalPosition {
    return new PhysicalPosition(this.x * scaleFactor, this.x * scaleFactor)
  }
}

/**
 *  A position represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalPosition {
  type = 'Physical'
  x: number
  y: number

  constructor(x: number, y: number) {
    this.x = x
    this.y = y
  }

  /**
   * Converts the physical position to a logical one.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const position = await appWindow.innerPosition();
   * const logical = position.toLogical(factor);
   * ```
   */
  toLogical(scaleFactor: number): LogicalPosition {
    return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor)
  }
}

export { LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize }
