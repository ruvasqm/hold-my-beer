use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
mod utils;

// Optional: Better panic messages in browser console
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccelerometerData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FluidParticle {
    pub x: f64,
    pub y: f64,
    pub vx: f64, // velocity x
    pub vy: f64, // velocity y
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FluidState {
    pub particles: Vec<FluidParticle>,
    // In a real sim: surface points, pressure, etc.
    // For now, let's add the calculated tilt angles directly
    pub tilt_x_deg: f64,
    pub tilt_z_deg: f64,
}

#[wasm_bindgen]
pub struct BeerSimulator {
    particles: Vec<FluidParticle>,
    width: f64,
    height: f64,
    // Constants for our CSS-like simulation within WASM
    max_tilt_angle_x: f64,
    max_tilt_angle_z: f64,
    sensitivity_multiplier: f64,
}

#[wasm_bindgen]
impl BeerSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64) -> Self {
        // Optional: Set panic hook for better debugging in browser console
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        // Initialize some dummy particles (you'd have a real particle system)
        let mut particles = Vec::new();
        for i in 0..10 {
            // Fewer particles for simplicity
            particles.push(FluidParticle {
                x: (i as f64 * (width / 10.0)) + width / 20.0,
                y: height * 0.5, // Start in the middle
                vx: 0.0,
                vy: 0.0,
            });
        }

        BeerSimulator {
            particles,
            width,
            height,
            max_tilt_angle_x: 35.0,
            max_tilt_angle_z: 45.0,
            sensitivity_multiplier: 5.0,
        }
    }

    // We'll pass accelerometer data as a JsValue and deserialize it
    pub fn update(&mut self, accel_js: JsValue, _dt: f64) {
        // dt for delta time, if needed for physics
        let accel_data: Result<AccelerometerData, _> = serde_wasm_bindgen::from_value(accel_js);

        if let Ok(accel) = accel_data {
            // ---- Simple Particle Update (Placeholder for real fluid sim) ----
            // This is a VERY naive update, just to show data flow
            for p in self.particles.iter_mut() {
                // Apply a force based on accelerometer (inverted for "gravity" effect)
                // This isn't realistic fluid physics, just a demo.
                p.vx += -accel.x * 0.1; // dt would be important here
                p.vy += -accel.y * 0.1; // dt would be important here (y often points up from screen)

                p.x += p.vx;
                p.y += p.vy;

                // Simple boundary collision
                if p.x < 0.0 {
                    p.x = 0.0;
                    p.vx *= -0.5;
                }
                if p.x > self.width {
                    p.x = self.width;
                    p.vx *= -0.5;
                }
                if p.y < 0.0 {
                    p.y = 0.0;
                    p.vy *= -0.5;
                }
                if p.y > self.height {
                    p.y = self.height;
                    p.vy *= -0.5;
                }
            }

            // ---- For the CSS-like tilt (replicate JS logic in Rust) ----
            // This is what we had in JS, using accel.x and accel.y like from accelerationIncludingGravity
            // If your generic sensor provides proper acceleration, this interpretation might be different.
            // Assuming accel.x and accel.y here are analogous to DeviceMotionEvent's accelIncludingGravity x,y
            let mut tilt_z = -accel.x * self.sensitivity_multiplier;
            let mut tilt_x = accel.y * self.sensitivity_multiplier;

            tilt_z = tilt_z
                .max(-self.max_tilt_angle_z)
                .min(self.max_tilt_angle_z);
            tilt_x = tilt_x
                .max(-self.max_tilt_angle_x)
                .min(self.max_tilt_angle_x);

            // Store these calculated tilts. The main thread will use them for CSS.
            // We will pass these back in `get_state()`.
            // (This isn't using the particles for tilt yet, just passing values through)
        } else {
            // Handle deserialization error if needed
            // web_sys::console::error_1(&"Failed to deserialize accelerometer data".into());
        }
    }

    pub fn get_state(&self, accel_js: JsValue) -> JsValue {
        // Also pass accel here to calculate tilt on demand
        let accel_data: Result<AccelerometerData, _> = serde_wasm_bindgen::from_value(accel_js);
        let (mut final_tilt_x, mut final_tilt_z) = (0.0, 0.0);

        if let Ok(accel) = accel_data {
            final_tilt_z = -accel.x * self.sensitivity_multiplier;
            final_tilt_x = accel.y * self.sensitivity_multiplier;

            final_tilt_z = final_tilt_z
                .max(-self.max_tilt_angle_z)
                .min(self.max_tilt_angle_z);
            final_tilt_x = final_tilt_x
                .max(-self.max_tilt_angle_x)
                .min(self.max_tilt_angle_x);
        }

        let state = FluidState {
            particles: self.particles.clone(), // Clone for sending
            tilt_x_deg: final_tilt_x,
            tilt_z_deg: final_tilt_z,
        };
        serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::UNDEFINED)
    }

    // A simple test function
    pub fn greet(&self, name: &str) -> String {
        format!(
            "Hello from Rust WASM, {}! Simulator ready for {}x{} area.",
            name, self.width, self.height
        )
    }
}
