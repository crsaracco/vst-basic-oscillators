#[macro_use] extern crate vst2;

use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Plugin, Info, CanDo};
use vst2::event::Event;
use vst2::api::{Supported, Events};

use std::f64::consts::PI;

/// Convert the midi note's pitch into the equivalent frequency.
///
/// This function assumes A4 is 440hz.
fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    (((pitch as i8 - A4_PITCH) as f64) / 12.).exp2() * A4_FREQ
}

struct SineSynth {
    sample_rate: f64,
    time: f64,
    note_duration: f64,
    note: Option<u8>,
}

impl SineSynth {
    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => ()
        }
    }

    fn note_on(&mut self, note: u8) {
        self.note_duration = 0.0;
        self.note = Some(note)
    }

    fn note_off(&mut self, note: u8) {
        if self.note == Some(note) {
            self.note = None
        }
    }
}

pub const TAU: f64 = PI * 2.0;
pub const ATTACK_DECAY: f64 = 1024.0 * 4.0;

impl Default for SineSynth {
    fn default() -> SineSynth {
        SineSynth {
            sample_rate: 44100.0,
            note_duration: 0.0,
            time: 0.0,
            note: None,
        }
    }
}

impl Plugin for SineSynth {
    fn get_info(&self) -> Info {
        Info {
            name: "sine-oscillator".to_string(),
            vendor: "crsaracco".to_string(),
            unique_id: 6667,
            category: Category::Synth,
            inputs: 2,
            outputs: 2,
            parameters: 1,
            initial_delay: 0,
            ..Info::default()
        }
    }

    #[allow(unused_variables)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                // More events can be handled here.
                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate as f64;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();

        let per_sample = self.time_per_sample();

        for (input_buffer, output_buffer) in buffer.zip() {
            let mut t = self.time;

            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {
                if let Some(current_note) = self.note {
                    let frequency = midi_pitch_to_freq(current_note);
                    let signal = (t * frequency * TAU).sin();

                    let mut current_amplitude = 1.0;

                    *output_sample = (signal * current_amplitude) as f32;

                    self.note_duration += 1.0;
                } else {
                    *output_sample = 0.0;
                }
                t += per_sample;
            }
        }

        self.time += samples as f64 * per_sample;
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            // Convert to a percentage
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            _ => "".to_string(),
        }
    }
}

plugin_main!(SineSynth);