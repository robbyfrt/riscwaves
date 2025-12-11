# riscwaves Copilot Instructions

## Project Overview
riscwaves visualizes real-time accelerometer data from ESP32-C6 hardware using a Rust/WASM particle fluid engine powered by the pixels crate. The server receives sensor data (initially USB serial, later wireless) and streams it via WebSocket to a WASM client for interactive wave/particle rendering. Hardware sensor code lives in separate "riscnrust" repo; this repo focuses on server + WASM visualization.[1]

## Tech Stack
- **Rendering**: pixels crate for WASM particle engine (fluid simulation with accelerometer impulses)
- **Server**: Rust (Axum/Tokio) with WebSocket for bidirectional data (sensor input → particle forces)
- **Data Flow**: USB serial (serialport crate) → JSON accel packets → WebSocket → WASM pixels renderer
- **Interactive**: Mouse/keyboard input in WASM simulates accel data for testing
- **Targets**: wasm32-unknown-unknown for client; x86_64-unknown-linux-gnu for server[2]

## Folder Structure
```
riscwaves/
├── Cargo.toml          # Workspace: server + wasm
├── server/             # Axum WebSocket + serial reader
│   ├── src/
│   │   ├── main.rs     # Tokio runtime, serial→WS broadcast
│   │   ├── serial.rs   # USB accel data parser
│   │   └── ws.rs       # Axum WS handler, channels
│   └── Cargo.toml
├── wasm/               # pixels renderer
│   ├── src/
│   │   ├── lib.rs      # WASM entry, pixels::SurfaceTexture
│   │   ├── particles.rs # Fluid sim (vel/pos update from accel)
│   │   └── ws.rs       # JS WebSocket → Rust channel
│   └── Cargo.toml      # wasm-bindgen, console_error_panic_hook
├── www/                # Static HTML/JS host
│   ├── index.html      # Canvas + WS connect
│   └── wasm_loader.js  # wasm-bindgen init
└── README.md           # Build/flash instructions
```

## Key Implementation Steps
1. **Server Serial Reader**: Spawn Tokio task reading USB (e.g., `/dev/ttyACM0`) at 115200 baud, parse JSON `{"x":0.1,"y":-0.2,"z":0.9}`, broadcast via `tokio::sync::broadcast`.[3]
2. **WebSocket Handler**: Axum route `/ws` accepts client, clones broadcast channel, sends accel frames at 60Hz.
3. **WASM Pixels Loop**: `pixels::Pixels` context, update 10k particles/frame: `vel += accel * dt; pos += vel * dt` with bounds/spread. Render as RGBA trails.[4]
4. **JS Bridge**: `ws_stream_wasm` for Rust→JS WebSocket, apply mouse drag as simulated accel.[5]
5. **Build**: `wasm-pack build wasm --target web`; `cargo run --bin server`.

## Cargo Dependencies
**server/Cargo.toml**:
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tokio-serial = "5"
serde = { version = "1", features = ["derive"] }
futures = "0.3"
```

**wasm/Cargo.toml**:
```toml
[dependencies]
pixels = "0.13"  # WASM support
winit = "0.29"
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
ws_stream_wasm = "0.7"  # WebSocket
serde = { version = "1", features = ["derive"] }
```

## Quickstart Commands
```bash
# Server (reads /dev/ttyACM0, serves localhost:8080)
cargo run --bin server

# WASM dev
cd wasm && wasm-pack build --target web --out-dir ../www/pkg
cd ../www && python3 -m http.server 8081

