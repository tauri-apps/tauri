import execa from "execa";

export const shell = async (
  command: string,
  args?: string[],
  options?: execa.Options
) => {
  try {
    if (options && options.shell === true) {
      return await execa([command, ...(!args ? [] : args)].join(" "), {
        stdio: "inherit",
        cwd: process.cwd(),
        env: process.env,
        ...options,
      });
    } else {
      return await execa(command, args, {
        stdio: "inherit",
        cwd: process.cwd(),
        env: process.env,
        ...options,
      });
    }
  } catch (error) {
    console.error("Error with command: %s", command);
    throw new Error(error);
  }
};
