import chalk from "chalk";
import ms from "ms";

let prevTime: number;

export default (banner: string, color: chalk.Chalk = chalk.green) => {
  return (msg?: string) => {
    const curr = +new Date();
    const diff = curr - (prevTime || curr);

    prevTime = curr;

    if (msg) {
      console.log(
        // TODO: proper typings for color and banner
        // eslint-disable-next-line @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-unsafe-call
        ` ${color(String(banner))} ${msg} ${chalk.green(`+${ms(diff)}`)}`
      );
    } else {
      console.log();
    }
  };
};
