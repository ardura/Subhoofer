# Subhoofer by Ardura
Subhoofer is a sub and bass enhancement plugin aimed at being a lightweight replacement for other subharmonic generation plugins. Use it to make your bass audible on small speakers or extend the sub range in bass signals! You can also beef up guitars, add bass to other instruments, presence to vocals etc. Experiment!

Join the discord! https://discord.com/invite/hscQXkTdfz

Check out the KVR page: https://www.kvraudio.com/product/subhoofer-by-ardura

## Description
Subhoofer can generate bass harmonics as well as a subharmonic in addition to a few saturations aiding it
![Subhoofer_ui](https://github.com/ardura/Subhoofer/assets/31751444/57567b43-3f72-410e-a1ec-6e57af619e87)

KVR Page: https://www.kvraudio.com/product/subhoofer-by-ardura

The default settings are already configured to mimic a bass plugin of renaissance 🙂 However feel free to tweak further!

## Parameters:

● In Gain - Gain before any processing happens

● Out Gain - Gain after all processing happens

● Wet - How much processed sound is there instead of dry/unprocessed sound

● Hardness - Tone control for harmonics - modified Chebyshev algorithm

● Harmonics - Generated harmonics added to the signal

● Harmonic Algorithm - The methods used to generate harmonics:

    ● A Bass New: Ardura's algorithm updated in 2024 for bass enhancement
    
    ● 8 Harmonic Stack: An 8 harmonic stack with a different low-focus than Algorithm 1
    
    ● Duro Console: A Modified 7 harmonic stack from Duro Console that favors non octave harmonics
    
    ● TanH Transfer: Harmonics added in from a tanh transfer function pretending to be tape

    ● Custom: Custom Harmonic Sliders for user to create their own tones
    
● Sub Gain - Gain for the subharmonic generator

● Sub Drive - Send the subharmonic signal to TanH Transfer for subtle Sub harmonics added in

## Installation
Drag the vst3 file into your "C:\Program Files\Common Files\VST3" directory or wherever your vst3 are stored.
Done!

## Examples and comparison
This is a sine wave run through Renaissance Bass and Subhoofer, then to SPAN:
![Screenshot 2024-04-17 100658](https://github.com/ardura/Subhoofer/assets/31751444/2314b7bf-6a81-4d19-9615-2510cdad6a2b)

## Known issues
● xcb flags as a security issue for some unchecked casts and unsafe returns from functions in a dependency

## Building

After installing [Rust](https://rustup.rs/), you can compile Subhoofer as follows:

```
cargo xtask bundle Subhoofer --profile release
```
Or use the following for debugging:
```
cargo xtask bundle Subhoofer --profile profiling
```

This plugin was made possible thanks to the Nih-Plug Rust Library, the egui GUI library, and
Airwindows source code thankfully being MIT licensed which helped me learn. I highly recommend supporting Chris
https://www.airwindows.com/
