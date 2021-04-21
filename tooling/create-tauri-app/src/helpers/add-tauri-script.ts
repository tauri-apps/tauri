import { readFileSync, writeFileSync } from "fs";
import { join } from "path";

export async function addTauriScript(appDirectory: string) {
  const pkgPath = join(appDirectory, "package.json");
  const rawData = readFileSync(pkgPath, "utf8");
  const pkg = JSON.parse(rawData) as {
    scripts: {
      tauri: string;
    };
  };

  pkg.scripts.tauri = "tauri";

  writeFileSync(pkgPath, JSON.stringify(pkg, undefined, 2));
}
