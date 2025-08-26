import random

NUM_SAMPLES = 512

burst = [random.uniform(-1.0, 1.0) for _ in range(NUM_SAMPLES)]

print("#[rustfmt::skip]\npub const NOISE_BURST: [f32; {}] = [".format(NUM_SAMPLES))
for i, v in enumerate(burst):
    print(f"    {v:.6f},", end="\n" if (i + 1) % 8 == 0 else " ")
print("];")
