// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { SERIALIZE_TO_IPC_FN } from './core'

/**
 * A size represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalSize {
  readonly type = 'Logical'
  width: number
  height: number

  constructor(width: number, height: number)
  constructor(object: { Logical: { width: number; height: number } })
  constructor(object: { width: number; height: number })
  constructor(
    ...args:
      | [number, number]
      | [{ width: number; height: number }]
      | [{ Logical: { width: number; height: number } }]
  ) {
    if (args.length === 1) {
      if ('Logical' in args[0]) {
        this.width = args[0].Logical.width
        this.height = args[0].Logical.height
      } else {
        this.width = args[0].width
        this.height = args[0].height
      }
    } else {
      this.width = args[0]
      this.height = args[1]
    }
  }

  /**
   * Converts the logical size to a physical one.
   * @example
   * ```typescript
   * import { LogicalSize } from '@tauri-apps/api/dpi';
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   *
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

  [SERIALIZE_TO_IPC_FN]() {
    return {
      width: this.width,
      height: this.height
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

/**
 * A size represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalSize {
  readonly type = 'Physical'
  width: number
  height: number

  constructor(width: number, height: number)
  constructor(object: { Physical: { width: number; height: number } })
  constructor(object: { width: number; height: number })
  constructor(
    ...args:
      | [number, number]
      | [{ width: number; height: number }]
      | [{ Physical: { width: number; height: number } }]
  ) {
    if (args.length === 1) {
      if ('Physical' in args[0]) {
        this.width = args[0].Physical.width
        this.height = args[0].Physical.height
      } else {
        this.width = args[0].width
        this.height = args[0].height
      }
    } else {
      this.width = args[0]
      this.height = args[1]
    }
  }

  /**
   * Converts the physical size to a logical one.
   * @example
   * ```typescript
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const size = await appWindow.innerSize(); // PhysicalSize
   * const logical = size.toLogical(factor);
   * ```
   */
  toLogical(scaleFactor: number): LogicalSize {
    return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor)
  }

  [SERIALIZE_TO_IPC_FN]() {
    return {
      width: this.width,
      height: this.height
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

/**
 * A size represented either in physical or in logical pixels.
 *
 * This type is basically a union type of {@linkcode LogicalSize} and {@linkcode PhysicalSize}
 * but comes in handy when using `tauri::Size` in Rust as an argument to a command, as this class
 * automatically serializes into a valid format so it can be deserialized correctly into `tauri::Size`
 *
 * So instead of
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * import { LogicalSize, PhysicalSize } from '@tauri-apps/api/dpi';
 *
 * const size: LogicalSize | PhysicalSize = someFunction(); // where someFunction returns either LogicalSize or PhysicalSize
 * const validSize = size instanceof LogicalSize
 *   ? { Logical: { width: size.width, height: size.height } }
 *   : { Physical: { width: size.width, height: size.height } }
 * await invoke("do_something_with_size", { size: validSize });
 * ```
 *
 * You can just use {@linkcode Size}
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * import { LogicalSize, PhysicalSize, Size } from '@tauri-apps/api/dpi';
 *
 * const size: LogicalSize | PhysicalSize = someFunction(); // where someFunction returns either LogicalSize or PhysicalSize
 * const validSize = new Size(size);
 * await invoke("do_something_with_size", { size: validSize });
 * ```
 *
 * @since 2.1.0
 */
class Size {
  size: LogicalSize | PhysicalSize

  constructor(size: LogicalSize | PhysicalSize) {
    this.size = size
  }

  toLogical(scaleFactor: number): LogicalSize {
    return this.size instanceof LogicalSize
      ? this.size
      : this.size.toLogical(scaleFactor)
  }

  toPhysical(scaleFactor: number): PhysicalSize {
    return this.size instanceof PhysicalSize
      ? this.size
      : this.size.toPhysical(scaleFactor)
  }

  [SERIALIZE_TO_IPC_FN]() {
    return {
      [`${this.size.type}`]: {
        width: this.size.width,
        height: this.size.height
      }
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

/**
 *  A position represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalPosition {
  readonly type = 'Logical'
  x: number
  y: number

  constructor(x: number, y: number)
  constructor(object: { Logical: { x: number; y: number } })
  constructor(object: { x: number; y: number })
  constructor(
    ...args:
      | [number, number]
      | [{ x: number; y: number }]
      | [{ Logical: { x: number; y: number } }]
  ) {
    if (args.length === 1) {
      if ('Logical' in args[0]) {
        this.x = args[0].Logical.x
        this.y = args[0].Logical.y
      } else {
        this.x = args[0].x
        this.y = args[0].y
      }
    } else {
      this.x = args[0]
      this.y = args[1]
    }
  }

  /**
   * Converts the logical position to a physical one.
   * @example
   * ```typescript
   * import { LogicalPosition } from '@tauri-apps/api/dpi';
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   *
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const position = new LogicalPosition(400, 500);
   * const physical = position.toPhysical(factor);
   * ```
   *
   * @since 2.0.0
   */
  toPhysical(scaleFactor: number): PhysicalPosition {
    return new PhysicalPosition(this.x * scaleFactor, this.y * scaleFactor)
  }

  [SERIALIZE_TO_IPC_FN]() {
    return {
      x: this.x,
      y: this.y
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

/**
 *  A position represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalPosition {
  readonly type = 'Physical'
  x: number
  y: number

  constructor(x: number, y: number)
  constructor(object: { Physical: { x: number; y: number } })
  constructor(object: { x: number; y: number })
  constructor(
    ...args:
      | [number, number]
      | [{ x: number; y: number }]
      | [{ Physical: { x: number; y: number } }]
  ) {
    if (args.length === 1) {
      if ('Physical' in args[0]) {
        this.x = args[0].Physical.x
        this.y = args[0].Physical.y
      } else {
        this.x = args[0].x
        this.y = args[0].y
      }
    } else {
      this.x = args[0]
      this.y = args[1]
    }
  }

  /**
   * Converts the physical position to a logical one.
   * @example
   * ```typescript
   * import { PhysicalPosition } from '@tauri-apps/api/dpi';
   * import { getCurrentWindow } from '@tauri-apps/api/window';
   *
   * const appWindow = getCurrentWindow();
   * const factor = await appWindow.scaleFactor();
   * const position = new PhysicalPosition(400, 500);
   * const physical = position.toLogical(factor);
   * ```
   *
   * @since 2.0.0
   */
  toLogical(scaleFactor: number): LogicalPosition {
    return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor)
  }

  [SERIALIZE_TO_IPC_FN]() {
    return {
      x: this.x,
      y: this.y
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

/**
 * A position represented either in physical or in logical pixels.
 *
 * This type is basically a union type of {@linkcode LogicalSize} and {@linkcode PhysicalSize}
 * but comes in handy when using `tauri::Position` in Rust as an argument to a command, as this class
 * automatically serializes into a valid format so it can be deserialized correctly into `tauri::Position`
 *
 * So instead of
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * import { LogicalPosition, PhysicalPosition } from '@tauri-apps/api/dpi';
 *
 * const position: LogicalPosition | PhysicalPosition = someFunction(); // where someFunction returns either LogicalPosition or PhysicalPosition
 * const validPosition = position instanceof LogicalPosition
 *   ? { Logical: { x: position.x, y: position.y } }
 *   : { Physical: { x: position.x, y: position.y } }
 * await invoke("do_something_with_position", { position: validPosition });
 * ```
 *
 * You can just use {@linkcode Position}
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * import { LogicalPosition, PhysicalPosition, Position } from '@tauri-apps/api/dpi';
 *
 * const position: LogicalPosition | PhysicalPosition = someFunction(); // where someFunction returns either LogicalPosition or PhysicalPosition
 * const validPosition = new Position(position);
 * await invoke("do_something_with_position", { position: validPosition });
 * ```
 *
 * @since 2.1.0
 */
class Position {
  position: LogicalPosition | PhysicalPosition

  constructor(position: LogicalPosition | PhysicalPosition) {
    this.position = position
  }

  toLogical(scaleFactor: number): LogicalPosition {
    return this.position instanceof LogicalPosition
      ? this.position
      : this.position.toLogical(scaleFactor)
  }

  toPhysical(scaleFactor: number): PhysicalPosition {
    return this.position instanceof PhysicalPosition
      ? this.position
      : this.position.toPhysical(scaleFactor)
  }

  [SERIALIZE_TO_IPC_FN]() {
    return {
      [`${this.position.type}`]: {
        x: this.position.x,
        y: this.position.y
      }
    }
  }

  toJSON() {
    // eslint-disable-next-line security/detect-object-injection
    return this[SERIALIZE_TO_IPC_FN]()
  }
}

export {
  LogicalPosition,
  LogicalSize,
  Size,
  PhysicalPosition,
  PhysicalSize,
  Position
}
