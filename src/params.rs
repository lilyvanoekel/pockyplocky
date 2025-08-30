use std::sync::Arc;

use nih_plug::prelude::*;

use crate::constants::MAX_BLOCK_SIZE;

#[derive(Params)]
pub struct PockyplockyParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "timbre"]
    pub timbre: EnumParam<Timbre>,
    #[id = "noise_level"]
    pub noise_level: FloatParam,
    #[id = "noise_decay"]
    pub noise_decay: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "fundamental_balance"]
    pub fundamental_balance: FloatParam,
    #[id = "sparkle"]
    pub sparkle: FloatParam,
    #[id = "click"]
    pub click: BoolParam,
    #[id = "wave_folder_enabled"]
    pub wave_folder_enabled: BoolParam,
    #[id = "wave_folder_amount"]
    pub wave_folder_amount: FloatParam,
    #[id = "second_voice_enabled"]
    pub second_voice_enabled: BoolParam,
    #[id = "second_voice_detune"]
    pub second_voice_detune: FloatParam,
    #[id = "second_voice_stereo_spread"]
    pub second_voice_stereo_spread: FloatParam,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq)]
pub enum Timbre {
    #[name = "Xylophone"]
    Xylophone,
    #[name = "Metal Pan"]
    MetalPan,
    #[name = "Glass Marimba"]
    GlassMarimba,
    #[name = "Piano"]
    Piano,
    #[name = "Wood Blocks"]
    WoodBlocks,
    #[name = "Steel Drum"]
    SteelDrum,
}

impl Default for PockyplockyParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-12.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-36.0),
                    max: util::db_to_gain(0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(5.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            timbre: EnumParam::new("Timbre", Timbre::Xylophone),

            noise_level: FloatParam::new(
                "Noise Level (Filter)",
                0.1,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            noise_decay: FloatParam::new(
                "Noise Decay",
                400.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 500.0,
                },
            )
            .with_unit(" ms"),

            decay: FloatParam::new("Decay", 0.461, FloatRange::Linear { min: 0.1, max: 2.0 })
                .with_unit(" s"),

            fundamental_balance: FloatParam::new(
                "Fundamental Balance",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            sparkle: FloatParam::new(
                "Sparkle",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),

            click: BoolParam::new("Click", true),

            wave_folder_enabled: BoolParam::new("Wave Folder", false),

            wave_folder_amount: FloatParam::new(
                "Wave Folder Amount",
                1.0,
                FloatRange::Linear { min: 1.0, max: 5.0 },
            ),

            second_voice_enabled: BoolParam::new("Second Voice", false),

            second_voice_detune: FloatParam::new(
                "Second Voice Detune",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_unit(" %"),

            second_voice_stereo_spread: FloatParam::new(
                "Second Voice Stereo Spread",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

pub struct ParamBuffers {
    params: Arc<PockyplockyParams>,
    gain_buffer: [f32; MAX_BLOCK_SIZE],
    noise_level_buffer: [f32; MAX_BLOCK_SIZE],
}

impl ParamBuffers {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params,
            gain_buffer: [0.0; MAX_BLOCK_SIZE],
            noise_level_buffer: [0.0; MAX_BLOCK_SIZE],
        }
    }

    pub fn process_block(&mut self, block_len: usize) {
        self.params
            .gain
            .smoothed
            .next_block(&mut self.gain_buffer[..block_len], block_len);

        self.params
            .noise_level
            .smoothed
            .next_block(&mut self.noise_level_buffer[..block_len], block_len);
    }

    pub fn get_gain_buffer(&self) -> &[f32] {
        &self.gain_buffer
    }

    pub fn get_noise_level_buffer(&self) -> &[f32] {
        &self.noise_level_buffer
    }
}
