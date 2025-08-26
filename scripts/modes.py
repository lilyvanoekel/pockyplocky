import math

# --- Material properties ---
materials = {
    "Wood": {"E": 1.0e10, "rho": 700.0, "eta": 0.005, "tune": 2**(33/1200)},
    "Glass": {"E": 9.0e10, "rho": 2200.0, "eta": 0.001, "tune": 2**(323/1200) / 2},       
    "Metal": {"E": 6.5e10, "rho": 2800.0, "eta": 0.0005, "tune": 2**(813/1200) / 2},
}

beta_l = [4.730, 7.853, 10.996, 14.137, 17.279, 20.521, 23.763, 27.005]  
MAX_MODES = 8
BAR_WIDTH = 0.03
BAR_HEIGHT = 0.02
L_BASE = 0.3
F_BASE = 440.0

def length_for_note(midi_note, material):
    freq = 440.0 * 2**((midi_note - 69)/12)
    length = L_BASE * math.sqrt(F_BASE / freq)
    return length / math.sqrt(material.get("tune", 1.0))  # shift length for tuning

def generate_modes(length, material, strike_pos=0.25, strike_strength=1.0):
    E, rho, eta = material["E"], material["rho"], material["eta"]
    a = BAR_WIDTH * BAR_HEIGHT
    I = BAR_WIDTH * BAR_HEIGHT**3 / 12.0
    flex = math.sqrt(E * I / (rho * a))
    x = max(0.0, min(strike_pos / length, 1.0))
    modes = []
    max_amp = 0.0
    for n in range(MAX_MODES):
        beta = beta_l[n]
        f = (beta**2) / (2.0 * math.pi * length**2) * flex
        shape = abs(math.sin(math.pi * (n+1) * x))
        amp = strike_strength * shape / math.sqrt(f)
        
        if material.get("name") == "Glass":
            if n == 0:
                pass
            elif n == 1:
                amp *= 0.2
            else:
                amp *= 0.8

        t60 = 2.2 / (eta * f)
        max_amp = max(max_amp, amp)
        modes.append([f, amp, t60])
    
    if max_amp > 0.0:
        for m in modes:
            m[1] /= max_amp
    return modes


# --- Generate Rust code ---
print("pub struct Mode { pub f: f32, pub amp: f32, pub t60: f32 }")

for mat_name, mat_props in materials.items():
    print(f"\n#[rustfmt::skip]\npub const {mat_name.upper()}_MODES: [[Mode; {MAX_MODES}]; 88] = [")
    for midi_note in range(21, 109):
        mat_props["name"] = mat_name
        length = length_for_note(midi_note, mat_props)
        modes = generate_modes(length, mat_props)
        mode_entries = ", ".join([f"Mode {{ f: {m[0]:.2f}, amp: {m[1]:.3f}, t60: {m[2]:.3f} }}" for m in modes])
        print(f"    [{mode_entries}],")
    print("];")
