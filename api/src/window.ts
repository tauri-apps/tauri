import { invoke } from './tauri'
import { EventCallback, emit, listen } from './helpers/event'

interface WindowDef {
  label: string
}

declare global {
  interface Window {
    __TAURI__: {
      __windows: WindowDef[]
      __currentWindow: WindowDef
    }
  }
}

function getCurrentWindow(): WindowDef {
  return window.__TAURI__.__currentWindow
}

function getWindows(): WindowDef[] {
  return window.__TAURI__.__windows
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
  async listen<T>(
    event: string,
    handler: EventCallback<T>,
    once = false
  ): Promise<void> {
    return listen(event, handler, once)
  }

  /**
   * emits an event to the webview
   *
   * @param event the event name
   * @param [payload] the event payload
   */
  async emit(event: string, payload?: string): Promise<void> {
    return emit(event, this.label, payload)
  }
}

function getTauriWindow(
  label: string = getCurrentWindow().label
): TauriWindow | null {
  if (getWindows().some((w) => w.label === label)) {
    return new TauriWindow(label)
  } else {
    return null
  }
}

class WindowManager {
  /**
   * Updates the window resizable flag.
   */
  async setResizable(resizable: boolean): Promise<void> {
    return invoke({
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
  async setTitle(title: string): Promise<void> {
    return invoke({
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
  async maximize(): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'maximize'
      }
    })
  }

  /**
   * Unmaximizes the window.
   */
  async unmaximize(): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'unmaximize'
      }
    })
  }

  /**
   * Minimizes the window.
   */
  async minimize(): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'minimize'
      }
    })
  }

  /**
   * Unminimizes the window.
   */
  async unminimize(): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'unminimize'
      }
    })
  }

  /**
   * Sets the window visibility to true.
   */
  async show(): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'show'
      }
    })
  }

  /**
   * Sets the window visibility to false.
   */
  async hide(): Promise<void> {
    return invoke({
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
  async setTransparent(transparent: boolean): Promise<void> {
    return invoke({
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
  async setDecorations(decorations: boolean): Promise<void> {
    return invoke({
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
  async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
    return invoke({
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
  async setWidth(width: number): Promise<void> {
    return invoke({
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
  async setHeight(height: number): Promise<void> {
    return invoke({
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
  async resize(width: number, height: number): Promise<void> {
    return invoke({
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
  async setMinSize(minWidth: number, minHeight: number): Promise<void> {
    return invoke({
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
  async setMaxSize(maxWidth: number, maxHeight: number): Promise<void> {
    return invoke({
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
  async setX(x: number): Promise<void> {
    return invoke({
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
  async setY(y: number): Promise<void> {
    return invoke({
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
  async setPosition(x: number, y: number): Promise<void> {
    return invoke({
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
  async setFullscreen(fullscreen: boolean): Promise<void> {
    return invoke({
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
  async setIcon(icon: 'string' | number[]): Promise<void> {
    return invoke({
      __tauriModule: 'Window',
      message: {
        cmd: 'setIcon',
        icon
      }
    })
  }
}

const manager = new WindowManager()

export interface WindowOptions {
  url?: 'app' | string
  x?: number
  y?: number
  width?: number
  height?: number
  minWidth?: number
  minHeight?: number
  maxWidth?: number
  maxHeight?: number
  resizable?: boolean
  title?: string
  fullscreen?: boolean
  transparent?: boolean
  maximized?: boolean
  visible?: boolean
  decorations?: boolean
  alwaysOnTop?: boolean
}

async function createWindow(
  label: string,
  options: WindowOptions = {}
): Promise<TauriWindow> {
  await invoke({
    __tauriModule: 'Window',
    message: {
      cmd: 'createWebview',
      options: {
        label,
        ...options
      }
    }
  })
  return new TauriWindow(label)
}

export {
  TauriWindow,
  getTauriWindow,
  getCurrentWindow,
  getWindows,
  manager,
  createWindow
}
