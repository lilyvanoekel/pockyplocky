use std::sync::Arc;

use nih_plug::prelude::*;

use crate::constants::MAX_BLOCK_SIZE;

#[derive(Params)]
pub struct PockyplockyParams {
    // Main params
    #[id = "volume"]
    pub volume: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "timbre"]
    pub timbre: EnumParam<Timbre>,

    // Exciter Params
    #[id = "strike"]
    pub strike: BoolParam,
    #[id = "mallet"]
    pub mallet: BoolParam,
    #[id = "breath_level"]
    pub breath_level: FloatParam,
    #[id = "breath_decay"]
    pub breath_decay: FloatParam,

    // Tweaking the modes
    #[id = "fundamental_balance"]
    pub fundamental_balance: FloatParam,
    #[id = "sparkle"]
    pub sparkle: FloatParam,

    // Effects

    // Wave Folder
    #[id = "wave_folder_enabled"]
    pub wave_folder_enabled: BoolParam,
    #[id = "wave_folder_amount"]
    pub wave_folder_amount: FloatParam,

    // Second voice
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
            volume: FloatParam::new(
                "Volume",
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

            decay: FloatParam::new("Decay", 0.461, FloatRange::Linear { min: 0.1, max: 2.0 })
                .with_unit(" s"),

            timbre: EnumParam::new("Timbre", Timbre::Xylophone),

            strike: BoolParam::new("Strike", false),

            mallet: BoolParam::new("Mallet", true),

            breath_level: FloatParam::new(
                "Breath Level",
                0.1,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            breath_decay: FloatParam::new(
                "Breath Decay",
                400.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 500.0,
                },
            )
            .with_unit(" ms"),

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
            .volume
            .smoothed
            .next_block(&mut self.gain_buffer[..block_len], block_len);

        self.params
            .breath_level
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
