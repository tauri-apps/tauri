import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { internalIpV4Sync } from 'internal-ip'

const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Vite optons tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    host: mobile ? "0.0.0.0" : false,
    port: 1420,
    strictPort: true,
    hmr: mobile ? {
      protocol: 'ws',
      host: internalIpV4Sync(),
      port: 1421
    } : undefined,
  },
})
