#!/usr/bin/env python3

"""

Script to normalize values into factors, also convenient dumping ground of experimental data.

const BASE_FREQS: [f32; 8] = [
            488.42, 1414.28, 2587.91, 2848.72, 4270.11, 4883.01, 5156.85, 5834.95,
        ];
        const MODAL_AMPS: [f32; 8] = [0.595, 0.115, 0.083, 1.000, 0.133, 0.044, 0.180, 0.338];
        const MODAL_DECAYS: [f32; 8] = [0.461, 0.085, 0.063, 0.044, 0.018, 0.075, 0.023, 0.022];

        // const BASE_FREQS: [f32; 16] = [
        //     484.07, 1451.23, 2863.93, 4254.90, 4874.31, 5189.46, 5526.33, 5863.21, 6678.23,
        //     7308.51, 8427.80, 8992.88, 9699.23, 10253.45, 11459.68, 12644.17,
        // ];

        // const MODAL_AMPS: [f32; 16] = [
        //     0.917, 0.058, 1.000, 0.244, 0.070, 0.087, 0.123, 0.384, 0.061, 0.043, 0.058, 0.052,
        //     0.026, 0.018, 0.060, 0.040,
        // ];

        // const MODAL_DECAYS: [f32; 16] = [
        //     0.476, 0.077, 0.040, 0.020, 0.081, 0.021, 0.036, 0.015, 0.041, 0.020, 0.020, 0.018,
        //     0.015, 0.028, 0.016, 0.012,
        // ];

        // const BASE_FREQS: [f32; 8] = [
        //     488.42, 1450.0, 2595.0, 2860.0, 4285.0, 4895.0, 5180.0, 5870.0,
        // ];

        // const MODAL_AMPS: [f32; 8] = [
        //     1.0,  // fundamental strong
        //     0.3,  // 2nd mode moderate
        //     0.2,  // 3rd
        //     0.25, // 4th
        //     0.1,  // 5th low sparkle
        //     0.05, // 6th very low
        //     0.1,  // 7th soft sparkle
        //     0.08, // 8th faint shimmer
        // ];

"""
values =[0.461, 0.085, 0.063, 0.044, 0.018, 0.075, 0.023, 0.022]

base = values[0]
factors = [v / base for v in values]
print(factors)
