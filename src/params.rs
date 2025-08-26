use nih_plug::prelude::*;

#[derive(Params)]
pub struct SinewhiskParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "material"]
    pub material: EnumParam<Material>,
    #[id = "filter_res"]
    pub filter_resonance: FloatParam,
    #[id = "mode0_amp"]
    pub mode0_amplitude: FloatParam,
    #[id = "mode1_amp"]
    pub mode1_amplitude: FloatParam,
    #[id = "mode2_amp"]
    pub mode2_amplitude: FloatParam,
    #[id = "mode3_amp"]
    pub mode3_amplitude: FloatParam,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq)]
pub enum Material {
    #[name = "Wood"]
    Wood,
    #[name = "Glass"]
    Glass,
    #[name = "Metal"]
    Metal,
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

            material: EnumParam::new("Material", Material::Wood),

            filter_resonance: FloatParam::new(
                "Filter Res",
                50.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 200.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" Q"),

            mode0_amplitude: FloatParam::new(
                "Mode 0 Amp",
                1.0, // Fundamental (480 Hz, 0 dB)
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mode1_amplitude: FloatParam::new(
                "Mode 1 Amp",
                0.0316, // 968 Hz, -30 dB
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mode2_amplitude: FloatParam::new(
                "Mode 2 Amp",
                0.1, // 1424 Hz, -20 dB
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mode3_amplitude: FloatParam::new(
                "Mode 3 Amp",
                0.178, // 2870 Hz, -15 dB
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}
