import { inject } from '../template'

module.exports = (args: {
  directory: string
  force: false | 'conf' | 'template' | 'all'
  logging: boolean
  tauriPath?: string
}): boolean => {
  return inject(args.directory, 'all', {
    force: args.force,
    logging: args.logging,
    tauriPath: args.tauriPath
  })
}
