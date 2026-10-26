import { defineConfig } from "vite";
export default defineConfig({
  server: {
    proxy: {
      "/auth": "http://localhost:3000",
      "/admin": "http://localhost:3000",
      "/api": "http://localhost:3000",
    },
  },
  build: { outDir: "dist" },
});
