import { appTasks } from '@ohos/hvigor-ohos-plugin';
import { hvigor, HvigorPlugin, HvigorNode } from '@ohos/hvigor';
import { execSync } from 'child_process';
import { resolve } from 'path';

export default {
    system: appTasks,  /* Built-in plugin of Hvigor. It cannot be modified. */
    plugins:[tauriPlugin()]         /* Custom plugin to extend the functionality of Hvigor. */
}

function tauriPlugin(): HvigorPlugin {
    return {
        pluginId: 'tauri',
        apply(node: HvigorNode) {
            const properties = hvigor.getParameter().getProperties();
            const target = properties.target || "aarch64";
            execSync(`{{tauri-binary}}`, [`{{quote-and-join tauri-binary-args}}`, "--target", target], {
                cwd: resolve(__dirname, "{{root-dir-rel}}"),
                stdio: "inherit",
                shell: true,
            });
        }
    }
}
