# Subhoofer

## Description
Subhoofer by Ardura

Subhoofer is a sub and bass enhancement plugin aimed at being a lightweight replacement for other subharmonic generation plugins. Use it to make your bass audible on small speakers or extend the sub range in bass signals! You can also beef up guitars, add bass to other instruments, presence to vocals etc. Experiment!

![image3](https://github.com/ardura/Subhoofer/assets/31751444/fcf389c3-6ec0-4ee3-8f46-39ecb5f312c2)

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

## Examples and comparison
This is a sine wave at C3:
![image1](https://github.com/ardura/Subhoofer/assets/31751444/f7a5e5af-e9c3-4c0f-85db-a4d1e29fc4e1)

Using a Renaissance Bass plugin at default settings:
![image2](https://github.com/ardura/Subhoofer/assets/31751444/5936785b-887a-4f67-92dc-8a6724d10764)

Using Subhoofer at default settings:
![image4](https://github.com/ardura/Subhoofer/assets/31751444/ad67e3ce-736a-4f34-9582-1f0f9376fb10)

Tweaking settings further in Subhoofer - Note the presence of the sub now:
![image5](https://github.com/ardura/Subhoofer/assets/31751444/2325bc5f-c092-48e9-8e71-576fc58ff6b7)

Here are the settings that produced the last example:
![image6](https://github.com/ardura/Subhoofer/assets/31751444/dd42174c-491d-4343-a528-35c4021c2893)

## Known issues
Plugin will shut down if you make it generate a hugely positive signal - don’t do this anyways if you want to keep your speakers/monitors working.

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
