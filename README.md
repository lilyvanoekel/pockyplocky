cargo xtask bundle pockyplocky --release

RUSTFLAGS="-C target-cpu=native" cargo xtask bundle pockyplocky --release

cp -r ./target/bundled/pockyplocky.clap /Library/Audio/Plug-Ins/CLAP

To do:

- Measure different points of xylophone and see if there's a pattern of diverging ratios
- Other... overtone... mod... things
  - Things to do with amplitude
  - Nudge the ratios a bit somehow "make more inharmonic?"
- Delay based reverby thing? or other delay effect
- Update the exciter
- Velocity sensitive noise? Low pass filter noise?
