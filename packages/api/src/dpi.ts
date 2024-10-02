// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * A size represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalSize {
  type = 'Logical' as const
  width: number
  height: number

  constructor(width: number, height: number)
  constructor(objcet: { Logical: { width: number; height: number } })
  constructor(objcet: { width: number; height: number })
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

  /**
   * Converts this size into IPC-compatible value, so it can be
   * deserialized correctly on the Rust side using `tauri::LogicalSize` struct.
   * @example
   * ```typescript
   * import { LogicalSize } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const size = new LogicalSize(400, 500);
   * await invoke("do_something_with_size", { size: size.toIPC() })
   * ```
   *
   * @since 2.0.0
   */
  toIPC() {
    return {
      width: this.width,
      height: this.height
    }
  }

  /**
   * Converts this size into JSON value, that can be deserialized
   * correctly on the Rust side using `tauri::LogicalSize` struct.
   * @example
   * ```typescript
   * import { LogicalSize } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const size = new LogicalSize(400, 500);
   * await invoke("do_something_with_size", { size: size.toJSON() })
   * ```
   *
   * @since 2.0.0
   */
  toJSON() {
    return this.toIPC()
  }
}

/**
 * A size represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalSize {
  type = 'Physical' as const
  width: number
  height: number

  constructor(width: number, height: number)
  constructor(objcet: { Physical: { width: number; height: number } })
  constructor(objcet: { width: number; height: number })
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

  /**
   * Converts this size into IPC-compatible value, so it can be
   * deserialized correctly on the Rust side using `tauri::PhysicalSize` struct.
   * @example
   * ```typescript
   * import { PhysicalSize } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const size = new PhysicalSize(400, 500);
   * await invoke("do_something_with_size", { size: size.toIPC() })
   * ```
   *
   * @since 2.0.0
   */
  toIPC() {
    return {
      width: this.width,
      height: this.height
    }
  }

  /**
   * Converts this size into JSON value, that can be deserialized
   * correctly on the Rust side using `tauri::PhysicalSize` struct.
   * @example
   * ```typescript
   * import { PhysicalSize } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const size = new PhysicalSize(400, 500);
   * await invoke("do_something_with_size", { size: size.toJSON() })
   * ```
   *
   * @since 2.0.0
   */
  toJSON() {
    return this.toIPC()
  }
}

/**
 *  A position represented in logical pixels.
 *
 * @since 2.0.0
 */
class LogicalPosition {
  type = 'Logical' as const
  x: number
  y: number

  constructor(x: number, y: number)
  constructor(objcet: { Logical: { x: number; y: number } })
  constructor(objcet: { x: number; y: number })
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
    return new PhysicalPosition(this.x * scaleFactor, this.x * scaleFactor)
  }

  /**
   * Converts this position into IPC-compatible value, so it can be
   * deserialized correctly on the Rust side using `tauri::LogicalPosition` struct.
   * @example
   * ```typescript
   * import { LogicalPosition } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const position = new LogicalPosition(400, 500);
   * await invoke("do_something_with_position", { position: position.toIPC() })
   * ```
   *
   * @since 2.0.0
   */
  toIPC() {
    return {
      x: this.x,
      y: this.y
    }
  }

  /**
   * Converts this position into JSON value, that can be deserialized
   * correctly on the Rust side using `tauri::LogicalPosition` struct.
   * @example
   * ```typescript
   * import { LogicalPosition } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const position = new LogicalPosition(400, 500);
   * await invoke("do_something_with_position", { position: position.toJSON() })
   * ```
   *
   * @since 2.0.0
   */
  toJSON() {
    return this.toIPC()
  }
}

/**
 *  A position represented in physical pixels.
 *
 * @since 2.0.0
 */
class PhysicalPosition {
  type = 'Physical' as const
  x: number
  y: number

  constructor(x: number, y: number)
  constructor(objcet: { Physical: { x: number; y: number } })
  constructor(objcet: { x: number; y: number })
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
   * Converts this position into IPC-compatible value, so it can be
   * deserialized correctly on the Rust side using `tauri::PhysicalPosition` struct.
   * @example
   * ```typescript
   * import { PhysicalPosition } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const position = new PhysicalPosition(400, 500);
   * await invoke("do_something_with_position", { position: position.toIPC() })
   * ```
   *
   * @since 2.0.0
   */
  toIPC() {
    return {
      x: this.x,
      y: this.y
    }
  }

  /**
   * Converts this position into JSON value, that can be deserialized
   * correctly on the Rust side using `tauri::PhysicalPosition` struct.
   * @example
   * ```typescript
   * import { PhysicalPosition } from '@tauri-apps/api/dpi';
   * import { invoke } from '@tauri-apps/api/core';
   *
   * const position = new PhysicalPosition(400, 500);
   * await invoke("do_something_with_position", { position: position.toJSON() })
   * ```
   *
   * @since 2.0.0
   */
  toJSON() {
    return this.toIPC()
  }
}

export { LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize }
