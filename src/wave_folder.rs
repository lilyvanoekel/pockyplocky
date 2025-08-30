use crate::constants::MAX_BLOCK_SIZE;

fn fast_sin(x: f32) -> f32 {
    fake_sin(range_limiter(x))
}

fn fake_sin(x: f32) -> f32 {
    x * (1.0 - x.abs())
}

fn range_limiter(x: f32) -> f32 {
    fmod2(x + 1.0) - 1.0
}

fn fmod2(x: f32) -> f32 {
    2.0 * (x * 0.5 - (x * 0.5).floor())
}

pub struct WaveFolder {
    output_buffer: [f32; MAX_BLOCK_SIZE],
    amount: f32,
}

impl WaveFolder {
    pub fn new() -> Self {
        Self {
            output_buffer: [0.0; MAX_BLOCK_SIZE],
            amount: 0.0,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount;
    }

    pub fn process_block(&mut self, input: &[f32], block_len: usize) -> &[f32] {
        for i in 0..block_len {
            let x = input[i] * self.amount;
            self.output_buffer[i] = fast_sin(x);
        }
        &self.output_buffer[..block_len]
    }
}
