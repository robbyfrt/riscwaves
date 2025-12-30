use glam::Vec2;

pub struct ParticleSystem {
    width: usize,
    height: usize,
    position: Vec<Vec2>,
    velocity: Vec<Vec2>,
    forces: Vec<Vec2>,
    mass: Vec<f32>,
    lifetime: Vec<f32>,
    pub count: usize,
    capacity: usize,
    radius: i16,
    pub simulation: SimParams,
    pub attractor: Option<Attractor>,
}

pub struct SimParams {
    pub gravity: Vec2,
    pub wind: Vec2,              // constant wind acceleration
    pub acceleration: Vec2,      // from acceleration sensor
    pub global_drag: Vec2,        // simple velocity damping
    pub restitution: f32,        // wall collision bounce factor
    pub dt: f32,
}

pub struct Attractor {
    pub position: Vec2,
    pub strength: f32,
    pub radius: u8,
}

impl ParticleSystem {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new(max_particles: usize, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            position: vec![Vec2::new(0.0, 0.0); max_particles],
            velocity: vec![Vec2::new(0.0, 0.0); max_particles],
            forces: vec![Vec2::new(0.0, 0.0); max_particles],
            mass: vec![1.0; max_particles],
            lifetime: vec![1.0; max_particles],
            count: 0,
            capacity: max_particles,
            radius: 4,
            simulation: SimParams {
                gravity: Vec2::new(0.0, 0.5),
                global_drag: Vec2::new(0.01, 0.01),
                wind: Vec2::new(0.0, 0.0),
                acceleration: Vec2::new(0.0, 0.0),
                restitution: 0.9,
                dt: 1.0,
            },
            attractor: None,
        }
    }
    pub fn spawn(&mut self, pos: [f32; 2], vel: [f32; 2], mass: f32, lifetime: f32) {
        if self.count < self.capacity {
            self.position[self.count] = Vec2::new(pos[0], pos[1]);
            self.velocity[self.count] = Vec2::new(vel[0], vel[1]);
            self.mass[self.count] = mass;
            self.lifetime[self.count] = lifetime;
            self.count += 1;
        }
    }
    pub fn spawn_random(&mut self, mass: f32, lifetime: f32) {
        if self.count < self.capacity {
            let position = [
                rand::random::<f32>() * self.width as f32,
                rand::random::<f32>() * self.height as f32,
            ];
            let velocity = [
                (rand::random::<f32>() - 0.5) * 4.0,
                (rand::random::<f32>() - 0.5) * 4.0,
            ];
            self.spawn(position, velocity, mass, lifetime);
        }
    }

    /// Update the `ParticleSystem` internal state; bounce the particles around the screen.
    pub fn update(&mut self) {
        let g = self.simulation.gravity;
        let wind = self.simulation.wind;
        let acc = self.simulation.acceleration;
        let drag = self.simulation.global_drag;
        let dt = self.simulation.dt;
        let radius = self.radius as f32;

        for i in 0..self.count {
            let m = self.mass[i];
            let mut pos = self.position[i];
            let mut vel = self.velocity[i];
            let mut lt = self.lifetime[i];

            let mut f = Vec2::new(0.0, 0.0);
            f += g * m;         // gravity
            f += wind;          // wind
            f += acc * m;       // external acceleration
            f += - drag * vel;  // simple drag: F = -k v

            // semi-implicit Euler integration  
            let acceleration = f / m;

            vel += acceleration * dt;
            
            pos += vel * dt;       
            // simple wall collisions
            if pos[0] - radius <= 0.0 || pos[0] + radius >= self.width as f32 {
                vel[0] *= -1.0;
                pos[0] = pos[0].clamp(0.0, (self.width - radius as usize) as f32);
            }
            if pos[1] - radius <= 0.0 || pos[1] + radius >= self.height as f32 {
                vel[1] *= -1.0;
                pos[1] = pos[1].clamp(0.0, (self.height - radius as usize) as f32);
            }
            
            //  repell at bottom left corner
            if pos[0] < 10.0 && pos[1] >= 0.95 * self.height as f32 {
                vel += Vec2::new(2.0,-8.0) / m;
            }
            if self.attractor.is_some() {
                let attractor = self.attractor.as_ref().unwrap();
                let to_particle = pos - attractor.position;
                let distance = to_particle.length();
                if distance < attractor.radius as f32 {
                    let n = to_particle * (1.0 / distance);
                    let falloff = 1.0 - (distance / attractor.radius as f32);
                    vel += -n * falloff * attractor.strength / m;
                }
            }
            // lt -= 0.001;
            if lt < 0.0 {
                lt = 0.0;
                vel = Vec2::ZERO;
                pos = Vec2::new(-100.0, -100.0); // move off-screen
            }

            // write back mutated values
            self.forces[i] = f;
            self.velocity[i] = vel;
            self.position[i] = pos;
            self.lifetime[i] = lt;
        }
    }
}

