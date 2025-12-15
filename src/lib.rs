use wasm_bindgen::prelude::*;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, MouseEvent};

static mut FRAME_COUNT: u32 = 0;
static mut PARTICLES: Vec<Particle> = Vec::new();
static mut LAST_TIME_MS: f64 = 0.0;
static mut MOUSE_X: f32 = -1.0;
static mut MOUSE_Y: f32 = -1.0;
static mut MOUSE_ACTIVE: bool = false;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const MASS_RANGE: [f32; 2] = [1.0, 4.0];

/// A simple particle with position and velocity
#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub life: f32, // 0.0 to 1.0

}

impl Particle {
    /// Create a new particle with random position and velocity
    pub fn new_random() -> Self {
        use js_sys::Math;
        Particle {
            x: Math::random() as f32 * WIDTH as f32,
            y: Math::random() as f32 * HEIGHT as f32,
            vx: (Math::random() as f32 - 0.5) * 4.0,
            vy: (Math::random() as f32 - 0.5) * 4.0,
            mass: MASS_RANGE[0] + (Math::random() as f32) * (MASS_RANGE[1] - MASS_RANGE[0]),
            life: 1.0,
        }
    }

    /// Update particle position and apply physics
    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        
        // Bounce off walls
        if self.x < 0.0 || self.x > WIDTH as f32 {
            self.vx *= -1.0;
            self.x = self.x.clamp(0.0, WIDTH as f32);
        }
        if self.y < 0.0 || self.y > HEIGHT as f32 {
            self.vy *= -1.0;
            self.y = self.y.clamp(0.0, HEIGHT as f32);
        }

        // jump boost in bottom-left corner
        if self.x < 0.05 * WIDTH as f32 && self.y > 0.95 * HEIGHT as f32 {
            self.vy += -2.0 / self.mass;
        } 

        self.vy += 0.1 / self.mass; // gravity
        self.vy *= 0.99; // air resistance

