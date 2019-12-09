const { inject } = require('../template')

module.exports = args => {
  return inject(args.directory, 'all', {
    force: args.force,
    logging: args.logging,
    tauriPath: args.tauriPath
  })
}
