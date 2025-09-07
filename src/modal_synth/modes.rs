use crate::params::{PockyplockyParams, Timbre};
use std::sync::Arc;

pub const NUM_MODES: usize = 8;

pub struct Mode {
    pub frequency: f32,
    pub amplitude: f32,
    pub decay: f32, // T60
}

struct TimbreData {
    freq_ratios: [f32; NUM_MODES],
    amp_factors: [f32; NUM_MODES],
    decay_factors: [f32; NUM_MODES],
}

const TIMBRE_DATA: [TimbreData; 9] = [
    // Xylophone
    TimbreData {
        freq_ratios: [
            1.0, 3.0075758, 6.007576, 8.969697, 10.25, 10.901515, 11.666667, 12.340909,
        ],
        amp_factors: [0.560, 0.097, 1.000, 0.279, 0.162, 0.261, 0.127, 0.430],
        decay_factors: [
            1.0,
            0.17207472,
            0.081453,
            0.025219591,
            0.097586736,
            0.031886134,
            0.05363951,
            0.031945642,
        ],
    },
    // Xylophone2
    TimbreData {
        freq_ratios: [
            1.0, 4.115607, 8.132948, 11.497109, 15.202312, 17.086704, 21.624277, 23.820808,
        ],
        amp_factors: [1.000, 0.101, 0.043, 0.003, 0.005, 0.003, 0.002, 0.005],
        decay_factors: [
            1.0, 0.30044422, 0.2148915, 0.33719775, 0.14416045, 0.14055772, 0.13748056, 0.12131587,
        ],
    },
    // MetalPan
    TimbreData {
        freq_ratios: [
            1.0, 3.1835206, 5.614232, 9.301498, 12.144195, 11.17603, 23.576779, 26.728464,
        ],
        amp_factors: [1.000, 0.981, 0.533, 0.169, 0.602, 0.699, 0.135, 0.147],
        decay_factors: [
            1.0, 0.7010969, 0.44068757, 0.4480493, 0.23032802, 0.14001636, 0.16176027, 0.10783238,
        ],
    },
    // GlassMarimba
    TimbreData {
        freq_ratios: [
            1.0, 1.9726688, 2.6623795, 5.226688, 5.4389067, 6.7299037, 8.657557, 9.61254,
        ],
        amp_factors: [1.000, 0.007, 0.024, 0.011, 0.007, 0.003, 0.001, 0.001],
        decay_factors: [
            1.0, 1.1131396, 0.19271448, 0.08029568, 0.4437378, 0.93251647, 1.4748467, 1.072432,
        ],
    },
    // Piano
    TimbreData {
        freq_ratios: [1.0, 2.026, 3.097, 4.244, 5.468, 6.807, 8.220, 9.712],
        amp_factors: [1.00, 0.45, 0.275, 0.15, 0.075, 0.04, 0.035, 0.02],
        decay_factors: [1.0, 0.4, 0.25, 0.175, 0.125, 0.1, 0.075, 0.05],
    },
    // WoodBlocks
    TimbreData {
        freq_ratios: [1.0, 3.721393, 6.5771146, 8.9801, 4.0, 8.0, 16.0, 32.0],
        amp_factors: [1.000, 0.076, 0.0, 0.0, 0.0, 0.0, 0.007, 0.003],
        decay_factors: [
            1.0, 0.6924592, 0.598266, 0.86315242, 0.51367573, 0.6337103, 0.5083179, 0.49256307,
        ],
    },
    // SteelDrum
    TimbreData {
        freq_ratios: [
            1.0, 1.9890109, 3.967033, 8.0, 15.989011, 62.89011, 94.79121, 122.65934,
        ],
        amp_factors: [0.831, 1.000, 0.892, 0.454, 0.335, 0.001, 0.000, 0.000],
        decay_factors: [
            1.0, 0.6238995, 0.58501945, 0.4073751, 0.43913596, 0.47025242, 0.45652582, 0.94848394,
        ],
    },
    // MetalCup
    TimbreData {
        freq_ratios: [
            1.0, 5.7730673, 8.700748, 13.329177, 20.957606, 24.117207, 31.286783, 35.411472,
        ],
        amp_factors: [1.000, 0.109, 0.058, 0.031, 0.007, 0.008, 0.012, 0.012],
        decay_factors: [
            1.0, 0.46632567, 0.19075759, 0.14047954, 0.22846884, 0.23673304, 0.18408032, 0.16092658,
        ],
    },
    // Cowbell
    TimbreData {
        freq_ratios: [
            0.99, 2.617788, 5.194327, 7.142597, 9.11625, 11.4548, 17.5057, 29.8047,
        ],
        amp_factors: [1.000, 0.157, 0.133, 0.041, 0.157, 0.077, 0.059, 0.004],
        decay_factors: [
            1.0, 0.479536, 0.987948, 1.087171, 0.457817, 0.545215, 0.449881, 0.324752,
        ],
    },
];

impl Timbre {
    pub fn build_modes(timbre: Timbre, fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let data = &TIMBRE_DATA[timbre as usize];
        std::array::from_fn(|i| Mode {
            frequency: data.freq_ratios[i] * fundamental,
            amplitude: data.amp_factors[i],
            decay: data.decay_factors[i] * decay,
        })
    }
}

pub struct ModeCalculator {
    modes: [Mode; NUM_MODES],
    params: Arc<PockyplockyParams>,
}

impl ModeCalculator {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            modes: std::array::from_fn(|_| Mode {
                frequency: 0.0,
                amplitude: 0.0,
                decay: 0.0,
            }),
            params,
        }
    }

    pub fn get_modes(&self) -> &[Mode; NUM_MODES] {
        &self.modes
    }

    #[allow(clippy::needless_range_loop)]
    pub fn set_frequency(&mut self, fundamental_freq: f32, decay: f32) {
        let timbre = self.params.timbre.value();
        let new_modes = Timbre::build_modes(timbre, fundamental_freq, decay);

        let fundamental_balance = self.params.fundamental_balance.value();
        let sparkle = self.params.sparkle.value();

        self.modes[0].frequency = new_modes[0].frequency;
        self.modes[0].decay = new_modes[0].decay;
        self.modes[0].amplitude = new_modes[0].amplitude * (1.0 + fundamental_balance);

        for i in 1..NUM_MODES {
            if new_modes[i].frequency > 20000.0 {
                self.modes[i].frequency = 20000.0;
                self.modes[i].decay = 1.0;
                self.modes[i].amplitude = 0.0;
            } else {
                self.modes[i].frequency = new_modes[i].frequency;
                self.modes[i].decay = new_modes[i].decay * (1.0 + sparkle);
                self.modes[i].amplitude = new_modes[i].amplitude * (1.0 - fundamental_balance);
            }
        }
    }

    pub fn reset(&mut self) {
        for mode in &mut self.modes {
            mode.frequency = 0.0;
            mode.amplitude = 0.0;
            mode.decay = 0.0;
        }
    }
}
