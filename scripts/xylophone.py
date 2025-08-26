import math

# Fundamental-to-overtone ratios for different materials
OVERTONE_RATIOS = {
    'wood':  [1.0, 2.76, 5.40, 8.91, 12.02, 16.1, 19.5, 23.0],
    'glass': [1.0, 2.78, 5.45, 8.95, 12.1, 16.2, 19.6, 23.1],
    'metal': [1.0, 2.74, 5.35, 8.85, 11.95, 16.0, 19.4, 22.9]
}

# Strike-point relative amplitudes per material
AMP_FACTORS = {
    'wood':  [1.0, 0.95, 0.64, 0.26, 0.07, 0.27, 0.32, 0.24],
    'glass': [1.0, 0.21, 0.71, 0.49, 0.26, 0.04, 0.13, 0.23],
    'metal': [1.0, 1.0, 0.76, 0.43, 0.10, 0.15, 0.29, 0.32]
}

# Decay times (t60) per material
DECAY_TIMES = {
    'wood':  [0.8, 0.29, 0.15, 0.09, 0.06, 0.04, 0.03, 0.02],
    'glass': [1.5, 0.55, 0.28, 0.17, 0.11, 0.08, 0.06, 0.04],
    'metal': [3.0, 1.15, 0.60, 0.36, 0.23, 0.17, 0.13, 0.10]
}

def generate_modes(material, fundamental_freqs):
    all_bars = []
    for f0 in fundamental_freqs:
        modes = []
        for i in range(8):
            freq = f0 * OVERTONE_RATIOS[material][i]
            amp  = AMP_FACTORS[material][i]
            t60  = DECAY_TIMES[material][i]
            modes.append({'f': freq, 'amp': amp, 't60': t60})
        all_bars.append(modes)
    return all_bars

def print_rust_array(name, all_bars):
    print(f"#[rustfmt::skip]\npub const {name.upper()}_MODES: [[Mode; 8]; 88] = [")
    for modes in all_bars:
        line = "    ["
        line += ", ".join(f"Mode {{ f: {m['f']:.2f}, amp: {m['amp']:.3f}, t60: {m['t60']:.3f} }}" for m in modes)
        line += "],"
        print(line)
    print("];")

if __name__ == "__main__":
    # 88-key piano frequencies (A0=27.5Hz)
    A0 = 27.5
    freqs = [A0 * 2**(i/12) for i in range(88)]

    print("pub struct Mode { pub f: f32, pub amp: f32, pub t60: f32 }")

    # Generate distinct materials
    for material in ['wood', 'glass', 'metal']:
        bars = generate_modes(material, freqs)
        print_rust_array(material, bars)
