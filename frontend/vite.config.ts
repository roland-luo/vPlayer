import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5175,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    host: "127.0.0.1",
  },
}));
