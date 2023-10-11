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
   * import { getCurrent } from '@tauri-apps/api/window';
   * const appWindow = getCurrent();
   * const factor = await appWindow.scaleFactor();
   * const size = await appWindow.innerSize();
   * const logical = size.toLogical(factor);
   * ```
   *  */
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
   * import { getCurrent } from '@tauri-apps/api/window';
   * const appWindow = getCurrent();
   * const factor = await appWindow.scaleFactor();
   * const position = await appWindow.innerPosition();
   * const logical = position.toLogical(factor);
   * ```
   * */
  toLogical(scaleFactor: number): LogicalPosition {
    return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor)
  }
}

export { LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize }
