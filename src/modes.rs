use crate::params::{PockyplockyParams, Timbre};
use std::sync::Arc;

pub const NUM_MODES: usize = 8;

pub struct Mode {
    pub frequency: f32,
    pub amplitude: f32,
    pub decay: f32, // T60
}

impl Timbre {
    pub fn build_xylophone_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.0,
            2.895622619876336,
            5.298534048564759,
            5.832521190778428,
            8.742700954096883,
            9.997563572335284,
            10.558228573768478,
            11.946582859014782,
        ];
        let amp_factors = [0.595, 0.115, 0.083, 1.000, 0.133, 0.044, 0.180, 0.338];
        let decay_factors = [
            1.0,
            0.1843817787418655,
            0.13665943600867678,
            0.09544468546637742,
            0.03904555314533622,
            0.16268980477223427,
            0.04989154013015184,
            0.04772234273318871,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_metal_pan_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000,
            3.183520599250933,
            5.614232209737822,
            9.301498127340812,
            12.144194756554295,
            11.176029962546805,
            23.576779026217196,
            26.728464419475625,
        ];
        let amp_factors = [1.000, 0.981, 0.533, 0.169, 0.602, 0.699, 0.135, 0.147];
        let decay_factors = [
            1.000000000000000,
            0.701096915541291,
            0.440687570841232,
            0.448049302753838,
            0.230328015228289,
            0.140016355987610,
            0.161760273026999,
            0.107832378123913,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_glass_marimba_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000,
            1.972668810289390,
            2.662379421221866,
            5.226688102893893,
            5.438906752411580,
            6.729903536977495,
            8.657556270096467,
            9.612540192926049,
        ];
        let amp_factors = [1.000, 0.007, 0.024, 0.011, 0.007, 0.003, 0.001, 0.001];
        let decay_factors = [
            1.000000000000000,
            1.113139658617827,
            0.192714479880441,
            0.080295683467624,
            0.443737814485616,
            0.932516467188044,
            1.474846708318130,
            1.072432050296799,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_piano_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [1.0, 2.026, 3.097, 4.244, 5.468, 6.807, 8.220, 9.712];
        let amp_factors = [1.00, 0.45, 0.275, 0.15, 0.075, 0.04, 0.035, 0.02];
        let decay_factors = [1.0, 0.4, 0.25, 0.175, 0.125, 0.1, 0.075, 0.05];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_wood_blocks_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000 * fundamental,
            3.721393034825871 * fundamental,
            6.577114427860696 * fundamental,
            8.980099502487562 * fundamental,
            16.587064676616915 * 721.36,
            18.626865671641792 * 721.36,
            21.990049751243781 * 721.36,
            27.567164179104477 * 721.36,
        ];
        let amp_factors = [1.000, 0.076, 0.0, 0.0, 0.019, 0.011, 0.007, 0.003];
        let decay_factors = [
            1.000000000000000,
            0.692459239922517,
            0.598266002812814,
            0.863152420857781,
            0.513675729823460,
            0.633710327570556,
            0.508317876367345,
            0.492563057302712,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i],
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
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

    pub fn set_frequency(&mut self, fundamental_freq: f32, decay: f32) {
        let timbre = self.params.timbre.value();

        let new_modes = match timbre {
            Timbre::Xylophone => Timbre::build_xylophone_modes(fundamental_freq, decay),
            Timbre::MetalPan => Timbre::build_metal_pan_modes(fundamental_freq, decay),
            Timbre::GlassMarimba => Timbre::build_glass_marimba_modes(fundamental_freq, decay),
            Timbre::Piano => Timbre::build_piano_modes(fundamental_freq, decay),
            Timbre::WoodBlocks => Timbre::build_wood_blocks_modes(fundamental_freq, decay),
        };

        for i in 0..NUM_MODES {
            if new_modes[i].frequency > 20000.0 {
                self.modes[i].frequency = 20000.0;
                self.modes[i].decay = 1.0;
                self.modes[i].amplitude = 0.0;
            } else {
                self.modes[i].frequency = new_modes[i].frequency;
                self.modes[i].amplitude = new_modes[i].amplitude;
                self.modes[i].decay = new_modes[i].decay;
            }
        }
    }
}
