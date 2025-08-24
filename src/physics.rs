#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    Rosewood,
    Maple,
    Steel,
    Aluminum,
    Glass,
}

impl Material {
    pub fn youngs_modulus(&self) -> f64 {
        match self {
            Material::Rosewood => 1.6e10,
            Material::Maple => 1.2e10,
            Material::Steel => 2.0e11,
            Material::Aluminum => 7.0e10,
            Material::Glass => 7.0e10,
        }
    }

    pub fn density(&self) -> f64 {
        match self {
            Material::Rosewood => 800.0,
            Material::Maple => 700.0,
            Material::Steel => 7850.0,
            Material::Aluminum => 2700.0,
            Material::Glass => 2500.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mode {
    pub freq: f64, // Hz
    pub amp: f64,  // relative amplitude (unitless)
}

/// Calculate cross-sectional area and area moment of inertia for a rectangular section
pub fn rect_section(b: f64, h: f64) -> (f64, f64) {
    let s = b * h;
    let i = b * h.powi(3) / 12.0;
    (s, i)
}

/// Calculate xylophone modes for a given bar
///
/// # Arguments
/// * `material` - Material of the bar
/// * `l` - Length of the bar (m)
/// * `b` - Width of the bar (m)
/// * `h` - Height/thickness of the bar (m)
pub fn xylophone_modes<const K: usize>(material: Material, l: f64, b: f64, h: f64) -> [Mode; K] {
    let e = material.youngs_modulus();
    let rho = material.density();
    let (s, i) = rect_section(b, h);
    let factor = (std::f64::consts::PI.powi(2)) / (4.0 * l.powi(2)) * ((e * i) / (rho * s)).sqrt();

    let mut modes: [Mode; K] = [Mode {
        freq: 0.0,
        amp: 0.0,
    }; K];

    for n in 0..K {
        let omega = factor * (2 * n + 1).pow(2) as f64;
        let freq = omega / (2.0 * std::f64::consts::PI);
        let amp = 1.0 / (n as f64 + 1.0).powi(2);

        modes[n] = Mode { freq, amp };
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

        // Area moment of inertia: 0.04 * 0.015^3 / 12 = 1.125e-8 m^4
        assert!((i - 1.125e-8).abs() < 1e-10);
    }

    #[test]
    fn test_xylophone_modes_rosewood() {
        // Example: rosewood bar
        let l = 0.25; // m
        let b = 0.04; // m
        let h = 0.015; // m

        let modes = xylophone_modes::<5>(Material::Rosewood, l, b, h);

        // Test first mode (fundamental frequency)
        let mode0 = &modes[0];
        assert!(mode0.freq > 0.0);
        assert!((mode0.amp - 1.0).abs() < 1e-10); // First mode should have amplitude 1.0

        // Test second mode
        let mode1 = &modes[1];
        assert!(mode1.freq > mode0.freq); // Higher frequency
        assert!((mode1.amp - 0.25).abs() < 1e-10); // 1/4 amplitude

        // Test third mode
        let mode2 = &modes[2];
        assert!(mode2.freq > mode1.freq);
        assert!((mode2.amp - 1.0 / 9.0).abs() < 1e-10); // 1/9 amplitude
    }

    #[test]
    fn test_xylophone_modes_frequency_ratios() {
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let modes = xylophone_modes::<3>(Material::Rosewood, l, b, h);

        // For a free-free bar, frequency ratios should follow (2n+1)^2 pattern
        let expected_ratios = [1.0, 9.0, 25.0]; // (2*0+1)^2, (2*1+1)^2, (2*2+1)^2

        for (i, mode) in modes.iter().enumerate() {
            let ratio = mode.freq / modes[0].freq;
            assert!((ratio - expected_ratios[i]).abs() < 0.1); // Allow some tolerance
        }
    }

    #[test]
    fn test_xylophone_modes_amplitude_decay() {
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let modes = xylophone_modes::<4>(Material::Rosewood, l, b, h);

        // Test amplitude pattern: 1, 1/4, 1/9, 1/16
        let expected_amps = [1.0, 0.25, 1.0 / 9.0, 1.0 / 16.0];

        for (i, mode) in modes.iter().enumerate() {
            assert!((mode.amp - expected_amps[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_zero_modes() {
        let l = 0.25;
        let b = 0.04;
        let h = 0.015;

        let modes = xylophone_modes::<0>(Material::Rosewood, l, b, h);

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

    #[test]
    fn print_xylophone_modes() {
        // Example: rosewood bar
        let l = 0.25; // m
        let b = 0.04; // m
        let h = 0.015; // m

        let modes = xylophone_modes::<5>(Material::Rosewood, l, b, h);

        for (idx, m) in modes.iter().enumerate() {
            println!("Mode {}: freq = {:.2} Hz, amp = {:.3}", idx, m.freq, m.amp);
        }
    }
}
