use nih_plug::prelude::*;

#[derive(Params)]
pub struct SinewhiskParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "amp_atk"]
    pub amp_attack_ms: FloatParam,
    #[id = "amp_rel"]
    pub amp_release_ms: FloatParam,
    #[id = "filter_res"]
    pub filter_resonance: FloatParam,
}

impl Default for SinewhiskParams {
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

            amp_attack_ms: FloatParam::new(
                "Attack",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_step_size(0.1)
            .with_unit(" ms"),

            amp_release_ms: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            ),

            filter_resonance: FloatParam::new(
                "Filter Res",
                50.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" Q"),
        }
    }
}
