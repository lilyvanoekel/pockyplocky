import numpy as np
import matplotlib.pyplot as plt
from matplotlib.widgets import Button
import librosa
import librosa.effects
from scipy.signal import hilbert, butter, filtfilt, find_peaks
from scipy.optimize import curve_fit

class ModeSelector:
    def __init__(self, y, sr):
        self.y = y
        self.sr = sr
        self.results = []
        self.fig, self.ax = plt.subplots(figsize=(12,4))
        plt.subplots_adjust(bottom=0.2)
        self.freqs = np.fft.rfftfreq(len(y), 1/sr)
        self.spectrum = np.abs(np.fft.rfft(y))
        self.ax.semilogy(self.freqs, self.spectrum)
        self.ax.set_xlabel("Frequency (Hz)")
        self.ax.set_ylabel("Magnitude")
        self.ax.set_title("Click peaks to select modes")
        self.fig.canvas.mpl_connect('button_press_event', self.onclick)
        ax_button = plt.axes([0.8, 0.05, 0.15, 0.075])
        self.button = Button(ax_button, 'Make it so')
        self.button.on_clicked(self.make_it_so)

    def nearest_peak(self, f0, search_hz=200):
        mask = (self.freqs >= f0 - search_hz) & (self.freqs <= f0 + search_hz)
        if not np.any(mask):
            return f0
        local_freqs = self.freqs[mask]
        local_spec = self.spectrum[mask]
        peaks, _ = find_peaks(local_spec)
        if len(peaks) == 0:
            return f0
        best = peaks[np.argmax(local_spec[peaks])]
        return float(local_freqs[best])

    def bandpass_filter(self, f0, width=None):
        ny = self.sr / 2.0
        if width is None:
            width = max(50.0, 0.05 * max(f0, 100.0))
        lowf = max(f0 - width / 2.0, 1.0)
        highf = min(f0 + width / 2.0, ny * 0.999)
        if lowf >= highf:
            return self.y
        low = lowf / ny
        high = highf / ny
        b, a = butter(2, [low, high], btype='band')
        return filtfilt(b, a, self.y)

    def fit_decay(self, y_filt):
        orig_max = np.max(np.abs(y_filt))
        y_norm = y_filt / orig_max if orig_max > 0 else y_filt
        env = np.abs(hilbert(y_norm))
        t = np.arange(len(env)) / self.sr
        def exp_func(t, A, tau):
            return A * np.exp(-t / tau)
        try:
            popt, _ = curve_fit(exp_func, t, env, p0=(env.max(), 0.1), maxfev=20000)
            return float(popt[0] * orig_max), float(popt[1])
        except:
            return float(orig_max), float(np.inf)

    def onclick(self, event):
        if event.inaxes == self.ax and event.button == 1 and event.xdata is not None:
            snapped = self.nearest_peak(event.xdata)
            self.ax.axvline(snapped, color='g', linestyle='-', alpha=0.8)
            self.fig.canvas.draw()
            y_filt = self.bandpass_filter(snapped)
            amp, decay = self.fit_decay(y_filt)
            self.results.append((snapped, amp, decay))
            print(f"Snapped {snapped:.2f} Hz")

    def make_it_so(self, event):
        if not self.results:
            plt.close(self.fig)
            return
        # normalize amplitudes
        max_amp = max(r[1] for r in self.results)
        if max_amp > 0:
            self.results = [(f, a/max_amp, d) for f, a, d in self.results]
        plt.close(self.fig)

def load_audio(filename, duration=2.0, trim_db=40):
    y, sr = librosa.load(filename, sr=None, mono=True)
    y, _ = librosa.effects.trim(y, top_db=trim_db)
    N = int(min(len(y), duration * sr))
    y = y[:N]
    if np.max(np.abs(y)) > 0:
        y = y / np.max(np.abs(y))
    return y, sr

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python pony.py <file.wav>")
        exit(1)
    fn = sys.argv[1]
    y, sr = load_audio(fn)
    selector = ModeSelector(y, sr)
    plt.show()
    if not selector.results:
        print("No modes selected.")
        exit(0)

    base_freq = selector.results[0][0]
    base_decay = selector.results[0][2]

    print(f"\n// Rust arrays (base freq {base_freq:.2f} Hz, base decay {base_decay:.3f}s):")

    print("const FREQ_FACTORS: [f32; NUM_MODES] = [")
    print("    " + ", ".join(f"{f/base_freq:.15f}" for f,_,_ in selector.results) + ",")
    print("];")

    print("const AMP_FACTORS: [f32; NUM_MODES] = [")
    print("    " + ", ".join(f"{a:.3f}" for _,a,_ in selector.results) + ",")
    print("];")

    print("const DECAY_FACTORS: [f32; NUM_MODES] = [")
    print("    " + ", ".join(f"{d/base_decay:.15f}" for _,_,d in selector.results) + ",")
    print("];")
