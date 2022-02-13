const { run } = require('./index')

module.exports.run = (args, binName) => {
  return new Promise((resolve, reject) => {
    run(args, binName, res => {
      if (res instanceof Error) {
        reject(res)
      } else {
        resolve(res)
      }
    })
  })
}
