use crate::{HEIGHT, WIDTH};

pub struct ParticleSystem {
    pub position: Vec<[f32; 2]>,
    pub velocity: Vec<[f32; 2]>,
    forces: Vec<[f32; 2]>,
    mass: Vec<f32>,
    lifetime: Vec<f32>,
    pub count: usize,
    capacity: usize,
    radius: i16,
    params: SimParams,
}

pub struct SimParams {
    pub gravity: [f32; 2],
    pub global_drag: f32,           // simple velocity damping
    pub wind: [f32; 2],             // constant wind acceleration
    pub acceleration: [f32; 2],      // from acceleration sensor
    pub restitution: f32,           // wall collision bounce factor
    pub dt: f32,
}


impl ParticleSystem {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new(max_particles: usize) -> Self {
        Self {
            position: vec![[0.0, 0.0]; max_particles],
            velocity: vec![[0.0, 0.0]; max_particles],
            forces: vec![[0.0, 0.0]; max_particles],
            mass: vec![1.0; max_particles],
            lifetime: vec![1.0; max_particles],
            count: 0,
            capacity: max_particles,
            radius: 4,
            params: SimParams {
                gravity: [0.0, 0.5],
                global_drag: 0.01,
                wind: [0.0, 0.0],
                acceleration: [0.0, 0.0],
                restitution: 0.9,
                dt: 1.0,
            },
        }
    }
    pub fn spawn(&mut self, pos: [f32; 2], vel: [f32; 2], mass: f32, lifetime: f32) {
        if self.count < self.capacity {
            self.position[self.count] = pos;
            self.velocity[self.count] = vel;
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

            self.forces[i] = [0.0, 0.0];
            self.forces[i][0] += self.params.gravity[0] * self.mass[i];
            self.forces[i][1] += self.params.gravity[1] * self.mass[i];
            self.forces[i][0] += self.params.wind[0];
            self.forces[i][1] += self.params.wind[1];
            self.forces[i][0] += self.params.acceleration[0] * self.mass[i];
            self.forces[i][1] += self.params.acceleration[1] * self.mass[i];

            // simple drag: F = -k v
            self.forces[i][0] += -self.params.global_drag * self.velocity[i][0];
            self.forces[i][1] += -self.params.global_drag * self.velocity[i][1];

            
            // semi-implicit Euler integration  
            let acceleration = [
                self.forces[i][0] / self.mass[i],
                self.forces[i][1] / self.mass[i],
            ];

            self.velocity[i][0] += acceleration[0] * self.params.dt;
            self.velocity[i][1] += acceleration[1] * self.params.dt;
            
            self.position[i][0] += self.velocity[i][0] * self.params.dt;
            self.position[i][1] += self.velocity[i][1] * self.params.dt;          
            
            // simple wall collisions
            if self.position[i][0] - self.radius as f32 <= 0.0 || self.position[i][0] + self.radius as f32 >= WIDTH as f32 {
                self.velocity[i][0] *= -1.0;
                self.position[i][0] = self.position[i][0].clamp(0.0, (WIDTH - self.radius as u32) as f32);
            }
            if self.position[i][1] - self.radius as f32 <= 0.0 || self.position[i][1] + self.radius as f32 >= HEIGHT as f32 {
                self.velocity[i][1] *= -1.0;
                self.position[i][1] = self.position[i][1].clamp(0.0, (HEIGHT - self.radius as u32) as f32);
            }
            
            // self.lifetime[i] -= 0.001;
            if self.lifetime[i] < 0.0 {
                self.lifetime[i] = 0.0;
                self.velocity[i] = [0.0, 0.0];
                self.position[i] = [-100.0, -100.0]; // move off-screen
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
            let circle_x = self.position[particle_index][0] as i16;
            let circle_y = self.position[particle_index][1] as i16;
            let lifetime = self.lifetime[particle_index];
            self.draw_circle(frame, circle_x, circle_y, self.radius, lifetime);
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
}

