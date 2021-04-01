import execa from "execa";

export const shell = async (command: string, args?: string[], options?: {}) => {
  return execa(command, args, { stdio: "inherit", ...options }).catch((err) => {
    console.error(err);
    throw new Error(
      `Caught an error running the command: ${command} ${
        args ? args.join(" ") : ""
      }`
    );
  });
};
