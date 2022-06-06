import { ConfigEnv, defineConfig, UserConfig } from "vite";
import react from "@vitejs/plugin-react";

// proxy host settings
const proxies = {
	development: {
		"/api/ws": {
			target: "ws://localhost:9000",
			changeOrigin: true,
			ws: true,
		},
		"^/api/.*": {
			target: "http://localhost:9000",
			changeOrigin: true,
		},
	},

	production: {},
};

// output directory
const outDir = {
	development: "../target/debug/frontend/dist",
	production: "../target/release/frontend/dist",
};

// https://vitejs.dev/config/
export default defineConfig(({ command, mode }: ConfigEnv): UserConfig => {
	console.log(`Command: ${command}`);
	console.log(`Mode: ${mode}`);
	return {
		plugins: [react()],
		server: {
			proxy: proxies[mode],
		},
		build: {
			outDir: outDir[mode],
			// our outDir is outside of project root, so
			// set emptyOutDir manually to refresh it on every build
			emptyOutDir: true,
		},
	};
});
