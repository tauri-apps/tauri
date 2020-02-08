import chalk from 'chalk'
import ms from 'ms'

let prevTime: number

export default (banner: string, color: string = 'green') => {
  return (msg?: string) => {
    const curr = +new Date()
    const diff = curr - (prevTime || curr)

    prevTime = curr

    if (msg) {
      console.log(
        // TODO: proper typings for color and banner
        // @ts-ignore
        // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
        ` ${chalk[String(color)](String(banner))} ${msg} ${chalk.green(`+${ms(diff)}`)}`
      )
    } else {
      console.log()
    }
  }
}
