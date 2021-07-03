const readline = require('readline')

module.exports = {
  onMessage(cb) {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false
    })

    rl.on('line', function (line) {
      cb(line)
    })
  },
  write(message) {
    console.log(message)
  }
}
