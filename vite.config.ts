import basicSsl from "@vitejs/plugin-basic-ssl";
import { defineConfig } from "vite";
import mkcert from "vite-plugin-mkcert";
export default defineConfig({
	plugins: [mkcert()],
	worker: {
		format: "es", // Important for modern worker syntax and WASM imports
	},
	build: {
		target: "esnext", // Ensures top-level await is supported if used
	},
});
