#[derive(Debug, Clone, PartialEq)]
pub struct Mode {
    pub freq: f64,  // Hz
    pub amp: f64,   // relative amplitude (unitless)
    pub decay: f64, // decay time (seconds)
}

/// Calculate cross-sectional area and area moment of inertia for a rectangular section
pub fn rect_section(b: f64, h: f64) -> (f64, f64) {
    let s = b * h; // S = cross-sectional area
    let i = b * h.powi(3) / 12.0; // I = area moment of inertia
    (s, i)
}

/// Calculate xylophone modes for a given bar
///
/// # Arguments
/// * `e` - Young's modulus (Pa)
/// * `i` - Area moment of inertia (m^4)
/// * `rho` - Density (kg/m^3)
/// * `s` - Cross-sectional area (m^2)
/// * `l` - Length of the bar (m)
/// * `k` - Number of modes to calculate
/// * `tau0` - Base decay time (seconds)
pub fn xylophone_modes(e: f64, i: f64, rho: f64, s: f64, l: f64, k: usize, tau0: f64) -> Vec<Mode> {
    let mut modes = Vec::new();
    let factor = (std::f64::consts::PI.powi(2)) / (4.0 * l.powi(2)) * ((e * i) / (rho * s)).sqrt();

    for n in 0..k {
        let omega = factor * (2 * n + 1).pow(2) as f64;
        let freq = omega / (2.0 * std::f64::consts::PI);

        let amp = 1.0 / (n as f64 + 1.0).powi(2);
        let decay = tau0 / (n as f64 + 1.0);

        modes.push(Mode { freq, amp, decay });
    }
    modes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_section() {
        let (s, i) = rect_section(0.04, 0.015);

        // Cross-sectional area: 0.04 * 0.015 = 0.0006 m^2
        assert!((s - 0.0006).abs() < 1e-10);

        // Area moment of inertia: 0.04 * 0.015^3 / 12 = 1.125e-9 m^4
        assert!((i - 1.125e-9).abs() < 1e-15);
    }

    #[test]
    fn test_xylophone_modes_rosewood() {
        // Example: rosewood bar
        let e = 1.6e10; // Pa
        let rho = 800.0; // kg/m^3
        let l = 0.25; // m
        let b = 0.04; // m
        let h = 0.015; // m

        let (s, i) = rect_section(b, h);
        let modes = xylophone_modes(e, i, rho, s, l, 5, 2.0);

        assert_eq!(modes.len(), 5);

        // Test first mode (fundamental frequency)
        let mode0 = &modes[0];
        assert!(mode0.freq > 0.0);
        assert!((mode0.amp - 1.0).abs() < 1e-10); // First mode should have amplitude 1.0
        assert!((mode0.decay - 2.0).abs() < 1e-10); // Base decay time

        // Test second mode
        let mode1 = &modes[1];
        assert!(mode1.freq > mode0.freq); // Higher frequency
        assert!((mode1.amp - 0.25).abs() < 1e-10); // 1/4 amplitude
        assert!((mode1.decay - 1.0).abs() < 1e-10); // Half decay time

        // Test third mode
        let mode2 = &modes[2];
        assert!(mode2.freq > mode1.freq);
        assert!((mode2.amp - 1.0 / 9.0).abs() < 1e-10); // 1/9 amplitude
        assert!((mode2.decay - 2.0 / 3.0).abs() < 1e-10); // 2/3 decay time
    }

    #[test]
    fn test_xylophone_modes_frequency_ratios() {
        let e = 1.6e10;
        let rho = 800.0;
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let (s, i) = rect_section(b, h);
        let modes = xylophone_modes(e, i, rho, s, l, 3, 2.0);

        // For a free-free bar, frequency ratios should follow (2n+1)^2 pattern
        let expected_ratios = [1.0, 9.0, 25.0]; // (2*0+1)^2, (2*1+1)^2, (2*2+1)^2

        for (i, mode) in modes.iter().enumerate() {
            let ratio = mode.freq / modes[0].freq;
            assert!((ratio - expected_ratios[i]).abs() < 0.1); // Allow some tolerance
        }
    }

    #[test]
    fn test_xylophone_modes_amplitude_decay() {
        let e = 1.6e10;
        let rho = 800.0;
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let (s, i) = rect_section(b, h);
        let modes = xylophone_modes(e, i, rho, s, l, 4, 3.0);

        // Test amplitude pattern: 1, 1/4, 1/9, 1/16
        let expected_amps = [1.0, 0.25, 1.0 / 9.0, 1.0 / 16.0];
        let expected_decays = [3.0, 1.5, 1.0, 0.75];

        for (i, mode) in modes.iter().enumerate() {
            assert!((mode.amp - expected_amps[i]).abs() < 1e-10);
            assert!((mode.decay - expected_decays[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_zero_modes() {
        let e = 1.6e10;
        let rho = 800.0;
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let (s, i) = rect_section(b, h);
        let modes = xylophone_modes(e, i, rho, s, l, 0, 2.0);

        assert_eq!(modes.len(), 0);
    }

    #[test]
    fn test_rect_section_edge_cases() {
        // Test with very small dimensions
        let (s, i) = rect_section(0.001, 0.001);
        assert!((s - 1e-6).abs() < 1e-12);
        assert!((i - 8.333333333333334e-14).abs() < 1e-20);

        // Test with larger dimensions
        let (s, i) = rect_section(0.1, 0.05);
        assert!((s - 0.005).abs() < 1e-10);
        assert!((i - 1.0416666666666667e-6).abs() < 1e-15);
    }
}
