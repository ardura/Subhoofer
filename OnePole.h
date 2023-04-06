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
    OnePole(double sampleRate, double Fc){ a0 = 1.0; b1 = 0.0; z1 = 0.0; setFc(sampleRate, Fc); };
//    OnePole(double Fc) { z1 = 0.0; setFc(Fc); };
    //OnePole(double sampleRate, double freq);// { z1 = 0.0; setFc(sampleRate, freq); };
    double setFc(double sampleRate, double Fc);
    double process(double in);
    double process_HPF(double in);

protected:
    double a0, b1, z1;
};

//inline void OnePole::setFc(double Fc) {
//    b1 = exp(-2.0 * M_PI * Fc);
//    a0 = 1.0 - b1;
//}

inline double OnePole::setFc(double sr, double Fc) {
    double n = tan(M_PI * Fc / sr);
    return n;
}

inline double OnePole::process(double in) {
    return z1 = in * a0 + z1 * b1;
}

inline double OnePole::process_HPF(double in) {
    z1 = in * a0 + z1 * b1;
    return in - z1;
}
#endif