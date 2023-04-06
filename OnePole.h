//
//  OnePole.h
//  From https://www.earlevel.com/main/2012/12/15/a-one-pole-filter/
//

#ifndef OnePole_h
#define OnePole_h
#define _USE_MATH_DEFINES
#include <math.h>

class OnePole {
public:
    OnePole() { a0 = 1.0; b1 = 0.0; z1 = 0.0; };
    OnePole(double Fc) { z1 = 0.0; setFc(Fc); };
    ~OnePole();
    void setFc(double Fc);
    float process(double in, double prevOut);

protected:
    double a0, b1, z1;
};

inline void OnePole::setFc(double Fc) {
    // Using HP
    //b1 = -exp(-2.0 * M_PI * (0.5 - Fc));
    //a0 = 1.0 + b1;

    b1 = exp(-2.0 * M_PI * Fc);
    a0 = 1.0 - b1;
}

inline float OnePole::process(double in, double prevOut) {
    return z1 = in * a0 + z1 * b1;
}

#endif