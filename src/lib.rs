#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod world;
pub use world::{ParticleSystem, Renderer};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*; // at top, gated only for wasm32

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_start() {
    // this runs automatically when the module is instantiated via `init()`
    main();
}


pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

/// Representation of the application state. In this example, a box will bounce around the screen.

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        pollster::block_on(run());
    }
}

// dynamic window size retrieval for wasm32 targets
/*
#[cfg(target_arch = "wasm32")]
/// Retrieve current width and height dimensions of browser client window
fn get_window_size() -> LogicalSize<f64> {
    let client_window = web_sys::window().unwrap();
    LogicalSize::new(
        client_window.inner_width().unwrap().as_f64().unwrap(),
        client_window.inner_height().unwrap().as_f64().unwrap(),
    )
}
*/

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + Web")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        //use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("pixels-canvas"))
            .and_then(|old_canvas| {
                old_canvas.replace_with_with_node_1(&web_sys::Element::from(window.canvas().unwrap()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        let winit_canvas = window.canvas().unwrap();
            winit_canvas.set_class_name("pixels-surface");


        let _ = window.request_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64));
        // dynamic resize handling for browser client
        /*
        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new({
            let window = Rc::clone(&window);
            move |_e: web_sys::Event| {
                let _ = window.request_inner_size(get_window_size());
            }
        }) as Box<dyn FnMut(_)>);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();

        // Trigger initial resize event
        let _ = window.request_inner_size(get_window_size());
        */
    }

    let mut input = WinitInputHelper::new();
    let mut pixels = {
        #[cfg(not(target_arch = "wasm32"))]
        let window_size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        //let window_size = get_window_size().to_physical::<u32>(window.scale_factor());
        let window_size = winit::dpi::PhysicalSize::new(WIDTH, HEIGHT);

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let builder = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture);

        #[cfg(target_arch = "wasm32")]
        let builder = {
            // Web targets do not support the default texture format
            let texture_format = pixels::wgpu::TextureFormat::Rgba8Unorm;
            builder
                .texture_format(texture_format)
                .surface_texture_format(texture_format)
        };

        builder.build_async().await.expect("Pixels error")
    };
    let mut particles = ParticleSystem::new(10000, WIDTH as usize, HEIGHT as usize);
    let mut renderer = Renderer::new(WIDTH as usize, HEIGHT as usize);
    for _ in 0..10000 {
        particles.spawn_random(1.0, 1.0);
    }
    
    #[cfg(target_arch = "wasm32")]
    let mut frame_count = 0u32;
    #[cfg(target_arch = "wasm32")]
    let mut last_fps_update = get_time_ms();

    let res = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Draw the current frame
                renderer.draw(pixels.frame_mut(), &particles);
                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);
                    elwt.exit();
                    return;
                }

               #[cfg(target_arch = "wasm32")]
                {
                    frame_count += 1;
                    let now = get_time_ms();
                    let elapsed = now - last_fps_update;
                    
                    // Update stats every 500ms
                    if elapsed >= 250.0 {
                        let fps = (frame_count as f64 * 1000.0) / elapsed;
                        let used_mb = get_memory_usage_mb();
                        update_stats(particles.count, fps as f32, used_mb);
                        frame_count = 0;
                        last_fps_update = now;
                    }
                }

                particles.spawn_random(1.0, 1.0);


                // Update internal state and request a redraw
                particles.update();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Resize the window
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { device_id: _, position },
                ..
            } => {
                // Update cursor position
                let cursor_x = position.x as i16;
                let cursor_y = position.y as i16;
                #[cfg(target_arch = "wasm32")]
                set_id_text("debug-text", &format!("Cursor: ({}, {})", cursor_x, cursor_y));
                particles.attractor = Some(world::Attractor {
                    position: glam::Vec2::new(cursor_x as f32, cursor_y as f32),
                    strength: 6.0,
                    radius: 50,
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CursorLeft { device_id: _,},
                ..
            } => {
                // clear cursor position
                #[cfg(target_arch = "wasm32")]
                set_id_text("debug-text", "");
                particles.attractor = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Touch (touch),
                ..
            } => {
                use glam::Vec2;
                let touch_x = touch.location.x as f32 / (3.0 * WIDTH as f32) - 0.5;
                let touch_y = touch.location.y as f32 / (3.0 * HEIGHT as f32) - 0.5;
                // update axis motion
                #[cfg(target_arch = "wasm32")]
                set_id_text("debug-text", &format!("Touch: {:?}", touch));
                particles.simulation.acceleration = Vec2::new(touch_x,touch_y);
            }
            

            _ => (),
        }

        // Handle input events
        if input.update(&event) && (input.key_pressed(KeyCode::Escape) || input.close_requested()) {
            elwt.exit();
        }
    });
    res.unwrap();
}

#[cfg(target_arch = "wasm32")]
fn update_stats(particle_count: usize, fps: f32, used_mb: f64) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Update particle count
            if let Some(elem) = document.get_element_by_id("particle-count") {
                elem.set_text_content(Some(&particle_count.to_string()));
            }
            
            // Update FPS
            if let Some(elem) = document.get_element_by_id("fps") {
                elem.set_text_content(Some(&format!("{:.0}", fps)));
            }
            // Update memory usage  
            if let Some(elem) = document.get_element_by_id("memory-usage") {
                elem.set_text_content(Some(&format!("{:.2}", used_mb)));
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn set_id_text(id: &str, text: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(elem) = document.get_element_by_id(id) {
                elem.set_text_content(Some(text));
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn get_memory_usage_mb() -> f64 {
    // Update memory usage (read from the wasm linear memory)
    // Note: the page uses id="memory-usage" for the span.
    let mem_js = wasm_bindgen::memory();
    match mem_js.dyn_into::<js_sys::WebAssembly::Memory>() {
        Ok(memory) => {
            // `memory.buffer()` returns a JS value; convert to `ArrayBuffer`
            let buffer = js_sys::ArrayBuffer::from(memory.buffer());
            let used_bytes = buffer.byte_length() as f64;
            used_bytes / (1024.0 * 1024.0)
        }
        Err(e) => {
            log::warn!("failed to access wasm memory: {:?}", e);
            -1.0
        }
    }
}   

#[cfg(target_arch = "wasm32")]
fn get_time_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}