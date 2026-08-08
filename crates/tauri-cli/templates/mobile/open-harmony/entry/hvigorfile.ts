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
        const properties = hvigor.getParameter().getProperties();
        // Priority: TAURI_OHOS_TARGET env var > hvigor `target` property >
        // aarch64 (the default device architecture; override to x86_64 for
        // the OpenHarmony emulator).
        const target = process.env.TAURI_OHOS_TARGET || properties.target || "aarch64";
        execFileSync(`{{tauri-binary}}`,
          [{{quote-and-join tauri-binary-args}}, "--target", target.toString()], {
            cwd: resolve(__dirname, "{{root-dir-rel}}"),
            stdio: "inherit",
          });
      }

      node.getTaskByName('default@ConfigureCmake')!.afterRun(buildRustCode);
    }
  }
}
