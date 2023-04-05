#pragma once
// Filter from talking with ChatGPT isolated via OnePole.h structure I found in another dsp forum and saved
// This should turn into a proper reusable header
// Ardura 4-5-2023

#ifndef OnePole_h
#define OnePole_h

class OnePole {
public:
    OnePole() { 
        input = 0.0;
        a0 = 1.0; 
        a1 = 0.0;
        a2 = 0.0;
        b0 = 0.0;
        b1 = 0.0;
        b2 = 0.0;
    };
    ~OnePole();
    void calculateNew(double cutoffFreq, double filterGain, double sampleRate);
    double process(double input, double &inputPrev, double &outputPrev);

protected:
    double a0, a1, a2, b0, b1, b2, input;
};

inline void OnePole::calculateNew(double cutoffFreq, double filterGain, double sampleRate) {
    double filterAmp = pow(10, filterGain / 40);  // Convert gain from dB to amplitude ratio
    double w0 = 2 * 3.141592654f * cutoffFreq / sampleRate;
    double sinw0 = sin(w0);
    double cosw0 = cos(w0);
    double alpha = sinw0 / (2.0 * sqrt(filterAmp));

    // Calculate the filter coefficients
    b0 = filterAmp * ((filterAmp + 1) - (filterAmp - 1) * cosw0 + 2 * sqrt(filterAmp) * alpha);
    b1 = 2 * filterAmp * ((filterAmp - 1) - (filterAmp + 1) * cosw0);
    b2 = filterAmp * ((filterAmp + 1) - (filterAmp - 1) * cosw0 - 2 * sqrt(filterAmp) * alpha);
    a0 = (filterAmp + 1) + (filterAmp - 1) * cosw0 + 2 * sqrt(filterAmp) * alpha;
    a1 = -2 * ((filterAmp - 1) + (filterAmp + 1) * cosw0);
    a2 = (filterAmp + 1) + (filterAmp - 1) * cosw0 - 2 * sqrt(filterAmp) * alpha;
}

inline double OnePole::process(double input, double &inputPrev, double &outputPrev) {
    double output = (b0 / a0) * input + (b1 / a0) * inputPrev + (b2 / a0) * outputPrev - (a1 / a0) * inputPrev - (a2 / a0) * outputPrev;
    return output;
}

#endif
