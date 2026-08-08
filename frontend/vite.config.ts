import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

function i18nGeneratePlugin(): Plugin {
  const root = path.dirname(fileURLToPath(import.meta.url));
  const run = () => {
    const result = spawnSync(process.execPath, ["./scripts/i18n-generate.mjs"], {
      cwd: root,
      stdio: "inherit",
    });
    if (result.status !== 0) {
      throw new Error("i18n-generate failed");
    }
  };

  return {
    name: "i18n-generate",
    buildStart() {
      run();
    },
    configureServer(server) {
      run();
      server.watcher.add(path.join(root, "assets/i18n"));
      server.watcher.on("change", (file) => {
        if (file.includes(`${path.sep}assets${path.sep}i18n${path.sep}`) && file.endsWith(".toml")) {
          run();
        }
      });
    },
  };
}

export default defineConfig({
  plugins: [i18nGeneratePlugin(), react()],
  clearScreen: false,
  server: {
    port: 3000,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
});
