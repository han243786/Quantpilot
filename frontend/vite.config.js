import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react({ fastRefresh: false })],
  server: {
    port: 5173,
    host: "0.0.0.0",
    strictPort: true,
    hmr: false,
    proxy: {
      "/api": {
        target: process.env.VITE_BACKEND_ORIGIN || "http://127.0.0.1:3000",
        changeOrigin: true
      }
    }
  },
  preview: {
    port: 4173,
    host: "0.0.0.0",
    strictPort: true
  },
  build: {
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalizedId = id.replace(/\\/g, "/");
          if (!normalizedId.includes("node_modules")) {
            return undefined;
          }

          if (normalizedId.includes("node_modules/@xyflow/react")) {
            return "flow-vendor";
          }

          if (normalizedId.includes("node_modules/zustand")) {
            return "state-vendor";
          }

          if (normalizedId.includes("node_modules/recharts")) {
            return "chart-vendor";
          }

          return "vendor";
        }
      }
    }
  }
});
