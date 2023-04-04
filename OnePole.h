#pragma once
//
//  OnePole.h from https://www.earlevel.com/main/2012/12/15/a-one-pole-filter/ by Nigel Redmon
//

#ifndef OnePole_h
#define OnePole_h

#include <math.h>

class OnePole {
public:
    OnePole() { a0 = 1.0; b1 = 0.0; z1 = 0.0; };
    OnePole(double Fc) { z1 = 0.0; setFc(Fc); };
    ~OnePole();
    void setFc(double Fc);
    // Ardura Added
    void setFcHP(double Fc);
    void setFcShelfish() { a0 = 0.02; b1 = -0.98; z1 = 0.0; };
    float process(float in);

protected:
    double a0, b1, z1;
};

inline void OnePole::setFc(double Fc) {
    b1 = exp(-2.0 * 3.141592654f * Fc);
    a0 = 1.0 - b1;
}

// Ardura Added
inline void OnePole::setFcHP(double Fc) {
    b1 = -exp(-2.0 * 3.141592654f * (0.5 - Fc));
    a0 = 1.0 + b1;
}

inline float OnePole::process(float in) {
    return z1 = in * a0 + z1 * b1;
}

#endif