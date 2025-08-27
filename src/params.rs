use nih_plug::prelude::*;

#[derive(Params)]
pub struct PockyplockyParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "material"]
    pub material: EnumParam<Material>,
    #[id = "filter_res"]
    pub filter_resonance: FloatParam,
    #[id = "noise_level"]
    pub noise_level: FloatParam,
    #[id = "noise_decay"]
    pub noise_decay: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "click"]
    pub click: BoolParam,
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
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),

            decay: FloatParam::new("Decay", 0.461, FloatRange::Linear { min: 0.1, max: 2.0 })
                .with_unit(" s"),

            click: BoolParam::new("Click", true),
        }
    }
}
