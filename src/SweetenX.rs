// Converted Airwindows' Sweeten to work like my other algorithm approximation
// Ardura

pub fn process(in_l: f32, in_r: f32, overallscale: f32, drive: f32, harmonic: i32, buffer: &mut [f32; 16]) -> (f32,f32) {
    let mut cycle_end = overallscale.floor() as usize;
    if cycle_end < 1 { cycle_end = 1; }
    if cycle_end > 4 { cycle_end = 4; }

    let out_l;
    let out_r;

    // Process left channel
    let mut sweet_sample = in_l;
    for j in 0..cycle_end {
        let sv = sweet_sample;
        sweet_sample = (sweet_sample + buffer[j]) * 0.5;
        buffer[j] = sv;
    }
    sweet_sample = sweet_sample.powi(harmonic) * drive;
    for j in cycle_end..cycle_end*2 {
        let sv = sweet_sample;
        sweet_sample = (sweet_sample + buffer[j]) * 0.5;
        buffer[j] = sv;
    }
    out_l = in_l - sweet_sample;

    // Process right channel
    sweet_sample = in_r;
    for j in 8..(8 + cycle_end) {
        let sv = sweet_sample;
        sweet_sample = (sweet_sample + buffer[j]) * 0.5;
        buffer[j] = sv;
    }
    sweet_sample = sweet_sample.powi(harmonic) * drive;
    for j in 8 + cycle_end..8 + cycle_end*2 {
        let sv = sweet_sample;
        sweet_sample = (sweet_sample + buffer[j]) * 0.5;
        buffer[j] = sv;
    }
    out_r = in_r - sweet_sample;

    (out_l, out_r)
}
