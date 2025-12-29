use::glam::Vec2;
use crate::{HEIGHT, WIDTH};

pub struct ParticleSystem {
    pub position: Vec<Vec2>,
    pub velocity: Vec<Vec2>,
    forces: Vec<Vec2>,
    mass: Vec<f32>,
    lifetime: Vec<f32>,
    pub count: usize,
    capacity: usize,
    radius: i16,
    pub params: SimParams,
    draw_mode: DrawMode,
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

#[allow(dead_code)]
enum DrawMode {
    Circle {radius: i16},
    Point
} 

pub struct Attractor {
    pub position: Vec2,
    pub strength: f32,
    pub radius: u8,
}

impl ParticleSystem {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new(max_particles: usize) -> Self {
        Self {
            position: vec![Vec2::new(0.0, 0.0); max_particles],
            velocity: vec![Vec2::new(0.0, 0.0); max_particles],
            forces: vec![Vec2::new(0.0, 0.0); max_particles],
            mass: vec![1.0; max_particles],
            lifetime: vec![1.0; max_particles],
            count: 0,
            capacity: max_particles,
            radius: 4,
            params: SimParams {
                gravity: Vec2::new(0.0, 0.5),
                global_drag: Vec2::new(0.01, 0.01),
                wind: Vec2::new(0.0, 0.0),
                acceleration: Vec2::new(0.0, 0.0),
                restitution: 0.9,
                dt: 1.0,
            },
            draw_mode: DrawMode::Point,
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
                rand::random::<f32>() * WIDTH as f32,
                rand::random::<f32>() * HEIGHT as f32,
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
        for i in 0..self.count {

            self.forces[i] = Vec2::new(0.0, 0.0);
            self.forces[i] += self.params.gravity * self.mass[i];       // gravity
            self.forces[i] += self.params.wind;                         // wind
            self.forces[i] += self.params.acceleration * self.mass[i];  // external acceleration

            // simple drag: F = -k v
            self.forces[i] += - self.params.global_drag * self.velocity[i];

            // semi-implicit Euler integration  
            let acceleration = self.forces[i] / self.mass[i];

            self.velocity[i] += acceleration * self.params.dt;
            
            self.position[i] += self.velocity[i] * self.params.dt;       
            
            // simple wall collisions
            if self.position[i][0] - self.radius as f32 <= 0.0 || self.position[i][0] + self.radius as f32 >= WIDTH as f32 {
                self.velocity[i][0] *= -1.0;
                self.position[i][0] = self.position[i][0].clamp(0.0, (WIDTH - self.radius as u32) as f32);
            }
            if self.position[i][1] - self.radius as f32 <= 0.0 || self.position[i][1] + self.radius as f32 >= HEIGHT as f32 {
                self.velocity[i][1] *= -1.0;
                self.position[i][1] = self.position[i][1].clamp(0.0, (HEIGHT - self.radius as u32) as f32);
            }
            
            //  repell at bottom left corner
            if self.position[i][0] < 10.0 && self.position[i][1] >= 0.95 * HEIGHT as f32 {
                self.velocity[i] += Vec2::new(2.0,-8.0) / self.mass[i];
            }
            if self.attractor.is_some() {
                let attractor = self.attractor.as_ref().unwrap();
                let to_particle = self.position[i] - attractor.position;
                let distance = to_particle.length();
                if distance < attractor.radius as f32 {
                    let n = to_particle * (1.0 / distance);
                    let falloff = 1.0 - (distance / attractor.radius as f32);
                    self.velocity[i] += -n * falloff * attractor.strength / self.mass[i];
                }
            }
            // self.lifetime[i] -= 0.001;
            if self.lifetime[i] < 0.0 {
                self.lifetime[i] = 0.0;
                self.velocity[i] = Vec2::ZERO;
                self.position[i] = Vec2::new(-100.0, -100.0); // move off-screen
            }
        }
    }

    /// Draw the `ParticleSystem` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        // Clear the frame to black
        frame.fill(0x00);

        for particle_index in 0..self.count {
            let x = self.position[particle_index][0] as i16;
            let y = self.position[particle_index][1] as i16;
            let lifetime = self.lifetime[particle_index];

            match self.draw_mode {
                DrawMode::Circle {radius} => self.draw_circle(frame, x, y, radius, lifetime),
                DrawMode::Point =>  self.draw_point_fast(frame, x, y, lifetime),
            }
        }
    }
    fn draw_circle(&self, frame: &mut [u8], center_x: i16, center_y: i16, radius: i16, lifetime: f32) {
        let radius_squared = radius * radius;
        let min_x = (center_x - radius).max(0);
        let max_x = (center_x + radius).min(WIDTH as i16 - 1);
        let min_y = (center_y - radius).max(0);
        let max_y = (center_y + radius).min(HEIGHT as i16 - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x - center_x;
                let dy = y - center_y;
                if dx * dx + dy * dy <= radius_squared {
                    let index = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    let alpha = (lifetime * 255.0) as u8;
                    frame[index] = 0xFF;     // R
                    frame[index + 1] = 0xFF; // G
                    frame[index + 2] = 0xFF; // B
                    frame[index + 3] = alpha; // A
                }
            }
        }
    } 
    fn draw_point_fast(&self, frame: &mut [u8], x: i16, y: i16, lifetime: f32) {
        if x < 0 || x >= WIDTH as i16 || y < 0 || y >= HEIGHT as i16 {
            return;
        }
        
        let index = ((y as u32 * WIDTH + x as u32) * 4) as usize;
        let alpha = (lifetime * 255.0) as u8;
        
        // Single pixel write (vastly faster than circle)
        frame[index] = 0xFF;
        frame[index + 1] = 0xFF;
        frame[index + 2] = 0xFF;
        frame[index + 3] = alpha;
    }
}

