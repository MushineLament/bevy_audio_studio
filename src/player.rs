use std::marker::PhantomData;

use assert_no_alloc::assert_no_alloc;
use bevy::{
    ecs::{component::Component, resource::Resource},
    prelude::{Deref, DerefMut},
};
use cpal::{OutputCallbackInfo, traits::StreamTrait as _};
use fundsp::prelude64::AudioUnit as _;

/// [`Sequencer`](fundsp::sequencer::Sequencer) 将会clone,放入即时播放的 [`Sequencer`] 队列中
#[derive(Component, Deref, DerefMut, Clone)]
pub struct FundspSequencer(pub fundsp::sequencer::Sequencer);

/// [`Sequencer`](fundsp::sequencer::Sequencer) 动态播放队列,
#[derive(Resource, Deref, DerefMut)]
pub struct PlayerSequencer<Mark = ()> {
    pub supported_config: cpal::SupportedStreamConfig,
    pub deviceid: Option<cpal::DeviceId>,
    pub config: cpal::StreamConfig,
    pub stream: cpal::Stream,
    /// 前端[`Sequencer`]
    #[deref]
    pub sequencer: fundsp::sequencer::Sequencer,
    pub _mark: PhantomData<Mark>,
}

impl<Mark> PlayerSequencer<Mark> {
    pub fn new(
        deviceid: Option<cpal::DeviceId>,
        supported_config: cpal::SupportedStreamConfig,
        config: cpal::StreamConfig,
        stream: cpal::Stream,
        mut sequencer: fundsp::sequencer::Sequencer,
    ) -> Self {
        sequencer.set_sample_rate(config.sample_rate as f64);

        Self {
            supported_config,
            deviceid,
            config,
            stream,
            sequencer,
            _mark: PhantomData,
        }
    }
}

impl<Mark> Drop for PlayerSequencer<Mark> {
    fn drop(&mut self) {
        let _ = self.stream.pause(); // ← 先停流，再 drop sequencer
    }
}

/// 当设备 [`Device`](cpal::Device) 丢失时,backend后端也会丢失
pub fn sequencer_callback<T>(
    channels: usize,
    mut backend: fundsp::realseq::SequencerBackend,
) -> impl FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static
where
    T: cpal::SizedSample + cpal::FromSample<f64>,
{
    move |data, _| {
        write_data(data, channels, &mut || {
            assert_no_alloc(|| backend.get_stereo())
        })
    }
}

pub fn write_data<T>(output: &mut [T], channels: usize, mut next_sample: impl FnMut() -> (f32, f32))
where
    T: cpal::SizedSample + cpal::FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0 as f64);
        let right: T = T::from_sample(sample.1 as f64);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
