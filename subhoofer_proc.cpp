/* ========================================
 *  EQ - EQ.h
 *  Copyright (c) 2016 airwindows, Airwindows uses the MIT license
 * ======================================== */

#ifndef __subhoofer_H
#include "subhoofer.h"
#endif
#include "smbPitchShift.cpp"

void subhoofer::processReplacing(float** inputs, float** outputs, VstInt32 sampleFrames)
{
	float* in1 = inputs[0];
	float* in2 = inputs[1];
	float* out1 = outputs[0];
	float* out2 = outputs[1];

	double overallscale = 1.0;
	overallscale /= 44100.0;
	double compscale = overallscale;
	overallscale = getSampleRate();
	compscale = compscale * overallscale;
	//compscale is the one that's 1 or something like 2.2 for 96K rates

	// Modified by Ardura
	double inputSampleL;
	double inputSampleR;

	double midSampleL = 0.0;
	double bassSampleL = 0.0;

	double midSampleR = 0.0;
	double bassSampleR = 0.0;

	// Added by Ardura
	double subzero = 0.0;
	double subSample = 0.0;
	//double subscale = (((A * 12.0) * 0.1) + 0.02) / overallscale;
	double subscale = ((A * 0.1) + 0.0001) / overallscale;

	double iirAmount = subscale / 44.1;
	//double iirAmount = ((C * 0.33) + 0.1) / overallscale;

	double clamp = 0.0;
	double noise = 0.054;

	double densityC = (C * 24.0) - 12.0;
	bool engageEQ = true;
	if (0.0 == densityC) engageEQ = false;

	densityC = pow(10.0, densityC / 20.0) - 1.0;
	//convert to 0 to X multiplier with 1.0 being O db
	//minus one gives nearly -1 to ? (should top out at 1)
	//calibrate so that X db roughly equals X db with maximum topping out at 1 internally

	double tripletIntensity = -densityC;

	//double iirAmountC = (((D * D * 15.0) + 1.0) * 0.0188) + 0.7;
	//Changed iirAmountC to see if that makes a steeper filter
	//double iirAmountC = (D * D * 4);
	double iirAmountC = (4 * D * D);
	if (iirAmountC > 1.0) iirAmountC = 1.0;
	bool engageLowpass = false;
	if ((D * D * 4) < 4) engageLowpass = true;

	double iirAmountB = (((F * F * 770.0) + 30.0) * 10) / overallscale;
	//bypass the highpass and lowpass if set to extremes
	double bridgerectifier;

	double outC = fabs(densityC);
	//end EQ
	double outputgain = pow(10.0, ((H * 36.0) - 18.0) / 20.0);

	while (--sampleFrames >= 0)
	{
		inputSampleL = *in1;
		inputSampleR = *in2;
		if (fabs(inputSampleL) < 1.18e-23) inputSampleL = fpdL * 1.18e-17;
		if (fabs(inputSampleR) < 1.18e-23) inputSampleR = fpdR * 1.18e-17;

		last2SampleL = lastSampleL;
		lastSampleL = inputSampleL;

		last2SampleR = lastSampleR;
		lastSampleR = inputSampleR;

		flip = !flip;
		flipthree++;
		if (flipthree < 1 || flipthree > 3) flipthree = 1;
		//counters

		//begin EQ
		if (engageEQ)
		{
			switch (flipthree)
			{
			case 1:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLA += tripletFactorL;
				tripletLC -= tripletFactorL;
				tripletFactorL = tripletLA;
				iirLowSampleLC = (iirLowSampleLC * (1.0 - iirAmountB)) + (inputSampleL * iirAmountB);
				bassSampleL = iirLowSampleLC;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRA += tripletFactorR;
				tripletRC -= tripletFactorR;
				tripletFactorR = tripletRA;
				iirLowSampleRC = (iirLowSampleRC * (1.0 - iirAmountB)) + (inputSampleR * iirAmountB);
				bassSampleR = iirLowSampleRC;
				break;
			case 2:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLB += tripletFactorL;
				tripletLA -= tripletFactorL;
				tripletFactorL = tripletLB;
				iirLowSampleLD = (iirLowSampleLD * (1.0 - iirAmountB)) + (inputSampleL * iirAmountB);
				bassSampleL = iirLowSampleLD;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRB += tripletFactorR;
				tripletRA -= tripletFactorR;
				tripletFactorR = tripletRB;
				iirLowSampleRD = (iirLowSampleRD * (1.0 - iirAmountB)) + (inputSampleR * iirAmountB);
				bassSampleR = iirLowSampleRD;
				break;
			case 3:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLC += tripletFactorL;
				tripletLB -= tripletFactorL;
				tripletFactorL = tripletLC;
				iirLowSampleLE = (iirLowSampleLE * (1.0 - iirAmountB)) + (inputSampleL * iirAmountB);
				bassSampleL = iirLowSampleLE;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRC += tripletFactorR;
				tripletRB -= tripletFactorR;
				tripletFactorR = tripletRC;
				iirLowSampleRE = (iirLowSampleRE * (1.0 - iirAmountB)) + (inputSampleR * iirAmountB);
				bassSampleR = iirLowSampleRE;
				break;
			}
			tripletLA /= 2.0;
			tripletLB /= 2.0;
			tripletLC /= 2.0;

			tripletRA /= 2.0;
			tripletRB /= 2.0;
			tripletRC /= 2.0;

			if (flip)
			{
				iirLowSampleLA = (iirLowSampleLA * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
				bassSampleL = iirLowSampleLA;

				iirLowSampleRA = (iirLowSampleRA * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
				bassSampleR = iirLowSampleRA;
			}
			else
			{
				iirLowSampleLB = (iirLowSampleLB * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
				bassSampleL = iirLowSampleLB;

				iirLowSampleRB = (iirLowSampleRB * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
				bassSampleR = iirLowSampleRB;
			}

			iirLowSampleL = (iirLowSampleL * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
			bassSampleL = iirLowSampleL;

			iirLowSampleR = (iirLowSampleR * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
			bassSampleR = iirLowSampleR;

			// Needed to change this to only be for bass
			midSampleL = (inputSampleL - bassSampleL);
			midSampleR = (inputSampleR - bassSampleL);

			//////////////////////////////////////////////////
			// Sub calculation code
			//////////////////////////////////////////////////

			//void smbPitchShift(float pitchShift, long numSampsToProcess, long fftFrameSize, long osamp, float sampleRate, float *indata, float *outdata)

			lp = (inputSampleL * inputSampleL) / 2.0;
			//float output;

			/*
  * smbPitchShift params:
  * 1: "pitchShift"         -> semitones to shift up
  * 2: "bufferLengthFrames" -> number of samples in input buffer must be larger than FFT_SIZE
  * 3: "FFT_SIZE"           -> size of the FFT, needs to be a power of 2
  * 4: "OVER_SAMPLE"        -> fifo buffer overlap factor, more the better but slower, has to be divisable by FFT_SIZE
  * 5: "sampleRate"         -> sample rate for sin generation
  * 6: "input"              -> input buffer
  * 7: "output"             -> output buffer
  */
			//smbPitchShift(pitchShift, bufferLengthFrames, OVER_SAMPLE, sampleRate, input, output);
			//smbPitchShift(0.5, 1, 1, 16, overallscale, lp, output);
			// Gate from airwindows
			oscGate += fabs(lp * 10.0);
			oscGate -= 0.001;
			if (oscGate > 1.0) oscGate = 1.0;
			if (oscGate < 0) oscGate = 0;
			//got a value that only goes down low when there's silence or near silence on input
			clamp = 1.0 - oscGate;
			clamp *= 0.00001;
			//set up the thing to choke off oscillations- belt and suspenders affair

			double rand = (double(fpdL) / UINT32_MAX);
			double invrand = (1 - rand);

			//subSample = lp - ((bassSampleL + bassSampleR)/2.0);

			double iirSampleSub = (subSample * (1.0 - iirAmount)) + (subscale * iirAmount);
			subSample -= iirSampleSub;

			subSample += fabs(subSample);
			subSample -= (subSample * subSample * subSample * subscale);
			subSample = (invrand * subSample) + (rand * subSample) + (rand * subSample);
			if (subSample > 0) subSample -= clamp;
			if (subSample < 0) subSample += clamp;

			//subSample = (subSample * (1.0 - iirAmount)) + (subscale * iirAmount);


			if (B != 0.0)
			{
				bassSampleL += subSample * ( B * 12.0 );
				bassSampleR += subSample * ( B * 12.0 );
			}

			//////////////////////////////////////////////////
			//////////////////////////////////////////////////
			//////////////////////////////////////////////////

			//drive section
			bassSampleL *= (densityC + 1.0);
			bridgerectifier = fabs(bassSampleL) * 1.57079633;
			if (bridgerectifier > 1.57079633) bridgerectifier = 1.57079633;
			//max value for sine function
			if (densityC > 0) bridgerectifier = sin(bridgerectifier);
			else bridgerectifier = 1 - cos(bridgerectifier);
			//produce either boosted or starved version
			if (bassSampleL > 0) bassSampleL = (bassSampleL * (1 - outC)) + (bridgerectifier * outC);
			else bassSampleL = (bassSampleL * (1 - outC)) - (bridgerectifier * outC);
			//blend according to densityC control

			bassSampleR *= (densityC + 1.0);
			bridgerectifier = fabs(bassSampleR) * 1.57079633;
			if (bridgerectifier > 1.57079633) bridgerectifier = 1.57079633;
			//max value for sine function
			if (densityC > 0) bridgerectifier = sin(bridgerectifier);
			else bridgerectifier = 1 - cos(bridgerectifier);
			//produce either boosted or starved version
			if (bassSampleR > 0) bassSampleR = (bassSampleR * (1 - outC)) + (bridgerectifier * outC);
			else bassSampleR = (bassSampleR * (1 - outC)) - (bridgerectifier * outC);
			//blend according to densityC control

			// Summing outputs
			inputSampleL = midSampleL;
			inputSampleL += bassSampleL;

			inputSampleR = midSampleR;
			inputSampleR += bassSampleR;
		}
		//end EQ

		//EQ lowpass is after all processing like the compressor that might produce hash
		if (engageLowpass)
		{
			if (flip)
			{
				lowpassSampleLAA = (lowpassSampleLAA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLAA;
				lowpassSampleLBA = (lowpassSampleLBA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLBA;
				lowpassSampleLCA = (lowpassSampleLCA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLCA;

				lowpassSampleRAA = (lowpassSampleRAA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRAA;
				lowpassSampleRBA = (lowpassSampleRBA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRBA;
				lowpassSampleRCA = (lowpassSampleRCA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRCA;
			}
			else
			{
				lowpassSampleLAB = (lowpassSampleLAB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLAB;
				lowpassSampleLBB = (lowpassSampleLBB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLBB;
				lowpassSampleLCB = (lowpassSampleLCB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLCB;

				lowpassSampleRAB = (lowpassSampleRAB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRAB;
				lowpassSampleRBB = (lowpassSampleRBB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRBB;
				lowpassSampleRCB = (lowpassSampleRCB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRCB;
				
			}
			lowpassSampleLG = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			lowpassSampleRG = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);

			inputSampleL = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			inputSampleR = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
		}

		 
		//built in output trim and dry/wet if desired
		if (outputgain != 1.0) {
			inputSampleL *= outputgain;
			inputSampleR *= outputgain;
		}



		//begin 32 bit stereo floating point dither
		int expon; 
		frexpf((float)inputSampleL, &expon);
		fpdL ^= fpdL << 13; fpdL ^= fpdL >> 17; fpdL ^= fpdL << 5;
		inputSampleL += ((double(fpdL) - uint32_t(0x7fffffff)) * 5.5e-36l * pow(2, expon + 62));
		frexpf((float)inputSampleR, &expon);
		fpdR ^= fpdR << 13; fpdR ^= fpdR >> 17; fpdR ^= fpdR << 5;
		inputSampleR += ((double(fpdR) - uint32_t(0x7fffffff)) * 5.5e-36l * pow(2, expon + 62));
		//end 32 bit stereo floating point dither

		*out1 = inputSampleL;
		*out2 = inputSampleR;

		*in1++;
		*in2++;
		*out1++;
		*out2++;
	}
}
/*
void subhoofer::processDoubleReplacing(double** inputs, double** outputs, VstInt32 sampleFrames)
{
	double* in1 = inputs[0];
	double* in2 = inputs[1];
	double* out1 = outputs[0];
	double* out2 = outputs[1];

	double overallscale = 1.0;
	overallscale /= 44100.0;
	double compscale = overallscale;
	overallscale = getSampleRate();
	compscale = compscale * overallscale;
	//compscale is the one that's 1 or something like 2.2 for 96K rates

	double inputSampleL;
	double inputSampleR;

	double midSampleL = 0.0;
	double bassSampleL = 0.0;

	double midSampleR = 0.0;
	double bassSampleR = 0.0;

	// Added by Ardura
	double subSample = 0.0;
	double subscale = ((A * 0.1) + 0.02) / overallscale;
	double clamp = 0.0;

	double densityC = (C * 24.0) - 12.0;
	bool engageEQ = true;
	if (0.0 == densityC) engageEQ = false;

	densityC = pow(10.0, densityC / 20.0) - 1.0;
	//convert to 0 to X multiplier with 1.0 being O db
	//minus one gives nearly -1 to ? (should top out at 1)
	//calibrate so that X db roughly equals X db with maximum topping out at 1 internally

	double tripletIntensity = -densityC;

	double iirAmountB = (((F * F * 770.0) + 30.0) * 10) / overallscale;
	//bypass the highpass and lowpass if set to extremes
	double bridgerectifier;

	//double iirAmountC = (((D * D * 15.0) + 1.0) * 0.0188) + 0.7;
	//Increased iirAmountC to see if that makes a steeper filter
	double iirAmountC = (((D * D * 19.0) + 1.0) * 0.02) + 0.3;
	if (iirAmountC > 1.0) iirAmountC = 1.0;
	bool engageLowpass = false;
	if (((D * D * 19.0) + 1.0) < 19.99) engageLowpass = true;

	double outC = fabs(densityC);
	//end EQ
	double outputgain = pow(10.0, ((H * 36.0) - 18.0) / 20.0);

	while (--sampleFrames >= 0)
	{
		inputSampleL = *in1;
		inputSampleR = *in2;
		if (fabs(inputSampleL) < 1.18e-23) inputSampleL = fpdL * 1.18e-17;
		if (fabs(inputSampleR) < 1.18e-23) inputSampleR = fpdR * 1.18e-17;

		last2SampleL = lastSampleL;
		lastSampleL = inputSampleL;

		last2SampleR = lastSampleR;
		lastSampleR = inputSampleR;

		flip = !flip;
		flipthree++;
		if (flipthree < 1 || flipthree > 3) flipthree = 1;
		//counters

		//begin EQ
		if (engageEQ)
		{
			switch (flipthree)
			{
			case 1:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLA += tripletFactorL;
				tripletLC -= tripletFactorL;
				tripletFactorL = tripletLA * tripletIntensity;
				bassSampleL = iirLowSampleLC;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRA += tripletFactorR;
				tripletRC -= tripletFactorR;
				tripletFactorR = tripletRA * tripletIntensity;
				bassSampleR = iirLowSampleRC;
				break;
			case 2:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLB += tripletFactorL;
				tripletLA -= tripletFactorL;
				tripletFactorL = tripletLB * tripletIntensity;
				iirLowSampleLD = (iirLowSampleLD * (1.0 - iirAmountB)) + (inputSampleL * iirAmountB);
				bassSampleL = iirLowSampleLD;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRB += tripletFactorR;
				tripletRA -= tripletFactorR;
				tripletFactorR = tripletRB * tripletIntensity;

				iirLowSampleRD = (iirLowSampleRD * (1.0 - iirAmountB)) + (inputSampleR * iirAmountB);
				bassSampleR = iirLowSampleRD;
				break;
			case 3:
				tripletFactorL = last2SampleL - inputSampleL;
				tripletLC += tripletFactorL;
				tripletLB -= tripletFactorL;
				tripletFactorL = tripletLC * tripletIntensity;

				iirLowSampleLE = (iirLowSampleLE * (1.0 - iirAmountB)) + (inputSampleL * iirAmountB);
				bassSampleL = iirLowSampleLE;

				tripletFactorR = last2SampleR - inputSampleR;
				tripletRC += tripletFactorR;
				tripletRB -= tripletFactorR;
				tripletFactorR = tripletRC * tripletIntensity;

				iirLowSampleRE = (iirLowSampleRE * (1.0 - iirAmountB)) + (inputSampleR * iirAmountB);
				bassSampleR = iirLowSampleRE;
				break;
			}
			tripletLA /= 2.0;
			tripletLB /= 2.0;
			tripletLC /= 2.0;

			tripletRA /= 2.0;
			tripletRB /= 2.0;
			tripletRC /= 2.0;

			if (flip)
			{
				iirLowSampleLA = (iirLowSampleLA * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
				bassSampleL = iirLowSampleLA;

				iirLowSampleRA = (iirLowSampleRA * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
				bassSampleR = iirLowSampleRA;
			}
			else
			{
				iirLowSampleLB = (iirLowSampleLB * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
				bassSampleL = iirLowSampleLB;

				iirLowSampleRB = (iirLowSampleRB * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
				bassSampleR = iirLowSampleRB;
			}

			iirLowSampleL = (iirLowSampleL * (1.0 - iirAmountB)) + (bassSampleL * iirAmountB);
			bassSampleL = iirLowSampleL;

			iirLowSampleR = (iirLowSampleR * (1.0 - iirAmountB)) + (bassSampleR * iirAmountB);
			bassSampleR = iirLowSampleR;

			// Needed to change this to only be for bass
			midSampleL = (inputSampleL - bassSampleL);
			midSampleR = (inputSampleR - bassSampleR);


			//////////////////////////////////////////////////
			// Sub calculation code
			//////////////////////////////////////////////////

			lp = (bassSampleL * bassSampleR) / 2;
			oscGate += fabs(lp * 10.0);
			oscGate -= 0.001;
			if (oscGate > 1.0) oscGate = 1.0;
			if (oscGate < 0) oscGate = 0;
			//got a value that only goes down low when there's silence or near silence on input
			clamp = 1.0 - oscGate;
			clamp *= 0.00001;
			//set up the thing to choke off oscillations- belt and suspenders affair

			if (A != 0)
			{
				subSample += (lp)*densityC;
				subSample -= (subSample * subSample * subSample * subscale);

				if (subSample > 0) subSample -= clamp;
				if (subSample < 0) subSample += clamp;

				//subSample *= subSample * A * 6;
			}

			//////////////////////////////////////////////////
			//////////////////////////////////////////////////
			//////////////////////////////////////////////////

			//drive section
			bassSampleL *= (densityC + 1.0);
			bridgerectifier = fabs(bassSampleL) * 1.57079633;

			// Modified this to set bridgerectifier if undefined
			if (bridgerectifier > 1.57079633) bridgerectifier = 1.57079633;
			//max value for sine function
			if (densityC > 0) bridgerectifier = sin(bridgerectifier);
			else bridgerectifier = 1 - cos(bridgerectifier);
			//produce either boosted or starved version
			if (bassSampleL > 0) bassSampleL = (bassSampleL * (1 - outC)) + (bridgerectifier * outC);
			else				 bassSampleL = (bassSampleL * (1 - outC)) - (bridgerectifier * outC);
			//blend according to densityC control

			bassSampleR *= (densityC + 1.0);
			bridgerectifier = fabs(bassSampleR) * 1.57079633;
			if (bridgerectifier > 1.57079633) bridgerectifier = 1.57079633;
			//max value for sine function
			if (densityC > 0) bridgerectifier = sin(bridgerectifier);
			else bridgerectifier = 1 - cos(bridgerectifier);
			//produce either boosted or starved version
			if (bassSampleR > 0) bassSampleR = (bassSampleR * (1 - outC)) + (bridgerectifier * outC);
			else				 bassSampleR = (bassSampleR * (1 - outC)) - (bridgerectifier * outC);
			//blend according to densityC control

			// Summing outputs
			inputSampleL = midSampleL;
			inputSampleL += bassSampleL;

			inputSampleR = midSampleR;
			inputSampleR += bassSampleR;

			if (subSample != 0.0)
			{
				inputSampleL += subSample;
			}
			if (subSample != 0.0)
			{
				inputSampleR += subSample;
			}
		}
		//end EQ

		//EQ lowpass is after all processing like the compressor that might produce hash
		if (engageLowpass)
		{
			if (flip)
			{
				lowpassSampleLAA = (lowpassSampleLAA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLAA;
				lowpassSampleLBA = (lowpassSampleLBA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLBA;
				lowpassSampleLCA = (lowpassSampleLCA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLCA;
				lowpassSampleLDA = (lowpassSampleLDA * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLDA;
				lowpassSampleLE = (lowpassSampleLE * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLE;

				lowpassSampleRAA = (lowpassSampleRAA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRAA;
				lowpassSampleRBA = (lowpassSampleRBA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRBA;
				lowpassSampleRCA = (lowpassSampleRCA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRCA;
				lowpassSampleRDA = (lowpassSampleRDA * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRDA;
				lowpassSampleRE = (lowpassSampleRE * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRE;
			}
			else
			{
				lowpassSampleLAB = (lowpassSampleLAB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLAB;
				lowpassSampleLBB = (lowpassSampleLBB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLBB;
				lowpassSampleLCB = (lowpassSampleLCB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLCB;
				lowpassSampleLDB = (lowpassSampleLDB * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLDB;
				lowpassSampleLF = (lowpassSampleLF * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
				inputSampleL = lowpassSampleLF;

				lowpassSampleRAB = (lowpassSampleRAB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRAB;
				lowpassSampleRBB = (lowpassSampleRBB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRBB;
				lowpassSampleRCB = (lowpassSampleRCB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRCB;
				lowpassSampleRDB = (lowpassSampleRDB * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRDB;
				lowpassSampleRF = (lowpassSampleRF * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
				inputSampleR = lowpassSampleRF;
			}
			lowpassSampleLG = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			lowpassSampleRG = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);

			inputSampleL = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			inputSampleR = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
		}

		//built in output trim and dry/wet if desired
		if (outputgain != 1.0) {
			inputSampleL *= outputgain;
			inputSampleR *= outputgain;
		}

		//begin 64 bit stereo floating point dither
		//int expon; frexp((double)inputSampleL, &expon);
		fpdL ^= fpdL << 13; fpdL ^= fpdL >> 17; fpdL ^= fpdL << 5;
		//inputSampleL += ((double(fpdL)-uint32_t(0x7fffffff)) * 1.1e-44l * pow(2,expon+62));
		//frexp((double)inputSampleR, &expon);
		fpdR ^= fpdR << 13; fpdR ^= fpdR >> 17; fpdR ^= fpdR << 5;
		//inputSampleR += ((double(fpdR)-uint32_t(0x7fffffff)) * 1.1e-44l * pow(2,expon+62));
		//end 64 bit stereo floating point dither

		*out1 = inputSampleL;
		*out2 = inputSampleR;

		*in1++;
		*in2++;
		*out1++;
		*out2++;
	}
}
*/