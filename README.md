# Pockyplocky

Pockyplocky is a synthesizer that aims to produce sounds inspired by tuned percussion instruments such as xylophones. It uses modal synthesis to achieve this and offers a number of fixed timbres that can be customized with parameters. It is available as a [CLAP](https://cleveraudio.org/) plugin (althought adding VST3 support is easy).

[pockyplocky.mp3](https://github.com/user-attachments/files/22163208/pockyplocky.mp3)

## Parameters

### Main Controls

- **Volume** - How loud the output is
- **Decay** - How long notes ring out (0.1s to 2.0s)
- **Timbre** - Choose from 8 different instruments

### Exciter Controls

The exciter is what starts the sound. We can start a note with a sharp percussive attack, a mallet strike of configurable hardness or a breath. These options can also be combined. A little bit of breath can add extra dimension to the sound of a Xylophone, for example.

- **Strike** - Sharp, percussive attack
- **Mallet** - Softer, more musical attack, responds to dynamics in playing
- **Mallet Hardness** - How hard the mallet hits (soft to hard)
- **Breath Level** - Add breathy noise to the sound, can make each note played subtle different, combines well with second voice, can also add brightness
- **Breath Attack** - How quickly the breath builds up (0-200ms)
- **Breath Attack Shape** - How the breath attack curves
- **Breath Decay** - How quickly the breath fades (1-500ms)
- **Breath Decay Shape** - How the breath decay curves

### Mode Controls

This synthesizer generates harmonics to simulate various timbres. A mode has a frequency, amplitude and decay. The modes are generally fixed by the timbre selected, but can be slightly customized by these parameters.

- **Fundamental Balance** - More fundamental tone vs more harmonics
- **Sparkle** - Causes higher harmonics to sound for longer (or shorter when negative) relative to the fundamental tone

### Effects

- **Wave Folder** - Add harmonic distortion and complexity, can help lower notes have more body
- **Wave Folder Amount** - How much distortion to apply
- **Second Voice** - Add a detuned second voice for thickness (make-sound-nice)
- **Second Voice Detune** - How much to detune the second voice
- **Second Voice Stereo Spread** - How wide to spread the voices in stereo

## Timbres

The synthesizer includes 8 different instrument sounds:

- **Xylophone** - Classic wooden xylophone sound
- **Bass Xylophone** - Lower-pitched variant
- **Metal Pan** - Metallic percussive sound
- **Glass Marimba** - Glassy, crystalline timbre
- **Piano** - Piano-like percussive attack
- **Wood Blocks** - Wooden block percussion
- **Steel Drum** - Caribbean steel drum sound
- **Metal Cup and Badminton Racquet** - Surprisingly pleasant

## Building

### OSX

RUSTFLAGS="-C target-cpu=native" cargo xtask bundle pockyplocky --release
cp -r ./target/bundled/pockyplocky.clap /Library/Audio/Plug-Ins/CLAP

## Usage

You need a DAW (such as [Reaper](https://www.reaper.fm/), or [Bitwig](https://www.bitwig.com/)) to use this synthesizer. It does not currently have a standalone or web version.

### UI

Currently Pockyplocky does not come with a UI... uh... I mean, Pockyplocky is by design a headless plugin. It will use the default parameter UI as provided by the DAW.