#[allow(dead_code)]
pub struct Renderer{
    width: usize,
    height: usize,
    mode: DrawMode,
    post_process: Option<PostProcess>,
    temp_buffer: Vec<u8>,
    blur_buffer: Vec<u8>,
    dirty_rect: Option<(usize, usize, usize, usize)>
}

#[allow(dead_code)]
enum DrawMode {
    Circle {radius: i16},
    Point
} 

#[allow(dead_code)]
enum PostProcess {
    BoxBlur {kernel_size: usize},
    Bloom {threshold: f32, intensity: f32},
    Dilate {radius: usize},
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mode: DrawMode::Point,
            post_process: None,
            temp_buffer: vec![0u8; width * height * 4],
            blur_buffer: vec![0u8; width * height * 4],
            dirty_rect: None,
            }
        }
    /// Draw the `ParticleSystem` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&mut self, frame: &mut [u8], particles: &ParticleSystem) {
        // Clear the frame to black
        frame.fill(0x00);

        // track region of interest
        let mut min_x = self.width ;
        let mut max_x = 0;
        let mut min_y = self.height;
        let mut max_y = 0;
                

        for particle_index in 0..particles.count {
            let x  = particles.position[particle_index].x as usize;
            let y  = particles.position[particle_index].y as usize;
            let lifetime = particles.lifetime[particle_index];

            match self.mode {
                DrawMode::Circle {radius} => self.draw_circle(frame, x as i16, y as i16, radius, lifetime),
                DrawMode::Point =>  self.draw_point_fast(frame, x, y),
            }

            // Update bounds for dirty region
            if x < min_x { min_x = x; }
            if x > max_x { max_x = x; }
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }
        
        // Store dirty region
        self.dirty_rect = Some((min_x, min_y, max_x, max_y));

        // Apply post-processing
        self.dilation(frame);
        // self.alpha_cross_blur(frame);

    }
    fn draw_circle(&self, frame: &mut [u8], center_x: i16, center_y: i16, radius: i16, lifetime: f32) {
        let radius_squared = radius * radius;
        let min_x = (center_x - radius).max(0);
        let max_x = (center_x + radius).min(self.width as i16 - 1);
        let min_y = (center_y - radius).max(0);
        let max_y = (center_y + radius).min(self.height as i16 - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x - center_x;
                let dy = y - center_y;
                if dx * dx + dy * dy <= radius_squared {
                    let index = (y as usize * self.width + x as usize) * 4;
                    let alpha = (lifetime * 255.0) as u8;
                    frame[index] = 0xFF;     // R
                    frame[index + 1] = 0xFF; // G
                    frame[index + 2] = 0xFF; // B
                    frame[index + 3] = alpha; // A
                }
            }
        }
    } 
    fn draw_point_fast(&self, frame: &mut [u8], x: usize, y: usize) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 4;
            frame[idx..idx + 4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        }
    }
    pub fn dilation(&mut self, frame: &mut [u8]) {
        let Some((min_x, min_y, max_x, max_y)) = self.dirty_rect else {
            return;
        };
        let width = self.width as usize;
        let height = self.height as usize;
        // Copy only dirty region to temp buffer
        self.temp_buffer.copy_from_slice(frame);
        let src = &self.temp_buffer;
        
        // Process only active region with 1-pixel padding
        for y in min_y..max_y {
            for x in min_x..max_x {
                let idx = (y * width + x) * 4;
                
                // Skip if already white
                if src[idx + 3] == 0xFF {
                    continue;
                }
                
                // Check 3x3 neighborhood (unrolled for speed)
                let w = width * 4;
                let has_neighbor = 
                    (x > 0 && src[idx - 4 + 3] > 0) ||
                    (x < width - 1 && src[idx + 4 + 3] > 0) ||
                    (y > 0 && src[idx - w + 3] > 0) ||
                    (y < height - 1 && src[idx + w + 3] > 0) ||
                    (x > 0 && y > 0 && src[idx - w - 4 + 3] > 0) ||
                    (x < width - 1 && y > 0 && src[idx - w + 4 + 3] > 0) ||
                    (x > 0 && y < height - 1 && src[idx + w - 4 + 3] > 0) ||
                    (x < width - 1 && y < height - 1 && src[idx + w + 4 + 3] > 0);
                
                if has_neighbor {
                    frame[idx..idx + 3].copy_from_slice(&[0xCC, 0xCC, 0xCC]);
                    frame[idx + 3] = 0xCC; // Slightly transparent dilated pixels
                }
            }
        }
    }
}