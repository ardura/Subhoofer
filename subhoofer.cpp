/* ========================================
 *  Adapted from Airwindows code to learn from - Ardura
 *  Copyright (c) 2016 airwindows, Airwindows uses the MIT license
 * ======================================== */

#ifndef __subhoofer_H
#include "subhoofer.h"
#endif

AudioEffect* createEffectInstance(audioMasterCallback audioMaster) { return new subhoofer(audioMaster); }

subhoofer::subhoofer(audioMasterCallback audioMaster) :
	AudioEffectX(audioMaster, kNumPrograms, kNumParameters)
{
	// Setup default values for knobs
	A = 0.0; //Hoof, 0 to 1
	B = 0.0; //Sub 0 to 12
	C = 0.5; //Initialize tilt EQ to mid
	D = 1.0; //Lowpass defaults all the way up
	F = 0.4; //TiltFrequency init 153 
	H = 0.5; //OutGain -18 to 18

	lowpassSampleLAA = 0.0;
	lowpassSampleLBA = 0.0;
	lowpassSampleLCA = 0.0;

	lowpassSampleRAA = 0.0;
	lowpassSampleRBA = 0.0;
	lowpassSampleRCA = 0.0;

	lowpassSampleLG = 0.0;
	lowpassSampleRG = 0.0;

	// Removed randomness to reduce noise
	// fpdL and fpdR are used to normalize high/low gains
	fpdL = 0.01; //while (fpdL < 16386) fpdL = rand() * UINT32_MAX;
	fpdR = 0.01; //while (fpdR < 16386) fpdR = rand() * UINT32_MAX;

	_canDo.insert("plugAsChannelInsert"); // plug-in can be used as a channel insert effect.
	_canDo.insert("plugAsSend"); // plug-in can be used as a send effect.
	_canDo.insert("x2in2out");
	setNumInputs(kNumInputs);
	setNumOutputs(kNumOutputs);
	setUniqueID(kUniqueId);
	canProcessReplacing();     // supports output replacing
	canDoubleReplacing();      // supports double precision processing
	programsAreChunks(true);
	vst_strncpy(_programName, "Default", kVstMaxProgNameLen); // default program name
}

subhoofer::~subhoofer() {}
VstInt32 subhoofer::getVendorVersion() { return 1000; }
void subhoofer::setProgramName(char* name) { vst_strncpy(_programName, name, kVstMaxProgNameLen); }
void subhoofer::getProgramName(char* name) { vst_strncpy(name, _programName, kVstMaxProgNameLen); }

static float pinParameter(float data)
{
	if (data < 0.0f) return 0.0f;
	if (data > 1.0f) return 1.0f;
	return data;
}

VstInt32 subhoofer::getChunk(void** data, bool isPreset)
{
	float* chunkData = (float*)calloc(kNumParameters, sizeof(float));
	chunkData[0] = A;		// Sub Amt		//Hoof, 0 to 1
	chunkData[1] = B;		// SubGain		//Sub 0 to 12
	chunkData[2] = D;		// Lowpass		//Lowpass defaults all the way up
	chunkData[3] = C;		// Tilt EQ		//Initialize tilt EQ to mid
	chunkData[4] = F;		// Split Freq	//TiltFrequency init 153 
	chunkData[5] = H;		// Output Gain	//OutGain -18 to 18
	/* Note: The way this is set up, it will break if you manage to save settings on an Intel
	 machine and load them on a PPC Mac. However, it's fine if you stick to the machine you
	 started with. */

	*data = chunkData;
	return kNumParameters * sizeof(float);
}

VstInt32 subhoofer::setChunk(void* data, VstInt32 byteSize, bool isPreset)
{
	float* chunkData = (float*)data;
	A = pinParameter(chunkData[0]);
	B = pinParameter(chunkData[1]);
	D = pinParameter(chunkData[2]);
	C = pinParameter(chunkData[3]);
	F = pinParameter(chunkData[4]);
	H = pinParameter(chunkData[5]);
	/* We're ignoring byteSize as we found it to be a filthy liar */

	/* calculate any other fields you need here - you could copy in
	 code from setParameter() here. */
	return 0;
}

