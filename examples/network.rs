//! Make real-time changes to a network while it is playing.
#![allow(clippy::precedence)]

use std::time::Duration;

use bevy::prelude::*;
use bevy_audio_studio::player::{PlayerSequencer, sequencer_callback};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, SizedSample};
use fundsp::prelude64::*;
use funutd::*;

#[derive(Debug, Component)]
pub struct NetRnd(pub Rnd);

#[derive(Component, Deref, DerefMut)]
pub struct FundspNet(pub Net);

#[derive(Debug, Component)]
pub struct NetNodes(pub Vec<NodeId>);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource({
            let host = cpal::default_host();

            let device: cpal::Device = host
                .default_output_device()
                .expect("Failed to find a default output device");

            let supported_config = device.default_output_config().unwrap();

            let mut config = supported_config.config();

            config.buffer_size = BufferSize::Fixed(256);
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
        .add_systems(Startup, set_up)
        .add_systems(Update, update)
        .run();
}

fn set_up(mut commands: Commands, mut player: ResMut<PlayerSequencer>) {
    let (mut net, rnd, nodes) = match player.supported_config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(player.config.sample_rate as f64).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(player.config.sample_rate as f64).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(player.config.sample_rate as f64).unwrap(),
        _ => panic!("Unsupported format"),
    };

    let _nodeid = player.push_relative(
        0.0,
        f64::INFINITY,
        Fade::Smooth,
        0.2,
        0.2,
        Box::new(net.backend()),
    );

    commands.spawn((net, rnd, nodes));

    commands
        .delayed()
        .duration(Duration::from_millis(10000))
        .write_message(AppExit::Success);
}

fn update(
    mut query: Query<(&mut FundspNet, &mut NetRnd, &NetNodes)>,
    time: Res<Time>,
    mut timer: Local<bevy::time::Timer>,
    mut delay_added: Local<bool>,
    mut filter_added: Local<bool>,
) {
    let (mut net, mut rnd, nodes) = query.single_mut().unwrap();

    if timer.is_finished() {
        // 随机设置下一次触发间隔
        let ms = rnd.0.u64_in(200, 500) as u64;
        timer.set_duration(Duration::from_millis(ms));
        timer.reset();
    }
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let id_noise = nodes.0[0];
    let id_pan = nodes.0[1];

    if rnd.0.bool(0.5) {
        // Fade to brown noise.
        net.crossfade(
            id_noise,
            Fade::Smooth,
            0.2,
            Box::new(brown() * lfo(|t| 0.5 * exp(-t * 2.0))),
        );
    } else if rnd.0.bool(0.5) {
        // Fade to pink noise.
        net.crossfade(
            id_noise,
            Fade::Smooth,
            0.1,
            Box::new(white() * lfo(|t| 0.5 * exp(-t * 5.0))),
        );
    }
    if rnd.0.bool(0.5) {
        // Note: settings are always applied (or sent to the backend) immediately,
        // without waiting for the next commit.
        net.set(Setting::pan(rnd.0.f32_in(-1.0, 1.0)).node(id_pan));
    }
    if !*delay_added && rnd.0.bool(0.1) {
        // Add a feedback delay.
        let id_delay = net.push(Box::new(
            pass() & feedback(delay(0.2) * db_amp(-3.0) >> pinkpass() >> highpole_hz(100.0)),
        ));
        net.pipe_all(id_noise, id_delay);
        net.pipe_all(id_delay, id_pan);
        *delay_added = true;
    }

    if !*filter_added && rnd.0.bool(0.05) {
        // We can also use the graph syntax to make changes. Connectivity can be temporarily altered,
        // as long as the network has the same number of inputs and outputs at commit time.
        net.0
            .chain(Box::new(peak_hz(1000.0, 2.0) | peak_hz(1000.0, 2.0)));

        *filter_added = true;
    }

    // We don't know whether we made any changes but an empty commit is harmless.
    net.0.commit();
}

fn run<T>(sample_rate: f64) -> Result<(FundspNet, NetRnd, NetNodes), anyhow::Error>
where
    T: SizedSample + FromSample<f64>,
{
    let mut net = Net::new(0, 2);

    let id_noise = net.chain(Box::new(zero()));
    let id_pan = net.chain(Box::new(pan(0.0)));

    net.set_sample_rate(sample_rate);

    let rnd = Rnd::from_u64(1);

    Ok((
        FundspNet(net),
        NetRnd(rnd),
        NetNodes([id_noise, id_pan].to_vec()),
    ))
}
