//! Screen-wide warm filter. v1.x only tinted its own window; v2 changes the
//! display gamma ramp, which colors the whole screen like f.lux does.

/// Warmest temperature Windows accepts without extra registry changes.
pub const MIN_KELVIN: f64 = 3400.0;
pub const NEUTRAL_KELVIN: f64 = 6500.0;

/// Maps the 0-100 strength slider to a color temperature.
pub fn kelvin_for_strength(strength: u8) -> f64 {
    let t = f64::from(strength.min(100)) / 100.0;
    NEUTRAL_KELVIN - (NEUTRAL_KELVIN - MIN_KELVIN) * t
}

/// Approximate RGB multipliers (0-1) for a black-body color temperature,
/// normalized so 6500 K is white.
pub fn rgb_multipliers(kelvin: f64) -> [f64; 3] {
    fn raw(kelvin: f64) -> [f64; 3] {
        let t = kelvin / 100.0;
        let red = if t <= 66.0 { 255.0 } else { 329.698_727_446 * (t - 60.0).powf(-0.133_204_759_2) };
        let green = if t <= 66.0 {
            99.470_802_586_1 * t.ln() - 161.119_568_166_1
        } else {
            288.122_169_528_3 * (t - 60.0).powf(-0.075_514_849_2)
        };
        let blue = if t >= 66.0 {
            255.0
        } else if t <= 19.0 {
            0.0
        } else {
            138.517_731_223_1 * (t - 10.0).ln() - 305.044_792_730_7
        };
        [red, green, blue].map(|c| c.clamp(0.0, 255.0) / 255.0)
    }
    let neutral = raw(NEUTRAL_KELVIN);
    let target = raw(kelvin.clamp(1000.0, 40000.0));
    [0, 1, 2].map(|i| (target[i] / neutral[i]).clamp(0.0, 1.0))
}

/// Builds the 3×256 ramp `SetDeviceGammaRamp` expects (red, green, blue).
pub fn gamma_ramp(strength: u8) -> [[u16; 256]; 3] {
    let mult = rgb_multipliers(kelvin_for_strength(strength));
    let mut ramp = [[0u16; 256]; 3];
    for (channel, m) in ramp.iter_mut().zip(mult) {
        for (i, entry) in channel.iter_mut().enumerate() {
            *entry = ((i as f64) * 257.0 * m).round().clamp(0.0, 65535.0) as u16;
        }
    }
    ramp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_strength_is_identity() {
        let ramp = gamma_ramp(0);
        for channel in ramp {
            for (i, v) in channel.iter().enumerate() {
                assert!((i as i64 * 257 - i64::from(*v)).abs() <= 257, "entry {i}");
            }
        }
    }

    #[test]
    fn full_strength_is_warm_and_monotonic() {
        let [r, g, b] = rgb_multipliers(kelvin_for_strength(100));
        assert!(r > g && g > b, "red > green > blue at {MIN_KELVIN}K, got {r} {g} {b}");
        assert!(b > 0.4, "stays within the range Windows accepts");
        let ramp = gamma_ramp(100);
        for channel in ramp {
            assert!(channel.windows(2).all(|w| w[0] <= w[1]));
        }
    }

    #[test]
    fn strength_maps_linearly() {
        assert_eq!(kelvin_for_strength(0), NEUTRAL_KELVIN);
        assert_eq!(kelvin_for_strength(100), MIN_KELVIN);
        assert_eq!(kelvin_for_strength(255), MIN_KELVIN);
    }
}
