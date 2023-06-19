# Duro Console
## Description
This is a mixture of a saturator and console “emulation” to do some subtle or not-so-subtle processing to audio in a few flavors I really like.

![image](https://github.com/ardura/Duro-Console/assets/31751444/3d2bf6ea-1b47-4192-9a53-ff5823aca5aa)


## Signal Path
1. Input gain
2. Saturation above threshold w/ drive
3. Console process w/ drive on entire signal
4. Output gain
5. Wet/Dry balance

## Saturation Types
● None - Bypass saturating the signal

● Tape - A tape-like saturation created by adding odd and even harmonics using a transfer

curve based off the hyperbolic tangent that is then softclipped

● Candle - Mimics distortion through a candle flame in front of a microphone

● Chebyshev - Chebyshev polynomial saturation curve with precomputed coefficients to
save processing - sounds tape-like-like but smoother
(https://en.wikipedia.org/wiki/Chebyshev_polynomials )

● Leaf Saturation - A gain > nonlinear curve > gain function I made. A mix of subtle
crunch to square-approaching saturation sound to me.

● Digital Clip - Clip the signal if it exceeds the threshold then mix original signal with
clipped signal based off the drive amount

● Golden Cubic - When above threshold, multiply by the golden ratio and cube the excess
amount then softclip. This is a variation of a really hardcore distortion with some pleasant
sounds at low values.

● Transformer - A transformer saturation implementation, this can mangle a sound
completely or introduce some small distortion, make sure you are using some drive here.
Low drive exaggerates the mangling.

● Odd Harmonics - REALLY LOUD- adds 10 odd harmonics overdrive at drive level

● Odd Harmonics - REALLY LOUD- adds every 4th harmonic to the signal overdriven at
drive level


## Console Types

● Bypass - Bypass adding console processing to the signal

● Neve Inspired - The Airwindows Neverland Tapped Delay Line code with drive linked to
the amount of signal sent through without dithering and denormalization - created from
Neve 1272 impulses (https://www.airwindows.com/neverland/ )

● API Inspired - The Airwindows Apicolypse Tapped Delay Line code with drive linked to
the amount of signal sent through without dithering and denormalization - created from
API 512 impulses (https://www.airwindows.com/apicolypse/ )

● Precision Inspired - The Airwindows Precious Tapped Delay Line code with drive linked
to the amount of signal sent through without dithering and denormalization - created
from Precision 8 impulses (https://www.airwindows.com/precious/ )

● Leaf - A mid-focused console with a less processed sound

● Vine - A console built from random number range then modified further into a subtle
change in tones

● Duro - A simplified console tap built off Vine with a stupid idea to take RAGE = 12463
and left shift that over and over with some other silliness.



## Building

After installing [Rust](https://rustup.rs/), you can compile Duro Console as follows:

```shell
cargo xtask bundle duro_console --profile release
```

This plugin was made possible thanks to the Nih-Plug Rust Library, the egui GUI library, and
Airwindows source code thankfully being MIT licensed. I highly recommend supporting Chris
https://www.airwindows.com/