void subhoofer::setParameter(VstInt32 index, float value) {
	switch (index) {
	case kParamA: A = value; break;
	case kParamB: B = value; break;
	case kParamC: C = value; break;
	case kParamD: D = value; break;
	case kParamF: F = value; break;
	case kParamH: H = value; break;
	default: throw; // unknown parameter, shouldn't happen!
	}
}

float subhoofer::getParameter(VstInt32 index) {
	switch (index) {
	case kParamA: return A; break;
	case kParamB: return B; break;
	case kParamC: return C; break;
	case kParamD: return D; break;
	case kParamF: return F; break;
	case kParamH: return H; break;
	default: break; // unknown parameter, shouldn't happen!
	} return 0.0; //we only need to update the relevant name, this is simple to manage
}

void subhoofer::getParameterName(VstInt32 index, char* text) {
	switch (index) {
	case kParamA: vst_strncpy(text, "SubHoof", kVstMaxParamStrLen); break;
	case kParamB: vst_strncpy(text, "SubGain", kVstMaxParamStrLen); break;
	case kParamC: vst_strncpy(text, "TiltEQ", kVstMaxParamStrLen); break;
	case kParamD: vst_strncpy(text, "Lowpass", kVstMaxParamStrLen); break;
	case kParamF: vst_strncpy(text, "TiltFrq", kVstMaxParamStrLen); break;
	case kParamH: vst_strncpy(text, "OutGain", kVstMaxParamStrLen); break;
	default: break; // unknown parameter, shouldn't happen!
	} //this is our labels for displaying in the VST host
}

void subhoofer::getParameterDisplay(VstInt32 index, char* text) {
	switch (index) {
	case kParamA: float2string(A, text, 4); break; //Sub Amt 0-1
	case kParamB: float2string((B * 24.0 ), text, 4); break; //SubGain 0 to 24
	case kParamC: float2string((C * 12.0) - 6.0, text, 0); break; // Tilt EQ -6 to 6
	case kParamD: float2string(D, text, 0); break;
	case kParamF: float2string(floor((F * F * 770.0) + 30.0), text, 5); break; //BassFrq 100.0 log 30 to 1600 defaulting to 100 hz
	case kParamH: float2string(round(((H * 36.0) - 18.0)*100.0)/100.0, text, 4); break; //OutGain -18 to 18
	default: break; // unknown parameter, shouldn't happen!
	} //this displays the values and handles 'popups' where it's discrete choices
}

void subhoofer::getParameterLabel(VstInt32 index, char* text) {
	switch (index) {
	case kParamA: vst_strncpy(text, "", kVstMaxParamStrLen); break;	//kVstMaxParamStrLen is the max label length (7)
	case kParamB: vst_strncpy(text, "dB", 3); break;
	case kParamC: vst_strncpy(text, "", kVstMaxParamStrLen); break;
	case kParamD: vst_strncpy(text, "dB", 0); break;
	case kParamF: vst_strncpy(text, "hz", kVstMaxParamStrLen); break;
	case kParamH: vst_strncpy(text, "dB", 3); break;
	default: break; // unknown parameter, shouldn't happen!
	}
}

VstInt32 subhoofer::canDo(char* text)
{
	return (_canDo.find(text) == _canDo.end()) ? -1 : 1;
} // 1 = yes, -1 = no, 0 = don't know

bool subhoofer::getEffectName(char* name) {
	vst_strncpy(name, "Subhoofer", kVstMaxProductStrLen); return true;
}

VstPlugCategory subhoofer::getPlugCategory() { return kPlugCategEffect; }

bool subhoofer::getProductString(char* text) {
	vst_strncpy(text, "Ardura Subhoofer", kVstMaxProductStrLen); return true;
}

bool subhoofer::getVendorString(char* text) {
	vst_strncpy(text, "Ardura", kVstMaxVendorStrLen); return true;
}