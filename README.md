cargo xtask bundle pockyplocky --release

cp -r ./target/bundled/pockyplocky.clap /Library/Audio/Plug-Ins/CLAP

To do:

- Velocity handling
- Measure different points of xylophone and see if there's a pattern of diverging ratios
- Restructure code to be nicely nice
  - Abstract time?
  - exciter.rs
  - mode... calculator?
- Shimmer!
- Boost fundamental in comparison to overtones
- Other... overtone... mod... things
  - Things to do with amplitude
  - Nudge the ratios a bit somehow "make more inharmonic?"
- Double voice
- Efficiency, vectorizing?
