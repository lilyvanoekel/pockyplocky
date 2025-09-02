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
            1.000000000000000,
            3.007575757575758,
            6.007575757575758,
            8.969696969696969,
            10.250000000000000,
            10.901515151515152,
            11.666666666666666,
            12.340909090909092,
        ];
        let amp_factors = [0.560, 0.097, 1.000, 0.279, 0.162, 0.261, 0.127, 0.430];
        let decay_factors = [
            1.000000000000000,
            0.172074719519104,
            0.081453005389775,
            0.025219592107013,
            0.097586738445422,
            0.031886135584511,
            0.053639510338887,
            0.031945640843137,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_xylophone2_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000,
            4.115606936416185,
            8.132947976878613,
            11.497109826589597,
            15.202312138728324,
            17.086705202312142,
            21.624277456647402,
            23.820809248554916,
        ];
        let amp_factors = [1.000, 0.101, 0.043, 0.003, 0.005, 0.003, 0.002, 0.005];
        let decay_factors = [
            1.000000000000000,
            0.300444224041017,
            0.214891488770736,
            0.337197751120588,
            0.144160455845560,
            0.140557714902721,
            0.137480554193496,
            0.121315868812830,
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
            // 16.587064676616915 * 721.36,
            // 18.626865671641792 * 721.36,
            // 21.990049751243781 * 721.36,
            // 27.567164179104477 * 721.36,
            4.0 * fundamental,
            8.0 * fundamental,
            16.0 * fundamental,
            32.0 * fundamental,
        ];
        //let amp_factors = [1.000, 0.076, 0.0, 0.0, 0.019, 0.011, 0.007, 0.003];
        let amp_factors = [1.000, 0.076, 0.0, 0.0, 0.0, 0.0, 0.007, 0.003];
        //let amp_factors = [1.000, 0.076, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
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

    pub fn build_steel_drum_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000,
            1.989010989010989,
            3.967032967032967,
            8.000000000000000,
            15.989010989010989,
            62.890109890109891,
            94.791208791208803,
            122.659340659340671,
        ];
        let amp_factors = [0.831, 1.000, 0.892, 0.454, 0.335, 0.001, 0.000, 0.000];
        let decay_factors = [
            1.000000000000000,
            0.623899493354938,
            0.585019446642690,
            0.407375089303861,
            0.439135958839246,
            0.470252423450744,
            0.456525816985401,
            0.948483948685520,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
            amplitude: amp_factors[i],
            decay: decay_factors[i] * decay,
        })
    }

    pub fn build_metal_cup_modes(fundamental: f32, decay: f32) -> [Mode; NUM_MODES] {
        let freq_ratios = [
            1.000000000000000,
            5.773067331670823,
            8.700748129675810,
            13.329177057356608,
            20.957605985037404,
            24.117206982543642,
            31.286783042394013,
            35.411471321695757,
        ];
        let amp_factors = [1.000, 0.109, 0.058, 0.031, 0.007, 0.008, 0.012, 0.012];
        let decay_factors = [
            1.000000000000000,
            0.466325672229958,
            0.190757590796148,
            0.140479544493103,
            0.228468838218124,
            0.236733042101392,
            0.184080322378180,
            0.160926579642933,
        ];

        std::array::from_fn(|i| Mode {
            frequency: freq_ratios[i] * fundamental,
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
            Timbre::Xylophone2 => Timbre::build_xylophone2_modes(fundamental_freq, decay),
            Timbre::MetalPan => Timbre::build_metal_pan_modes(fundamental_freq, decay),
            Timbre::GlassMarimba => Timbre::build_glass_marimba_modes(fundamental_freq, decay),
            Timbre::Piano => Timbre::build_piano_modes(fundamental_freq, decay),
            Timbre::WoodBlocks => Timbre::build_wood_blocks_modes(fundamental_freq, decay),
            Timbre::SteelDrum => Timbre::build_steel_drum_modes(fundamental_freq, decay),
            Timbre::MetalCup => Timbre::build_metal_cup_modes(fundamental_freq, decay),
        };

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
