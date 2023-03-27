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
#include <math.h>

enum {
	kParamA = 0,	
	kParamB = 1,
	kParamC = 2,
	kParamD = 3,
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

	double lastSampleL;
	double last2SampleL;
	double lastSampleR;
	double last2SampleR;

	double iirDriveSampleA;
	double iirDriveSampleB;

	//begin EQ
	double iirLowSampleLA;
	double iirLowSampleLB;
	double iirLowSampleLC;
	double iirLowSampleLD;
	double iirLowSampleLE;
	double iirLowSampleL;

	double iirLowSampleRA;
	double iirLowSampleRB;
	double iirLowSampleRC;
	double iirLowSampleRD;
	double iirLowSampleRE;
	double iirLowSampleR;

	double tripletLA;
	double tripletLB;
	double tripletLC;
	double tripletFactorL;

	double tripletRA;
	double tripletRB;
	double tripletRC;
	double tripletFactorR;

	double lowpassSampleLAA;
	double lowpassSampleLAB;
	double lowpassSampleLBA;
	double lowpassSampleLBB;
	double lowpassSampleLCA;
	double lowpassSampleLCB;
	double lowpassSampleLDA;
	double lowpassSampleLDB;
	double lowpassSampleLE;
	double lowpassSampleLF;
	double lowpassSampleLG;

	double lowpassSampleRAA;
	double lowpassSampleRAB;
	double lowpassSampleRBA;
	double lowpassSampleRBB;
	double lowpassSampleRCA;
	double lowpassSampleRCB;
	double lowpassSampleRDA;
	double lowpassSampleRDB;
	double lowpassSampleRE;
	double lowpassSampleRF;
	double lowpassSampleRG;

	bool flip;
	int flipthree;
	//end EQ


	float A;
	float B;
	float C;
	float D;
	//float E;
	float F;
	//float G;
	float H;

	double lp;
	double oscGate;
	bool WasNegative;
	bool SubOctave;

};

#endif