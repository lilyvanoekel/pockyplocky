pub const NUM_MODES: usize = 8;

pub struct Mode {
    pub frequency: f32,
    pub amplitude: f32,
    pub decay: f32, // T60
}

pub struct ModeCalculator {
    modes: [Mode; NUM_MODES],
}

impl ModeCalculator {
    pub fn new() -> Self {
        Self {
            modes: std::array::from_fn(|_| Mode {
                frequency: 0.0,
                amplitude: 0.0,
                decay: 0.0,
            }),
        }
    }

    pub fn get_modes(&self) -> &[Mode; NUM_MODES] {
        &self.modes
    }

    pub fn set_frequency(&mut self, fundamental_freq: f32, decay: f32) {
        const FREQ_FACTORS: [f32; NUM_MODES] = [
            1.0,
            2.895622619876336,
            5.298534048564759,
            5.832521190778428,
            8.742700954096883,
            9.997563572335284,
            10.558228573768478,
            11.946582859014782,
        ];
        const AMP_FACTORS: [f32; NUM_MODES] =
            [0.595, 0.115, 0.083, 1.000, 0.133, 0.044, 0.180, 0.338];
        const DECAY_FACTORS: [f32; NUM_MODES] = [
            1.0,
            0.1843817787418655,
            0.13665943600867678,
            0.09544468546637742,
            0.03904555314533622,
            0.16268980477223427,
            0.04989154013015184,
            0.04772234273318871,
        ];

        for i in 0..NUM_MODES {
            let freq = FREQ_FACTORS[i] * fundamental_freq;
            let amp = AMP_FACTORS[i];
            let decay = DECAY_FACTORS[i] * decay;

            if freq > 20000.0 {
                self.modes[i].frequency = 20000.0;
                self.modes[i].decay = 1.0;
                self.modes[i].amplitude = 0.0;
            } else {
                self.modes[i].frequency = freq;
                self.modes[i].decay = decay;
                self.modes[i].amplitude = amp;
            }
        }
    }
}
