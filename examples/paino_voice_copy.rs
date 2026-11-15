//! Make real-time changes to a network while it is playing.

use bevy::input::keyboard::KeyboardInput;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_audio_studio::player::{PlayerSequencer, sequencer_callback};
use bevy_audio_studio::sounds::acoustic_grand_piano_fixed;
use cpal::BufferSize;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fundsp::prelude64::*;

#[derive(Component, Deref, DerefMut)]
pub struct FundspNet(pub Net);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource({
            let host = cpal::default_host();

            let device: cpal::Device = host
                .default_output_device()
                .expect("Failed to find a default output device");

            let supported_config = device.default_output_config().unwrap();

            let mut config = supported_config.config();

            config.buffer_size = BufferSize::Fixed(1024);
            let sample_rate = config.sample_rate as f64;
            let channels = config.channels as usize;

            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

            let mut sequencer = Sequencer::new(0, 2, ReplayMode::None);
            sequencer.set_sample_rate(sample_rate);

            let backend = sequencer.backend();

            let stream = match supported_config.sample_format() {
                cpal::SampleFormat::F32 => device.build_output_stream(
                    config.clone(),
                    sequencer_callback::<f32>(channels, backend),
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_output_stream(
                    config.clone(),
                    sequencer_callback::<i16>(channels, backend),
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_output_stream(
                    config.clone(),
                    sequencer_callback::<u16>(channels, backend),
                    err_fn,
                    None,
                ),
                _ => panic!("Unsupported format"),
            }
            .expect("initalize stream error");

            stream.play().unwrap();

            PlayerSequencer::<()>::new(
                device.id().ok(),
                supported_config,
                config,
                stream,
                sequencer,
            )
        })
        .add_systems(Update, update)
        .run();
}

fn update(
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut local: Local<HashMap<usize, EventId>>,
    mut player: ResMut<PlayerSequencer>,
) {
    if keyboard_input.is_empty() {
        return;
    }

    for input in keyboard_input.read() {
        println!("input:{:?}", input);
        let index = match input.key_code {
            KeyCode::KeyQ => 0,
            KeyCode::KeyW => 1,
            KeyCode::KeyE => 2,
            KeyCode::KeyR => 3,
            KeyCode::KeyT => 4,
            KeyCode::KeyY => 5,
            KeyCode::KeyU => 6,
            KeyCode::KeyI => 7,
            KeyCode::KeyO => 8,
            KeyCode::KeyP => 9,
            _ => continue,
        };
        if input.state.is_pressed() {
            let frequency = midi_hz(60.0 + index as f64) as f32;

            let new = player.push_relative(
                0.0,
                f64::INFINITY,
                Fade::Smooth,
                0.01, // 淡入 10ms
                0.1,  // 淡出 100ms
                Box::new(acoustic_grand_piano_fixed(frequency) >> pan(0.0)),
            );

            if let Some(old) = local.remove(&index) {
                player.edit_relative(old, 0.1, 0.1);
            }

            local.insert(index, new);
        } else if let Some(old) = local.remove(&index) {
            player.edit_relative(old, 0.1, 0.1);
        }
    }
}
