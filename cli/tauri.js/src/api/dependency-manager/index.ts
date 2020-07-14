import logger from '../../helpers/logger'
import * as cargoCommands from './cargo-commands'

const log = logger('dependency:manager')

module.exports = {
  async installDependencies() {
    log('Installing missing dependencies...')
    await cargoCommands.install()
  },
  async updateDependencies() {
    log('Updating dependencies...')
    await cargoCommands.update()
  }
}
