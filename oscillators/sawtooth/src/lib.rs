#[macro_use] extern crate vst2;

mod sawtooth_oscillator;

use sawtooth_oscillator::SawtoothOscillator;
use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Plugin, Info, CanDo};
use vst2::event::Event;
use vst2::api::{Supported, Events};

/// Convert the midi note's pitch into the equivalent frequency.
///
/// This function assumes A4 is 440hz.
fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    (((pitch as i8 - A4_PITCH) as f64) / 12.).exp2() * A4_FREQ
}

struct SawtoothSynth {
    sawtooth_oscillator: SawtoothOscillator,
    sample_rate: f64,
    note_duration: f64,
    note: Option<u8>,
}

impl SawtoothSynth {
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
        self.note = Some(note);
        self.sawtooth_oscillator.change_frequency(midi_pitch_to_freq(note));
    }

    fn note_off(&mut self, note: u8) {
        if self.note == Some(note) {
            self.note = None;
        }
    }
}

impl Default for SawtoothSynth {
    fn default() -> SawtoothSynth {
        SawtoothSynth {
            sawtooth_oscillator: SawtoothOscillator::new(),
            sample_rate: 44100.0,
            note_duration: 0.0,
            note: None,
        }
    }
}

impl Plugin for SawtoothSynth {
    fn get_info(&self) -> Info {
        Info {
            name: "sawtooth-oscillator".to_string(),
            vendor: "crsaracco".to_string(),
            unique_id: 1147000002, // Make sure this is a unique number across all of your VSTs!
            category: Category::Synth,
            inputs: 2,
            outputs: 2,
            parameters: 0,
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
        let output_channels = buffer.output_count();
        let num_samples = buffer.samples();
        let (_, output_buffer) = buffer.split();

        // Precompute the samples that should go to each channel.
        // Our oscillator will output the same signal to all channels.
        let mut samples: Vec<f64> = Vec::new();
        if let Some(_) = self.note {
            for _ in 0..(num_samples) {
                samples.push(self.sawtooth_oscillator.next_sample(self.sample_rate));
            }
        }
            else {
                for _ in 0..(num_samples) {
                    // NOTE: You want to use some sort of envelope for real music use, otherwise you
                    // will get clicks at the start and end of playback.
                    samples.push(0.0);
                }
            }

        // Write the output to each channel.
        for channel in 0..output_channels {
            let output_channel = output_buffer.get_mut(channel);
            let mut sample_counter = 0;
            for output_sample in output_channel {
                *output_sample = samples[sample_counter] as f32;
                sample_counter += 1;
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }
}

plugin_main!(SawtoothSynth);