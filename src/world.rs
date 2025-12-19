use crate::{BOX_SIZE, HEIGHT, WIDTH};

pub struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: f32,
    velocity_y: f32,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1.0,
            velocity_y: 1.0,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    pub fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1.0;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1.0;
        }

        self.velocity_y += 1.0; // gravity effect
        self.velocity_y *= 0.995; // air resistance effect

        self.box_x += self.velocity_x as i16;
        self.box_y += self.velocity_y as i16;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

