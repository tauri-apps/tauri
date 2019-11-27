const
  ms = require('ms')
const chalk = require('chalk')

let prevTime

module.exports = function (banner, color = 'green') {
  return function (msg) {
    const
      curr = +new Date()
    const diff = curr - (prevTime || curr)

    prevTime = curr

    if (msg) {
      console.log(` ${chalk[color](banner)} ${msg} ${chalk.green(`+${ms(diff)}`)}`)
    } else {
      console.log()
    }
  }
}
