use crate::constants::{DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE};
use crate::params;

const EPS: f32 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeCurve {
    Linear,
    Logarithmic,
    Exponential,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Attack,
    Decay,
    Idle,
}

pub struct Envelope {
    sample_rate: f32,
    current_stage: EnvelopeStage,
    current_value: f32,
    target_value: f32,
    steps_left: i32,
    step_size: f32,

    attack_time_ms: f32,
    attack_curve: EnvelopeCurve,
    decay_time_ms: f32,
    decay_curve: EnvelopeCurve,

    envelope_values: [f32; MAX_BLOCK_SIZE],
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            current_stage: EnvelopeStage::Idle,
            current_value: 0.0,
            target_value: 0.0,
            steps_left: 0,
            step_size: 0.0,
            attack_time_ms: 0.0,
            attack_curve: EnvelopeCurve::Linear,
            decay_time_ms: 100.0,
            decay_curve: EnvelopeCurve::Exponential,
            envelope_values: [0.0; MAX_BLOCK_SIZE],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.current_stage = EnvelopeStage::Idle;
        self.current_value = 0.0;
        self.target_value = 0.0;
        self.steps_left = 0;
        self.step_size = 0.0;
    }

    pub fn set_attack_time(&mut self, time_ms: f32) {
        self.attack_time_ms = time_ms;
    }

    pub fn set_attack_curve(&mut self, curve: params::BreathAttackCurve) {
        self.attack_curve = match curve {
            params::BreathAttackCurve::Linear => EnvelopeCurve::Linear,
            params::BreathAttackCurve::Logarithmic => EnvelopeCurve::Logarithmic,
            params::BreathAttackCurve::Exponential => EnvelopeCurve::Exponential,
        };
    }

    pub fn set_decay_time(&mut self, time_ms: f32) {
        self.decay_time_ms = time_ms;
    }

    pub fn set_decay_curve(&mut self, curve: params::BreathDecayCurve) {
        self.decay_curve = match curve {
            params::BreathDecayCurve::Linear => EnvelopeCurve::Linear,
            params::BreathDecayCurve::Exponential => EnvelopeCurve::Exponential,
        };
    }

    fn num_steps(&self, time_ms: f32) -> u32 {
        (self.sample_rate * time_ms * 0.001).round() as u32
    }

    fn step_size(&self, start: f32, target: f32, num_steps: u32, curve: EnvelopeCurve) -> f32 {
        match curve {
            EnvelopeCurve::Linear => (target - start) / num_steps as f32,
            EnvelopeCurve::Logarithmic => {
                ((target + EPS) / (start + EPS)).powf(1.0 / num_steps as f32)
            }
            EnvelopeCurve::Exponential => 0.0001f32.powf(1.0 / num_steps as f32),
        }
    }

    fn next_value(&self, current: f32, target: f32, step_size: f32, curve: EnvelopeCurve) -> f32 {
        match curve {
            EnvelopeCurve::Linear => current + step_size,
            EnvelopeCurve::Logarithmic => (current + EPS) * step_size - EPS,
            EnvelopeCurve::Exponential => (current * step_size) + (target * (1.0 - step_size)),
        }
    }

    pub fn start(&mut self) {
        self.current_stage = EnvelopeStage::Attack;
        self.current_value = 0.0;
        self.target_value = 1.0;

        let num_steps = self.num_steps(self.attack_time_ms);
        self.steps_left = num_steps as i32;
        self.step_size = self.step_size(0.0, 1.0, num_steps, self.attack_curve);
    }

    fn process_sample(&mut self) -> f32 {
        if self.steps_left > 0 {
            let old_steps_left = self.steps_left;
            self.steps_left -= 1;

            let new_value = if old_steps_left == 1 {
                self.target_value
            } else {
                let curve = match self.current_stage {
                    EnvelopeStage::Attack => self.attack_curve,
                    EnvelopeStage::Decay => self.decay_curve,
                    EnvelopeStage::Idle => EnvelopeCurve::Linear,
                };
                self.next_value(self.current_value, self.target_value, self.step_size, curve)
            };

            self.current_value = new_value;
            new_value
        } else {
            match self.current_stage {
                EnvelopeStage::Attack => {
                    self.current_stage = EnvelopeStage::Decay;
                    self.current_value = 1.0;
                    self.target_value = 0.0;

                    let num_steps = self.num_steps(self.decay_time_ms);
                    self.steps_left = num_steps as i32;
                    self.step_size = self.step_size(1.0, 0.0, num_steps, self.decay_curve);

                    self.current_value
                }
                EnvelopeStage::Decay => {
                    self.current_stage = EnvelopeStage::Idle;
                    self.current_value = 0.0;
                    self.steps_left = 0;
                    self.current_value
                }
                EnvelopeStage::Idle => 0.0,
            }
        }
    }

    pub fn process_block(&mut self, block_len: usize) -> &[f32] {
        for i in 0..block_len {
            self.envelope_values[i] = self.process_sample();
        }
        &self.envelope_values[..block_len]
    }
}
