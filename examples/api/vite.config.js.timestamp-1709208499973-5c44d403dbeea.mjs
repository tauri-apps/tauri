// vite.config.js
import { defineConfig } from "file:///home/lucas/projects/tauri/tauri/examples/api/node_modules/vite/dist/node/index.js";
import Unocss from "file:///home/lucas/projects/tauri/tauri/examples/api/node_modules/unocss/dist/vite.mjs";
import { svelte } from "file:///home/lucas/projects/tauri/tauri/examples/api/node_modules/@sveltejs/vite-plugin-svelte/src/index.js";
import { internalIpV4Sync } from "file:///home/lucas/projects/tauri/tauri/examples/api/node_modules/internal-ip/index.js";
var mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);
var vite_config_default = defineConfig({
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
  // Vite optons tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    host: mobile ? "0.0.0.0" : false,
    port: 1420,
    strictPort: true,
    hmr: mobile ? {
      protocol: "ws",
      host: internalIpV4Sync(),
      port: 1421
    } : void 0,
    fs: {
      allow: [".", "../../tooling/api/dist"]
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9sdWNhcy9wcm9qZWN0cy90YXVyaS90YXVyaS9leGFtcGxlcy9hcGlcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIi9ob21lL2x1Y2FzL3Byb2plY3RzL3RhdXJpL3RhdXJpL2V4YW1wbGVzL2FwaS92aXRlLmNvbmZpZy5qc1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vaG9tZS9sdWNhcy9wcm9qZWN0cy90YXVyaS90YXVyaS9leGFtcGxlcy9hcGkvdml0ZS5jb25maWcuanNcIjsvLyBDb3B5cmlnaHQgMjAxOS0yMDIzIFRhdXJpIFByb2dyYW1tZSB3aXRoaW4gVGhlIENvbW1vbnMgQ29uc2VydmFuY3lcbi8vIFNQRFgtTGljZW5zZS1JZGVudGlmaWVyOiBBcGFjaGUtMi4wXG4vLyBTUERYLUxpY2Vuc2UtSWRlbnRpZmllcjogTUlUXG5cbmltcG9ydCB7IGRlZmluZUNvbmZpZyB9IGZyb20gJ3ZpdGUnXG5pbXBvcnQgVW5vY3NzIGZyb20gJ3Vub2Nzcy92aXRlJ1xuaW1wb3J0IHsgc3ZlbHRlIH0gZnJvbSAnQHN2ZWx0ZWpzL3ZpdGUtcGx1Z2luLXN2ZWx0ZSdcbmltcG9ydCB7IGludGVybmFsSXBWNFN5bmMgfSBmcm9tICdpbnRlcm5hbC1pcCdcblxuY29uc3QgbW9iaWxlID0gISEvYW5kcm9pZHxpb3MvLmV4ZWMocHJvY2Vzcy5lbnYuVEFVUklfRU5WX1BMQVRGT1JNKVxuXG4vLyBodHRwczovL3ZpdGVqcy5kZXYvY29uZmlnL1xuZXhwb3J0IGRlZmF1bHQgZGVmaW5lQ29uZmlnKHtcbiAgcGx1Z2luczogW1Vub2NzcygpLCBzdmVsdGUoKV0sXG4gIGJ1aWxkOiB7XG4gICAgcm9sbHVwT3B0aW9uczoge1xuICAgICAgb3V0cHV0OiB7XG4gICAgICAgIGVudHJ5RmlsZU5hbWVzOiBgYXNzZXRzL1tuYW1lXS5qc2AsXG4gICAgICAgIGNodW5rRmlsZU5hbWVzOiBgYXNzZXRzL1tuYW1lXS5qc2AsXG4gICAgICAgIGFzc2V0RmlsZU5hbWVzOiBgYXNzZXRzL1tuYW1lXS5bZXh0XWBcbiAgICAgIH1cbiAgICB9XG4gIH0sXG5cbiAgLy8gVml0ZSBvcHRvbnMgdGFpbG9yZWQgZm9yIFRhdXJpIGRldmVsb3BtZW50IGFuZCBvbmx5IGFwcGxpZWQgaW4gYHRhdXJpIGRldmAgb3IgYHRhdXJpIGJ1aWxkYFxuICAvLyBwcmV2ZW50IHZpdGUgZnJvbSBvYnNjdXJpbmcgcnVzdCBlcnJvcnNcbiAgY2xlYXJTY3JlZW46IGZhbHNlLFxuICAvLyB0YXVyaSBleHBlY3RzIGEgZml4ZWQgcG9ydCwgZmFpbCBpZiB0aGF0IHBvcnQgaXMgbm90IGF2YWlsYWJsZVxuICBzZXJ2ZXI6IHtcbiAgICBob3N0OiBtb2JpbGUgPyAnMC4wLjAuMCcgOiBmYWxzZSxcbiAgICBwb3J0OiAxNDIwLFxuICAgIHN0cmljdFBvcnQ6IHRydWUsXG4gICAgaG1yOiBtb2JpbGVcbiAgICAgID8ge1xuICAgICAgICAgIHByb3RvY29sOiAnd3MnLFxuICAgICAgICAgIGhvc3Q6IGludGVybmFsSXBWNFN5bmMoKSxcbiAgICAgICAgICBwb3J0OiAxNDIxXG4gICAgICAgIH1cbiAgICAgIDogdW5kZWZpbmVkLFxuICAgIGZzOiB7XG4gICAgICBhbGxvdzogWycuJywgJy4uLy4uL3Rvb2xpbmcvYXBpL2Rpc3QnXVxuICAgIH1cbiAgfVxufSlcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFJQSxTQUFTLG9CQUFvQjtBQUM3QixPQUFPLFlBQVk7QUFDbkIsU0FBUyxjQUFjO0FBQ3ZCLFNBQVMsd0JBQXdCO0FBRWpDLElBQU0sU0FBUyxDQUFDLENBQUMsY0FBYyxLQUFLLFFBQVEsSUFBSSxrQkFBa0I7QUFHbEUsSUFBTyxzQkFBUSxhQUFhO0FBQUEsRUFDMUIsU0FBUyxDQUFDLE9BQU8sR0FBRyxPQUFPLENBQUM7QUFBQSxFQUM1QixPQUFPO0FBQUEsSUFDTCxlQUFlO0FBQUEsTUFDYixRQUFRO0FBQUEsUUFDTixnQkFBZ0I7QUFBQSxRQUNoQixnQkFBZ0I7QUFBQSxRQUNoQixnQkFBZ0I7QUFBQSxNQUNsQjtBQUFBLElBQ0Y7QUFBQSxFQUNGO0FBQUE7QUFBQTtBQUFBLEVBSUEsYUFBYTtBQUFBO0FBQUEsRUFFYixRQUFRO0FBQUEsSUFDTixNQUFNLFNBQVMsWUFBWTtBQUFBLElBQzNCLE1BQU07QUFBQSxJQUNOLFlBQVk7QUFBQSxJQUNaLEtBQUssU0FDRDtBQUFBLE1BQ0UsVUFBVTtBQUFBLE1BQ1YsTUFBTSxpQkFBaUI7QUFBQSxNQUN2QixNQUFNO0FBQUEsSUFDUixJQUNBO0FBQUEsSUFDSixJQUFJO0FBQUEsTUFDRixPQUFPLENBQUMsS0FBSyx3QkFBd0I7QUFBQSxJQUN2QztBQUFBLEVBQ0Y7QUFDRixDQUFDOyIsCiAgIm5hbWVzIjogW10KfQo=
