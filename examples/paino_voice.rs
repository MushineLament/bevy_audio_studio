use bevy_audio_studio::sounds::acoustic_grand_piano_fixed;
use fundsp::prelude64::*;

fn main() {
    let sample_rate = 44100.0;
    let bpm = 120.0;
    let beat = 60.0 / bpm; // 120 BPM → 1 拍 = 0.5 秒

    // 「小星星」前两句：C C G G A A G - | F F E E D D C -
    // (MIDI 音符, 持续拍数)
    let melody: [(u8, f64); 14] = [
        (60, 1.0),
        (60, 1.0),
        (67, 1.0),
        (67, 1.0),
        (69, 1.0),
        (69, 1.0),
        (67, 2.0),
        (65, 1.0),
        (65, 1.0),
        (64, 1.0),
        (64, 1.0),
        (62, 1.0),
        (62, 1.0),
        (60, 2.0),
    ];

    // ── 用 Sequencer 调度旋律 ──
    let mut sequencer = Sequencer::new(0, 2, ReplayMode::None);
    sequencer.set_sample_rate(sample_rate);

    let mut t = 0.0;
    for (note, beats) in melody {
        let freq = midi_hz(note as f64) as f32;
        let dur = beats * beat;

        sequencer.push(
            t,
            t + dur,
            Fade::Smooth,
            0.01, // 淡入 10ms
            0.1,  // 淡出 100ms
            Box::new(acoustic_grand_piano_note(freq, dur as f32) >> pan(0.0)),
        );

        t += dur;
    }

    // 多留 1 秒尾巴，让最后一个音的衰减自然结束
    let total_duration = t + 1.0;

    // ── 渲染 + 检测 ──
    let wave = Wave::render(sample_rate, total_duration, &mut sequencer);

    println!("采样数: {}", wave.length());
    println!("时长: {:.3} 秒", wave.duration());
    println!("声道数: {}", wave.channels());
    let amp = wave.amplitude();
    println!("峰值振幅: {:.6} ({:.2} dBFS)", amp, amp_db(amp as f64));

    // ── 保存 ──
    wave.save_wav16(".tmp/piano_melody.wav").expect("保存失败");
    println!("已保存 piano_melody.wav");
}

/// 带包络的钢琴音符：快速 attack + 指数衰减。
/// 钢琴是敲击乐器，即使不释放也会自然衰减，所以包络是关键。
pub fn acoustic_grand_piano_note(
    frequency: f32,
    duration: f32,
) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let piano = acoustic_grand_piano_fixed(frequency);

    // 包络：5ms attack，之后按指数衰减，时间常数 = duration / 3
    // 指数衰减比线性更接近钢琴的物理特性（琴弦振动能量按指数耗散）
    let env = envelope(move |t: f64| {
        let attack = 0.005;
        if t < attack {
            t / attack
        } else {
            let tau = (duration as f64 / 3.0).max(0.1);
            (-(t - attack) / tau).exp()
        }
    });

    piano * env
}
