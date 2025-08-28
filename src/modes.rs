use crate::params::{PockyplockyParams, Timbre};
use std::sync::Arc;

pub const NUM_MODES: usize = 8;

pub struct Mode {
    pub frequency: f32,
    pub amplitude: f32,
    pub decay: f32, // T60
}

pub struct TimbreData {
    pub freq_factors: [f32; NUM_MODES],
    pub amp_factors: [f32; NUM_MODES],
    pub decay_factors: [f32; NUM_MODES],
}

impl Timbre {
    pub const XYLOPHONE_DATA: TimbreData = TimbreData {
        freq_factors: [
            1.0,
            2.895622619876336,
            5.298534048564759,
            5.832521190778428,
            8.742700954096883,
            9.997563572335284,
            10.558228573768478,
            11.946582859014782,
        ],
        amp_factors: [0.595, 0.115, 0.083, 1.000, 0.133, 0.044, 0.180, 0.338],
        decay_factors: [
            1.0,
            0.1843817787418655,
            0.13665943600867678,
            0.09544468546637742,
            0.03904555314533622,
            0.16268980477223427,
            0.04989154013015184,
            0.04772234273318871,
        ],
    };

    pub const METAL_PAN_DATA: TimbreData = TimbreData {
        freq_factors: [
            1.000000000000000,
            3.183520599250933,
            5.614232209737822,
            9.301498127340812,
            12.144194756554295,
            11.176029962546805,
            23.576779026217196,
            26.728464419475625,
        ],
        amp_factors: [1.000, 0.981, 0.533, 0.169, 0.602, 0.699, 0.135, 0.147],
        decay_factors: [
            1.000000000000000,
            0.701096915541291,
            0.440687570841232,
            0.448049302753838,
            0.230328015228289,
            0.140016355987610,
            0.161760273026999,
            0.107832378123913,
        ],
    };

    pub const GLASS_MARIMBA_DATA: TimbreData = TimbreData {
        freq_factors: [
            1.000000000000000,
            1.972668810289390,
            2.662379421221866,
            5.226688102893893,
            5.438906752411580,
            6.729903536977495,
            8.657556270096467,
            9.612540192926049,
        ],
        amp_factors: [1.000, 0.007, 0.024, 0.011, 0.007, 0.003, 0.001, 0.001],
        decay_factors: [
            1.000000000000000,
            1.113139658617827,
            0.192714479880441,
            0.080295683467624,
            0.443737814485616,
            0.932516467188044,
            1.474846708318130,
            1.072432050296799,
        ],
    };

    pub const PIANO_DATA: TimbreData = TimbreData {
        freq_factors: [1.0, 2.026, 3.097, 4.244, 5.468, 6.807, 8.220, 9.712],
        amp_factors: [1.00, 0.45, 0.275, 0.15, 0.075, 0.04, 0.035, 0.02],
        decay_factors: [1.0, 0.4, 0.25, 0.175, 0.125, 0.1, 0.075, 0.05],
    };

    pub fn data(&self) -> &'static TimbreData {
        match self {
            Timbre::Xylophone => &Self::XYLOPHONE_DATA,
            Timbre::MetalPan => &Self::METAL_PAN_DATA,
            Timbre::GlassMarimba => &Self::GLASS_MARIMBA_DATA,
            Timbre::Piano => &Self::PIANO_DATA,
        }
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
        let timbre_data = timbre.data();

        for i in 0..NUM_MODES {
            let freq = timbre_data.freq_factors[i] * fundamental_freq;
            let amp = timbre_data.amp_factors[i];
            let decay = timbre_data.decay_factors[i] * decay;

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