# ESP32-C6 (separate riscnrust repo)
# Use esp-idf-hal I2C for Grove accel (MPU6050?), print JSON to USB CDC [web:1][web:33]
```

## Particle Engine Tips
- 1024-4096 particles max for 60fps WASM.
- Fluid: velocity field grid (16x16), advect particles, add curl noise.
- Accel impulse: global force vector scaled by magnitude.
- Render: distance-field circles or simple additive RGBA.[4]

## Testing Workflow
- Fake serial: `echo '{"x":0.5,"y":0,"z":0}' > test_serial.txt`; tail -f → named pipe.
- Interactive mode: Toggle WS sim data vs real sensor.
- Debug: `RUST_LOG=debug cargo run`; browser console for WASM.[6]

[1](https://github.com/esp-rs/esp-idf-hal)
[2](https://wasmbyexample.dev/examples/reading-and-writing-graphics/reading-and-writing-graphics.rust.en-us)
[3](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-guides/usb-serial-jtag-console.html)
[4](https://dgerrells.com/blog/how-fast-is-rust-simulating-200-000-000-particles)
[5](https://users.rust-lang.org/t/convenient-websockets-in-rust-wasm/30480)
[6](https://users.rust-lang.org/t/how-to-make-this-wasm-rust-code-faster-than-just-pure-js/123942)
[7](https://github.com/esp-rs/esp-idf-hal/issues/389)
[8](https://github.com/esp-rs/esp-idf-hal/blob/master/src/i2s.rs)
[9](https://docs.espressif.com/projects/rust/book/application-development/async.html)
[10](https://github.com/esp-rs/esp-idf-hal/blob/master/CHANGELOG.md)
[11](https://github.com/rust-embedded/awesome-embedded-rust)
[12](https://github.com/esp-rs/esp-idf-sys)
[13](https://github.com/esp-rs/awesome-esp-rust)
[14](https://github.com/esp-rs/esp-idf-hal/issues?u=http%3A%2F%2Fgithub.com%2Fesp-rs%2Fesp-idf-hal%2Fissues%2F349)
[15](https://github.com/esp-rs/esp-idf-sys/issues/200)
[16](https://github.com/esp-rs/esp-idf-svc)
[17](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/get-started/index.html)
[18](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/contribute/creating-examples.html)
[19](https://www.youtube.com/watch?v=DShBrBBUAt4)
[20](https://www.youtube.com/watch?v=FudHj7I1kTk)
[21](https://github.com/gametorch/image_to_pixel_art_wasm)
[22](https://gist.github.com/Graunephar/57a9882cb3a2ab98be8d63a59ab16ef3)
[23](https://www.reddit.com/r/WebAssembly/comments/rtdpo2/webassembly_pixel_game_engine_example_in_rust/)
[24](https://www.reddit.com/r/rust/comments/1aww22a/fluid_simulation_optimization_on_rust/)
[25](https://www.reddit.com/r/Esphome/comments/1h5rxvu/getting_sensor_data_from_serial_usb_instead_of/)
[26](https://github.com/rustwasm/wasm-bindgen/issues/597)
[27](https://www.reddit.com/r/esp32/comments/13gli3k/posted_open_source_project_for_an_esp32c6_using/)
[28](https://crates.io/crates/rust_pixel)
[29](https://www.reddit.com/r/rust/comments/ygkwqb/fluid_simulation_with_adaptive_particle_sizes/)
[30](https://docs.espressif.com/projects/esp-idf/en/v5.1/esp32c6/api-guides/usb-serial-jtag-console.html)
[31](https://stackoverflow.com/questions/79611233/how-should-i-structure-this-websocket-based-wasm-app)
[32](https://gist.github.com/Graunephar/57a9882cb3a2ab98be8d63a59ab16ef3?permalink_comment_id=5379198)
[33](https://dev.to/fallenstedt/using-rust-and-webassembly-to-process-pixels-from-a-video-feed-4hhg)
[34](https://randomnerdtutorials.com/esp32-mpu-6050-accelerometer-gyroscope-arduino/)
[35](https://stackoverflow.com/questions/60800934/rust-stm32-webusb-publishing-sensors)
[36](https://leapcell.io/blog/bringing-rust-s-performance-to-the-web-with-webassembly)
[37](https://github.com/hyperium/tonic/issues/491)
[38](https://crates.io/crates/pix-engine)
[39](https://botland.store/grove-accelerometers-and-gyroscopes/12916-grove-3-axis-accelerometer-gyroscope-and-magnetometer-icm20600-ak09918-i2c-5904422341510.html)
[40](https://github.com/Microtome/websocket-serial-server)
[41](https://lib.rs/simulation)
[42](https://www.berrybase.de/en/adafruit-adxl375-high-g-three-axis-accelerometer-200g-with-i2c-and-spi)
[43](https://www.reddit.com/r/rust/comments/ltujpi/fnwalkrs_project_using_actixweb_actixbroker/)
[44](https://rustwasm.github.io/docs/wasm-bindgen/print.html)