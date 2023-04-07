#pragma once
/* ========================================
 *  Adapted from Airwindows code to  learn from - Ardura
 *  Created 8/12/11 by SPIAdmin
 *  Copyright (c) 2011 __MyCompanyName__, Airwindows uses the MIT license
 * ======================================== */

#ifndef __subhoofer_H
#define __subhoofer_H

#ifndef __audioeffect__
#include "audioeffectx.h"
#endif

#include <set>
#include <string>
#define _USE_MATH_DEFINES
#include <math.h>

enum {
	kParamA = 0,	
	kParamB = 1,
	kParamD = 2,
	kParamC = 3,
	//kParamE = 4,
	kParamF = 4,
	//kParamG = 6,
	kParamH = 5,
	kNumParameters = 6
}; //

const int kNumPrograms = 0;
const int kNumInputs = 2;
const int kNumOutputs = 2;
const unsigned long kUniqueId = 'subh';    //Change this to what the AU identity is!

class subhoofer :
	public AudioEffectX
{
public:
	subhoofer(audioMasterCallback audioMaster);
	~subhoofer();
	virtual bool getEffectName(char* name);                       // The plug-in name
	virtual VstPlugCategory getPlugCategory();                    // The general category for the plug-in
	virtual bool getProductString(char* text);                    // This is a unique plug-in string provided by Steinberg
	virtual bool getVendorString(char* text);                     // Vendor info
	virtual VstInt32 getVendorVersion();                          // Version number
	virtual void processReplacing(float** inputs, float** outputs, VstInt32 sampleFrames);
	//virtual void calcBiQuad(float frequency, float gain, float filterQ);
	//virtual void processDoubleReplacing(double** inputs, double** outputs, VstInt32 sampleFrames);
	virtual void getProgramName(char* name);                      // read the name from the host
	virtual void setProgramName(char* name);                      // changes the name of the preset displayed in the host
	virtual VstInt32 getChunk(void** data, bool isPreset);
	virtual VstInt32 setChunk(void* data, VstInt32 byteSize, bool isPreset);
	virtual float getParameter(VstInt32 index);                   // get the parameter value at the specified index
	virtual void setParameter(VstInt32 index, float value);       // set the parameter at index to value
	virtual void getParameterLabel(VstInt32 index, char* text);  // label for the parameter (eg dB)
	virtual void getParameterName(VstInt32 index, char* text);    // name of the parameter
	virtual void getParameterDisplay(VstInt32 index, char* text); // text description of the current value    
	virtual VstInt32 canDo(char* text);
private:
	char _programName[kVstMaxProgNameLen + 1];
	std::set< std::string > _canDo;

	uint32_t fpdL;
	uint32_t fpdR;
	//default stuff

	// High Pass Calculation
	float hp_alpha = 2.0 * M_PI * 20.0 / getSampleRate();
	float hp_a0 = 0.0;
	float hp_a1 = (1 - exp(-hp_alpha)) / (1 + exp(-hp_alpha));
	float hp_b0 = (1 + hp_a1) / 2.0;
	float hp_b1 = -hp_b0;
	float hp_x1 = 0;
	float hp_y1 = 0;

	// Tilt EQ
	float gain1;
	float gain2;
	float highGainT;
	float lowGainT;
	float lp_outL;
	float lp_outR;
	double denorm = (1.0 / 4294967295.0);
	float amplitude = 0.0f;

	// Biquad filter coefficients for TILT EQ
	double b0;
	double b1;
	double b2;
	double a0;
	double a1;
	double a2;

	// Parameter memory
	float lastF;
	float previousC;

	// Feedback/Feedforward variables
	double inputLPrev;
	double inputRPrev;
	double tempL;
	double tempR;
	double outputLPrev;
	double outputRPrev;

	double lastSampleL;
	double last2SampleL;
	double lastSampleR;
	double last2SampleR;

	double iirDriveSampleA;
	double iirDriveSampleB;
	double iirHeadBumpA;
	double iirHeadBumpB;
	double iirHeadBumpC;

	// Lowpass variables
	double lowpassSampleLAA;
	double lowpassSampleLBA;
	double lowpassSampleLCA;
	double lowpassSampleLG;
	double lowpassSampleRAA;
	double lowpassSampleRBA;
	double lowpassSampleRCA;
	double lowpassSampleRG;

	// Samples for bass reduction
	double iirSampleA;
	double iirSampleB;
	double iirSampleC;
	double iirSampleD;
	double iirSampleE;
	double iirSampleF;
	double iirSampleG;
	double iirSampleH;
	double iirSampleI;
	double iirSampleJ;
	double iirSampleK;
	double iirSampleL;
	double iirSampleM;
	double iirSampleN;
	double iirSampleO;
	double iirSampleP;
	double iirSampleQ;
	double iirSampleR;
	double iirSampleS;
	double iirSampleT;
	double iirSampleU;
	double iirSampleV;
	double iirSampleW;
	double iirSampleX;
	double iirSampleY;
	double iirSampleZ;
	
	double iirSubBumpA;
	double iirSubBumpB;
	double iirSubBumpC;

	double oscGate;

	double iirDriveSampleC;
	double iirDriveSampleD;
	double iirDriveSampleE;
	double iirDriveSampleF;

	// Sub logic
	bool WasNegative;
	bool SubOctave;

	// Counter for octave
	int bflip;

	// Inputs
	float A;
	float B;
	float C;
	float D;

	float F;

	float H;

	double randD;
	double invrandD;
};

#endif