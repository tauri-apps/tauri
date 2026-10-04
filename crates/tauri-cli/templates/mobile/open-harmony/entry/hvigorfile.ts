import { hapTasks } from '@ohos/hvigor-ohos-plugin';
import { hvigor, HvigorPlugin, HvigorNode, HvigorTask } from '@ohos/hvigor';
import { execFileSync } from 'child_process';
import { resolve } from 'path';

export default {
  system: hapTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
  plugins:[tauriPlugin()]         /* Custom plugin to extend the functionality of Hvigor. */
}

function tauriPlugin(): HvigorPlugin {
  return {
    pluginId: 'tauri',
    apply(node: HvigorNode) {
      const buildRustCode = () => {
        // `ohos build` has already staged Rust libraries. Keep this hook for dev/IDE rebuilds.
        if (process.env.TAURI_OHOS_SKIP_RUST_BUILD === "1") return;
        const properties = hvigor.getParameter().getProperties();
        const target = properties.target || process.env.TAURI_OHOS_TARGET || "aarch64";
        const args = [{{quote-and-join tauri-binary-args}}, "--target", target.toString()];
        if (process.env.TAURI_OHOS_PROFILE === "release") args.push("--release");
        execFileSync(`{{tauri-binary}}`,
          args, {
            cwd: resolve(__dirname, "{{root-dir-rel}}"),
            stdio: "inherit",
          });
      }

      node.getTaskByName('default@ConfigureCmake')!.afterRun(buildRustCode);
    }
  }
}
