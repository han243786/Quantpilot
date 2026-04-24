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
          if (!id.includes("node_modules")) {
            return undefined;
          }

          if (id.includes("@xyflow/react")) {
            return "flow-vendor";
          }

          if (id.includes("react") || id.includes("scheduler")) {
            return "react-vendor";
          }

          if (id.includes("zustand")) {
            return "state-vendor";
          }

          return "vendor";
        }
      }
    }
  }
});
