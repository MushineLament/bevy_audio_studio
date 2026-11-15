use fundsp::prelude64::*;

/// 固定频率的钢琴音色（无包络，持续发声）。
pub fn acoustic_grand_piano_fixed(frequency: f32) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let a1 = 0.9;
    let a2 = 0.55;
    let a3 = 0.42;
    let a4 = 0.38;
    let a5 = 0.25;
    let a6 = 0.13;
    let a7 = 0.02;
    let a8 = 0.015;
    let a9 = 0.01;

    // ✅ 修正 1：用括号包裹整条 & 链，确保低通/高架/去直流作用于所有谐波之和
    // ✅ 修正 2：highshelf_hz(频率, Q, 增益) —— 原代码把参数顺序写反了
    let tone = ((mul(1.0) >> sine() * a1)
        & (mul(2.0) >> sine() * a2)
        & (mul(3.0) >> sine() * a3)
        & (mul(4.0) >> sine() * a4)
        & (mul(5.0) >> sine() * a5)
        & (mul(6.0) >> sine() * a6)
        & (mul(7.0) >> sine() * a7)
        & (mul(8.0) >> sine() * a8)
        & (mul(9.0) >> sine() * a9))
        >> lowpass_hz(400.0, 0.8)
        >> highshelf_hz(2000.0, 0.7, db_amp(-12.0)) // 2000Hz 以上衰减 12dB
        >> dcblock();

    // 琴体共鸣：多个固定频率的谐振器并联
    // pass() 保留干信号，各 resonator 模拟不同的琴体共振峰
    let body = pass()
        & (0.25 * resonator_hz(82.4, 25.0))
        & (0.20 * resonator_hz(110.0, 22.0))
        & (0.18 * resonator_hz(146.8, 20.0))
        & (0.15 * resonator_hz(220.0, 18.0))
        & (0.12 * resonator_hz(293.7, 16.0))
        & (0.10 * resonator_hz(440.0, 14.0))
        & (0.08 * resonator_hz(587.3, 12.0))
        & (0.06 * resonator_hz(880.0, 10.0))
        & (0.04 * resonator_hz(1760.0, 8.0))
        & (0.02 * resonator_hz(800.0, 10.0) * brown());

    dc(frequency) >> tone >> body >> highpass_hz(20.0, 0.7) >> limiter(0.002, 0.06) >> dcblock()
}
