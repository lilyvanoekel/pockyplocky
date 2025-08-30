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
    amount: f32,
}

impl WaveFolder {
    pub fn new() -> Self {
        Self { amount: 0.0 }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let x = input * self.amount;
        fast_sin(x)
    }
}
