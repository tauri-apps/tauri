import init, * as wasm from 'wasm'

function initialize() {
  return init('wasm/wasm_bg.wasm')
}

export const NAME = 'WEB'

/**
 * Greets someone.
 * @param {string} name
 * @returns
 */
export async function greet(name) {
  return initialize().then(() => wasm.greet(name))
}
