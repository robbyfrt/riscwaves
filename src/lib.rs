use wasm_bindgen::prelude::*;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

static mut FRAME_COUNT: u32 = 0;
static mut PARTICLES: Vec<Particle> = Vec::new();
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

/// A simple particle with position and velocity
#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    mass: f32,
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
            mass: 1.0 + (Math::random() as f32) * 3.0, // between 1.0 and 4.0
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
        let radius = 3 * self.mass as i32;

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
        for _ in 0..100 {
            PARTICLES.push(Particle::new_random());
        }
        log::info!("Initialized {} particles", PARTICLES.len());
    }
}

fn request_animation_frame(context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    let window = window().ok_or("No window")?;

    let context = context.clone();
    let closure = Closure::wrap(Box::new(move |_time: f64| {
        update_frame(&context).ok();
        request_animation_frame(&context).ok();
    }) as Box<dyn FnMut(f64)>);

    window.request_animation_frame(closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

fn update_frame(context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    unsafe {
        FRAME_COUNT = FRAME_COUNT.wrapping_add(1);
    }

    // Create background with animated color
    let mut pixels = vec![0u8; (WIDTH * HEIGHT * 4) as usize];


    // === CUSTOM EVENT LOOP ===
    // Update and render your particles here
    update_and_render_particles(&mut pixels);

    let image_data = ImageData::new_with_u8_clamped_array(
        wasm_bindgen::Clamped(&pixels),
        WIDTH,
    )?;

    context.put_image_data(&image_data, 0.0, 0.0)?;

    Ok(())
}

/// === CUSTOM EVENT LOOP ===
/// Update particle physics and render them.
/// Modify this function to customize particle behavior!
fn update_and_render_particles(pixel_data: &mut [u8]) {
    unsafe {
        // Remove dead particles
        PARTICLES.retain(|p| p.life > 0.0);

        // Update all particles
        for particle in PARTICLES.iter_mut() {
            particle.update();
        }

        // Render all particles
        for particle in PARTICLES.iter() {
            particle.draw(pixel_data, WIDTH, HEIGHT);
        }

        // Spawn new particles occasionally
        if FRAME_COUNT % 10 == 0 && PARTICLES.len() < 400 {
            PARTICLES.push(Particle::new_random());
        }
    }
}
