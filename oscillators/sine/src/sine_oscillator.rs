use std::f64;

pub struct SineOscillator {
    frequency: f64,
    phase: f64,
}

impl SineOscillator {
    /// Creates a new Sine wave signal generator.
    pub fn new() -> SineOscillator {
        SineOscillator {
            frequency: 0.0,
            phase: 0.0,
        }
    }

    pub fn change_frequency(&mut self, frequency: f64) {
        self.frequency = frequency;
    }

    pub fn next_sample(&mut self, sample_rate: f64) -> f64 {
        let output = (2.0 * f64::consts::PI * (self.phase)).sin();
        self.phase = (self.phase + self.frequency / sample_rate).fract();

        output
    }
}