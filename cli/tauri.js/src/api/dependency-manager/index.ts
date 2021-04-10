// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import logger from '../../helpers/logger'
import * as rust from './rust'
import * as cargoCommands from './cargo-commands'
import * as cargoCrates from './cargo-crates'
import * as npmPackages from './npm-packages'

const log = logger('dependency:manager')

module.exports = {
  async installDependencies() {
    log('Installing missing dependencies...')
    rust.install()
    await cargoCommands.install()
    await cargoCrates.install()
    await npmPackages.install()
  },
  async updateDependencies() {
    log('Updating dependencies...')
    rust.update()
    await cargoCommands.update()
    await cargoCrates.update()
    await npmPackages.update()
  }
}
