import { invoke } from './tauri'

/**
 * Updates the window resizable flag.
 */
function setResizable(resizable: boolean): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setResizable',
      resizable
    }
  })
}

/**
 * sets the window title
 *
 * @param title the new title
 */
function setTitle(title: string): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setTitle',
      title
    }
  })
}

/**
 * Maximizes the window.
 */
function maximize(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'maximize'
    }
  })
}

/**
 * Unmaximizes the window.
 */
function unmaximize(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'unmaximize'
    }
  })
}

/**
 * Minimizes the window.
 */
function minimize(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'minimize'
    }
  })
}

/**
 * Unminimizes the window.
 */
function unminimize(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'unminimize'
    }
  })
}

/**
 * Sets the window visibility to true.
 */
function show(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'show'
    }
  })
}

/**
 * Sets the window visibility to false.
 */
function hide(): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'hide'
    }
  })
}

/**
 * Sets the window transparent flag.
 *
 * @param {boolean} transparent whether the the window should be transparent or not
 */
function setTransparent(transparent: boolean): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setTransparent',
      transparent
    }
  })
}

/**
 * Whether the window should have borders and bars.
 *
 * @param {boolean} decorations whether the window should have borders and bars
 */
function setDecorations(decorations: boolean): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setDecorations',
      decorations
    }
  })
}

/**
 * Whether the window should always be on top of other windows.
 *
 * @param {boolean} alwaysOnTop whether the window should always be on top of other windows or not
 */
function setAlwaysOnTop(alwaysOnTop: boolean): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setAlwaysOnTop',
      alwaysOnTop
    }
  })
}

/**
 * Sets the window width.
 *
 * @param {number} width the new window width
 */
function setWidth(width: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setWidth',
      width
    }
  })
}

/**
 * Sets the window height.
 *
 * @param {number} height the new window height
 */
function setHeight(height: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setHeight',
      height
    }
  })
}

/**
 * Resizes the window.
 *
 * @param {number} width the new window width
 * @param {number} height the new window height
 */
function resize(width: number, height: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'resize',
      width,
      height
    }
  })
}

/**
 * Sets the window min size.
 *
 * @param {number} minWidth the new window min width
 * @param {number} minHeight the new window min height
 */
function setMinSize(minWidth: number, minHeight: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setMinSize',
      minWidth,
      minHeight
    }
  })
}

/**
 * Sets the window max size.
 *
 * @param {number} maxWidth the new window max width
 * @param {number} maxHeight the new window max height
 */
function setMaxSize(maxWidth: number, maxHeight: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setMaxSize',
      maxWidth,
      maxHeight
    }
  })
}

/**
 * Sets the window x position.
 *
 * @param {number} x the new window x position
 */
function setX(x: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setX',
      x
    }
  })
}

/**
 * Sets the window y position.
 *
 * @param {number} y the new window y position
 */
function setY(y: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setY',
      y
    }
  })
}

/**
 * Sets the window position.
 *
 * @param {number} x the new window x position
 * @param {number} y the new window y position
 */
function setPosition(x: number, y: number): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setPosition',
      x,
      y
    }
  })
}

/**
 * Sets the window fullscreen state.
 *
 * @param {boolean} fullscreen whether the window should go to fullscreen or not
 */
function setFullscreen(fullscreen: boolean): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setFullscreen',
      fullscreen
    }
  })
}

/**
 * Sets the window icon
 *
 * @param {string | number[]} icon icon bytes or path to the icon file
 */
function setIcon(icon: 'string' | number[]): void {
  invoke({
    module: 'Window',
    message: {
      cmd: 'setIcon',
      icon
    }
  })
}

export {
  setResizable,
  setTitle,
  maximize,
  unmaximize,
  minimize,
  unminimize,
  show,
  hide,
  setTransparent,
  setDecorations,
  setAlwaysOnTop,
  setWidth,
  setHeight,
  resize,
  setMinSize,
  setMaxSize,
  setX,
  setY,
  setPosition,
  setFullscreen,
  setIcon
}
