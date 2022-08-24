import { defineConfig } from 'vite'
import Unocss from 'unocss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { internalIpV4 } from 'internal-ip'

// https://vitejs.dev/config/
export default defineConfig(async ({ command, mode }) => {
  const host = await internalIpV4()
  return {
    plugins: [Unocss(), svelte()],
    build: {
      rollupOptions: {
        output: {
          entryFileNames: `assets/[name].js`,
          chunkFileNames: `assets/[name].js`,
          assetFileNames: `assets/[name].[ext]`
        }
      }
    },
    server: {
      port: 5173,
      strictPort: true,
      hmr: {
        protocol: 'ws',
        host,
        port: 5183
      },
      fs: {
        allow: ['.', '../../tooling/api/dist']
      }
    }
  }
})
