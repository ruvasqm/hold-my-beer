// src/fluid.worker.js
let simulator = null;
let lastTimestamp = 0;

async function initWasm() {
	try {
		// This imports the module, and wasmModule will be an object where 'default' is the init function
		const wasmModule = await import("./pkg_fluid_simulation");

		// Call the default export, which is the __wbg_init function.
		// This function handles fetching and instantiating the .wasm file.
		// The glue code you provided will try to load 'fluid_simulation_wasm_bg.wasm'
		// relative to import.meta.url (which is the location of the glue JS file itself).
		// Since both fluid_simulation_wasm.js and fluid_simulation_wasm_bg.wasm
		// are in /pkg_fluid_simulation/, this relative path should work.
		await wasmModule.default();

		// After wasmModule.default() completes successfully, the `wasm` variable
		// INSIDE THE GLUE CODE (fluid_simulation_wasm.js) is populated with the exports
		// from your Rust WASM module. The classes and functions you defined in Rust
		// (like BeerSimulator) are then available as named exports from `wasmModule`.
		// So, wasmModule.BeerSimulator should now be defined.

		const simWidth = 300;
		const simHeight = 500;

		// Ensure BeerSimulator is actually exported from the wasmModule object
		// after default() has been called.
		if (typeof wasmModule.BeerSimulator !== "function") {
			console.error(
				"WASM Worker: BeerSimulator class not found on wasmModule after init.",
				Object.keys(wasmModule),
			);
			throw new Error(
				"BeerSimulator class not found on wasmModule after init.",
			);
		}

		simulator = new wasmModule.BeerSimulator(simWidth, simHeight);

		console.log(simulator.greet("Worker (Default Export Init)"));
		postMessage({
			type: "WORKER_READY",
			payload: "WASM Simulator Initialized (Default Export Init)",
		});
	} catch (err) {
		console.error("WASM Worker: Error initializing WASM:", err);
		// Your existing error handling...
		if (
			err instanceof WebAssembly.CompileError &&
			err.message.includes("expected magic word")
		) {
			postMessage({
				type: "WORKER_ERROR",
				payload: `WASM Init Failed: Path to .wasm file is likely incorrect or file not found. ${err.message}`,
			});
		} else if (
			err instanceof TypeError &&
			(err.message.includes("is not a function") ||
				err.message.includes("is not a constructor"))
		) {
			postMessage({
				type: "WORKER_ERROR",
				payload: `WASM Init Failed: Problem calling init or constructor. Check exports. ${err.message} Keys: ${err.message.includes("wasmModule.default") ? Object.keys(err.target || {}) : ""}`,
			});
		} else {
			postMessage({
				type: "WORKER_ERROR",
				payload: `WASM Init Failed: ${err.message}`,
			});
		}
	}
}

// ... (rest of your worker code as before) ...
self.onmessage = async (event) => {
	if (!simulator && event.data.type !== "INIT_WORKER") {
		console.warn(
			"WASM Worker: Simulator not ready, message ignored:",
			event.data.type,
		);
		postMessage({ type: "WORKER_ERROR", payload: "WASM not initialized yet." });
		return;
	}
	const { type, payload } = event.data;
	switch (type) {
		case "INIT_WORKER":
			await initWasm();
			break;
		case "ACCELEROMETER_DATA":
			if (simulator) {
				const currentTimestamp = performance.now();
				const dt =
					lastTimestamp > 0
						? (currentTimestamp - lastTimestamp) / 1000.0
						: 1 / 60;
				lastTimestamp = currentTimestamp;
				simulator.update(payload, dt);
				const fluidState = simulator.get_state(payload);
				postMessage({ type: "FLUID_STATE_UPDATE", payload: fluidState });
			}
			break;
	}
};