        // Decay life
        //self.life -= 0.002;
    }

    /// Draw particle as a circle on the canvas
    pub fn draw(&self, pixel_data: &mut [u8], _width: u32, _height: u32) {
        let x = self.x as i32;
        let y = self.y as i32;
        let radius = 3 + ((self.mass / 2.0) as i32);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = x + dx;
                let ny = y + dy;

                if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 {
                    let idx = (ny as u32 * WIDTH + nx as u32) as usize * 4;
                    if idx + 3 < pixel_data.len() {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist <= radius as f32 {
                            // Blend particle color based on life
                            let alpha = (self.life * 255.0) as u8;
                            pixel_data[idx] = pixel_data[idx].saturating_add(100);
                            pixel_data[idx + 1] = pixel_data[idx + 1].saturating_add(150);
                            pixel_data[idx + 2] = pixel_data[idx + 2].saturating_add(255);
                            pixel_data[idx + 3] = pixel_data[idx + 3].saturating_add(alpha / 2);
                        }
                    }
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();

    log::info!("Initializing riscwaves WASM...");

    // Get canvas element
    let window = window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    let canvas = document
        .get_element_by_id("pixels-canvas")
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
        .ok_or("Canvas element not found")?;

    canvas.set_width(WIDTH);
    canvas.set_height(HEIGHT);

    let context = canvas
        .get_context("2d")
        .map_err(|_| JsValue::from_str("Failed to get 2D context"))?
        .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
        .ok_or("Failed to get 2D rendering context")?;

    // === CUSTOM ENTRY POINT ===
    // Initialize your custom particles here
    initialize_custom_particles();

    // Set up mouse listeners to update global mouse position
    {
        let canvas_clone = canvas.clone();

        let mv = Closure::wrap(Box::new(move |event: MouseEvent| {
            unsafe {
                MOUSE_X = (event.offset_x() as f32) / 900.0 * (WIDTH as f32);
                MOUSE_Y = (event.offset_y() as f32) / 675.0 * (HEIGHT as f32);
                MOUSE_ACTIVE = true;
            }
        }) as Box<dyn FnMut(_)>);
        canvas_clone.add_event_listener_with_callback("mousemove", mv.as_ref().unchecked_ref())?;
        mv.forget();

        let leave = Closure::wrap(Box::new(move |_event: MouseEvent| {
            unsafe {
                MOUSE_ACTIVE = false;
                MOUSE_X = -1.0;
                MOUSE_Y = -1.0;
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseleave", leave.as_ref().unchecked_ref())?;
        leave.forget();
    }

    // Start the animation loop
    request_animation_frame(&context)?;

    Ok(())
}

/// === CUSTOM ENTRY POINT ===
/// Add your particle initialization logic here!
/// Example: spawn particles, set up initial state, etc.
fn initialize_custom_particles() {
    unsafe {
        // Example: Create 50 random particles
        for _ in 0..1000 {
            PARTICLES.push(Particle::new_random());
        }
        log::info!("Initialized {} particles", PARTICLES.len());
    }
}

fn request_animation_frame(context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    let window = window().ok_or("No window")?;

    let context = context.clone();
    let closure = Closure::wrap(Box::new(move |_time: f64| {
        update_frame(&context, _time).ok();
        request_animation_frame(&context).ok();
    }) as Box<dyn FnMut(f64)>);

    window.request_animation_frame(closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn update_frame(context: &CanvasRenderingContext2d, time_ms: f64) -> Result<(), JsValue> {
    // `time_ms` is the high-resolution timestamp provided by requestAnimationFrame (in ms)
    // Compute frametime and FPS using a simple last-time delta.
    let mut frametime_ms: f64 = 0.0;
    let mut fps: f64 = 0.0;

    unsafe {
        FRAME_COUNT = FRAME_COUNT.wrapping_add(1);
        if LAST_TIME_MS > 0.0 {
            frametime_ms = time_ms - LAST_TIME_MS;
            if frametime_ms > 0.0 {
                fps = 1000.0 / frametime_ms;
            }
        }
        LAST_TIME_MS = time_ms;
    }

    // Create black background
    let mut pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

    // === CUSTOM EVENT LOOP ===
    // Update and render your particles here
    update_and_render_particles(&mut pixels);

    let image_data = ImageData::new_with_u8_clamped_array(
        wasm_bindgen::Clamped(&pixels),
        WIDTH,
    )?;

    context.put_image_data(&image_data, 0.0, 0.0)?;

    // Draw overlay text in upper-right corner
    let particles_count = unsafe { PARTICLES.len() };

    // Style
    context.set_fill_style(&JsValue::from_str("rgba(255,255,255,0.95)"));
    context.set_font("10px monospace");
    context.set_text_baseline("top");
    context.set_text_align("right");

    let x = WIDTH as f64 - 8.0;
    let y1 = 8.0;
    let y2 = y1 + 14.0;

    // Pad numeric fields so widths remain stable (right-aligned within fixed width)
    let s1 = format!("particles: {:>3}", particles_count);
    let framems_i = frametime_ms.round() as i64;
    let fps_i = fps.round() as i64;
    let s2 = format!(
        "frametime: {:>3} ms ({:>3}FPS)",
        framems_i,
        fps_i
    );

    let _ = context.fill_text(&s1, x, y1);
    let _ = context.fill_text(&s2, x, y2);

    Ok(())
}

/// === CUSTOM EVENT LOOP ===
/// Update particle physics and render them.
/// Modify this function to customize particle behavior!
fn update_and_render_particles(pixel_data: &mut [u8]) {
    unsafe {
        // Remove dead particles
        PARTICLES.retain(|p| p.life > 0.0);

        // Apply radial mouse force: particles within radius are attracted toward mouse
        if MOUSE_ACTIVE {
            let radius: f32 = 50.0;
            let strength: f32 = 8.0;
            for particle in PARTICLES.iter_mut() {
                let dx = MOUSE_X - particle.x;
                let dy = MOUSE_Y - particle.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.0 && dist <= radius {
                    let inv_dist = 1.0 / dist;
                    let nx = dx * inv_dist;
                    let ny = dy * inv_dist;
                    let falloff = 1.0 - (dist / radius);
                    let fx = nx * falloff * strength / particle.mass;
                    let fy = ny * falloff * strength / particle.mass;
                    particle.vx += fx;
                    particle.vy += fy;
                }
            }
        }

        // Update all particles
        for particle in PARTICLES.iter_mut() {
            particle.update();
        }

        // Render all particles
        for particle in PARTICLES.iter() {
            particle.draw(pixel_data, WIDTH, HEIGHT);
        }

        // Spawn new particles occasionally
        if PARTICLES.len() < 4000 {
            PARTICLES.push(Particle::new_random());
        }
    }
}
