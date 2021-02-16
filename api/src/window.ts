import { invoke } from './tauri'
import { EventCallback, emit, listen } from './helpers/event'

interface WindowDef {
  label: string
}

declare global {
  interface Window {
    __TAURI__: {
      windows: WindowDef[],
      currentWindow: WindowDef
    };
  }
}

class TauriWindow {
  label: string
  constructor(label: string) {
    this.label = label
  }

  /**
   * Listen to an event emitted by the webview
   *
   * @param event the event name
   * @param handler the event handler callback
   * @param once unlisten after the first trigger if true
   */
  async listen<T>(event: string, handler: EventCallback<T>, once = false): Promise<void> {
    await listen(event, handler, once)
  }

  /**
  * emits an event to the webview
  *
  * @param event the event name
  * @param [payload] the event payload
  */
  async emit(event: string, payload?: string): Promise<void> {
    await emit(event, this.label, payload)
  }
}

function getTauriWindow(label: string = window.__TAURI__.currentWindow.label): TauriWindow | null {
  if (window.__TAURI__.windows.some(w => w.label === label)) {
    return new TauriWindow(label)
  } else {
    return null
  }
}

/**
 * Updates the window resizable flag.
 */
async function setResizable(resizable: boolean): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setTitle(title: string): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'setTitle',
      title
    }
  })
}

/**
 * Maximizes the window.
 */
async function maximize(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'maximize'
    }
  })
}

/**
 * Unmaximizes the window.
 */
async function unmaximize(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'unmaximize'
    }
  })
}

/**
 * Minimizes the window.
 */
async function minimize(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'minimize'
    }
  })
}

/**
 * Unminimizes the window.
 */
async function unminimize(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'unminimize'
    }
  })
}

/**
 * Sets the window visibility to true.
 */
async function show(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'show'
    }
  })
}

/**
 * Sets the window visibility to false.
 */
async function hide(): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setTransparent(transparent: boolean): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setDecorations(decorations: boolean): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setWidth(width: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setHeight(height: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function resize(width: number, height: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setMinSize(minWidth: number, minHeight: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setMaxSize(maxWidth: number, maxHeight: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setX(x: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setY(y: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setPosition(x: number, y: number): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setFullscreen(fullscreen: boolean): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
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
async function setIcon(icon: 'string' | number[]): Promise<void> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'setIcon',
      icon
    }
  })
}

export {
  TauriWindow,
  getTauriWindow,
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
