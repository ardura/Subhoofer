# Subhoofer

## Description
Subhoofer by Ardura

Subhoofer is a sub and bass enhancement plugin aimed at being a lightweight replacement for other subharmonic generation plugins. Use it to make your bass audible on small speakers or extend the sub range in bass signals! You can also beef up guitars, add bass to other instruments, presence to vocals etc. Experiment!

The default settings are already configured to mimic a bass plugin of renaissance 🙂 However feel free to tweak further!

## Parameters:
● In Gain - Gain before any processing happens
● Out Gain - Gain after all processing happens
● Wet - How much processed sound is there instead of dry/unprocessed sound
● Hardness - Tone control for harmonics - modified Chebyshev algorithm
● Harmonics - Generated harmonics added to the signal
● Harmonic Algorithm - The methods used to generate harmonics:
    ● 1: An approximation of the first 4 harmonics with varying sin/cos amplitudes - sounds like a renaissance
    ● 2: An 8 harmonic stack with a different low-focus than Algorithm 1
    ● 3: A Modified 7 harmonic stack from Duro Console that favors non octave harmonics
    ● 4. Harmonics added in from a tanh transfer function pretending to be tape
● Sub Gain - Gain for the subharmonic generator
● Sub Drive - Send the subharmonic signal to Harmonic Algorithm 4 for subtle harmonics

## Installation
Drag the vst3 file into your "C:\Program Files\Common Files\VST3" directory or wherever your vst3 are stored.
Done!
```
  ((        ))
   \\      //
 _| \\____// |__
\~~/ ~    ~\/~~~/
 -(|    _/o  ~.-
   /  /     ,|
  (~~~)__.-\ |
   ``-     | |
    |      | |
    |        |
    ```

## Building

After installing [Rust](https://rustup.rs/), you can compile Subhoofer as follows:

```shell
cargo xtask bundle Subhoofer --profile release
    or use the following for debugging
cargo xtask bundle Subhoofer --profile profiling
```

This plugin was made possible thanks to the Nih-Plug Rust Library, the egui GUI library, and
Airwindows source code thankfully being MIT licensed which helped me learn. I highly recommend supporting Chris
https://www.airwindows.com/
