#ifndef __subhoofer_H
#include "subhoofer.h"
#endif
#include <windows.h>
#include <debugapi.h>
#include <cmath>

#define _USE_MATH_DEFINES
#include <math.h>

void subhoofer::processReplacing(float** inputs, float** outputs, VstInt32 sampleFrames)
{
	float* in1 = inputs[0];
	float* in2 = inputs[1];
	float* out1 = outputs[0];
	float* out2 = outputs[1];

	// Used to scale head bump frequency for Subhoof
	double overallscale = 1.0;
	overallscale /= 44100.0;
	overallscale *= getSampleRate();

	// Hold our sample rate for use in other spots
	double sampleRateVal = getSampleRate();

	double clamp = 0.0;

	// Sub Voicing(Hoof) knob
	double HeadBump = 0.0;
	double HeadBumpFreq = (((A) * 0.1) + 0.02) / overallscale;
	double iirAmount = HeadBumpFreq / 44.1;

	// Sub hardcoded gain/drive for Hoof knob
	double BassGain = 0.7;

	// Sub Gain knob
	double SubOutGain = B*24.0;
	double SubBump = 0.0;

	// Tilt EQ
	double lowGain = (C * 12.0) - 6.0;
	previousC = 0.0f;

	// Split Freq Knob
	double SplitFrequency = ((F * F * 770.0) + 30.0);
	lastF = 0.0;

	// If the EQ is Engaged (Tilt EQ knob)
	bool engageEQ = true;
	if (0.0 == lowGain) engageEQ = false;

	// Lowpass knob
	double lp;
	// 4*Frequency^2 Curve for lowpass freq slider
	double iirAmountC = (4 * D * D);
	// Normalize iirAmountC (shouldn't happen)
	if (iirAmountC > 1.0) iirAmountC = 1.0;

	// Should lowpass code run?
	bool engageLowpass = false;
	// If the knob is not all the way up
	if ((D * D * 4) < 4) engageLowpass = true;

	// Output Gain knob - the pow(10.0,X/20.0) converts to amplitude from dB
	double outputgain = pow(10.0, ((H * 36.0) - 18.0) / 20.0);

	// Process audio sample by sample as a stream
	while (--sampleFrames >= 0)
	{
		double inputSampleL = *in1;
		double inputSampleR = *in2;

		// Normalize high/low gains
		if (fabs(inputSampleL) < 1.18e-23) inputSampleL = fpdL * 1.18e-17;
		if (fabs(inputSampleR) < 1.18e-23) inputSampleR = fpdR * 1.18e-17;

		// Used in the distortion of the gain
		randD = (double(fpdL) / UINT32_MAX);
		invrandD = (1 - randD);
		randD /= 2.0;

		// If Sub Gain or Subhoof
		if ((A > 0) && (B > 0))
		{
			//////////////////////////////////////////////////
			// Sub calculation code
			//////////////////////////////////////////////////

			// Calculate the lowpass to be used - This division SHARPLY cuts out freqs
			lp = ((inputSampleL + inputSampleR) / 2048.0);
			iirDriveSampleA = (iirDriveSampleA * (1.0 - HeadBumpFreq)) + (lp * HeadBumpFreq); lp = iirDriveSampleA;
			iirDriveSampleB = (iirDriveSampleB * (1.0 - HeadBumpFreq)) + (lp * HeadBumpFreq); lp = iirDriveSampleB;

			// Gate from airwindows
			oscGate += fabs(lp * 10.0);
			oscGate -= 0.001;
			if (oscGate > 1.0) oscGate = 1.0;
			if (oscGate < 0) oscGate = 0;
			//got a value that only goes down low when there's silence or near silence on input
			clamp = 1.0 - oscGate;
			clamp *= 0.00001;

			// Which side of the zero crossing are we on?
			if (lp > 0)
			{
				// We are on top!
				if (WasNegative)
				{
					// Flip the SubOctave boolean
					SubOctave = !SubOctave;
				}
				WasNegative = false;
			}
			else
			{
				// We are on bottom
				WasNegative = true;
			}

			// This calculates a low pass, then different samples from it and subtracts from the low pass even more
			iirSampleA = (iirSampleA * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleA;
			iirSampleB = (iirSampleB * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleB;
			iirSampleC = (iirSampleC * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleC;
			iirSampleD = (iirSampleD * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleD;
			iirSampleE = (iirSampleE * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleE;
			iirSampleF = (iirSampleF * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleF;
			iirSampleG = (iirSampleG * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleG;
			iirSampleH = (iirSampleH * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleH;
			iirSampleI = (iirSampleI * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleI;
			iirSampleJ = (iirSampleJ * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJ;
			iirSampleK = (iirSampleK * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleK;
			iirSampleL = (iirSampleL * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleL;
			iirSampleM = (iirSampleM * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleM;
			iirSampleN = (iirSampleN * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleN;
			iirSampleO = (iirSampleO * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleO;
			iirSampleP = (iirSampleP * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleP;
			iirSampleQ = (iirSampleQ * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleQ;
			iirSampleR = (iirSampleR * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleR;
			iirSampleS = (iirSampleS * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleS;
			iirSampleT = (iirSampleT * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleT;
			iirSampleU = (iirSampleU * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleU;
			iirSampleV = (iirSampleV * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleV;

			switch (bflip)
			{
				case 1:
					iirHeadBumpA += (lp * BassGain);
					iirHeadBumpA -= (iirHeadBumpA * iirHeadBumpA * iirHeadBumpA * HeadBumpFreq);
					iirHeadBumpA = (invrandD * iirHeadBumpA) + (randD * iirHeadBumpB) + (randD * iirHeadBumpC);
					if (iirHeadBumpA > 0) iirHeadBumpA -= clamp;
					if (iirHeadBumpA < 0) iirHeadBumpA += clamp;
					HeadBump = iirHeadBumpA;
					break;
				case 2:
					iirHeadBumpB += (lp * BassGain);
					iirHeadBumpB -= (iirHeadBumpB * iirHeadBumpB * iirHeadBumpB * HeadBumpFreq);
					iirHeadBumpB = (randD * iirHeadBumpA) + (invrandD * iirHeadBumpB) + (randD * iirHeadBumpC);
					if (iirHeadBumpB > 0) iirHeadBumpB -= clamp;
					if (iirHeadBumpB < 0) iirHeadBumpB += clamp;
					HeadBump = iirHeadBumpB;
					break;
				case 3:
					iirHeadBumpC += (lp * BassGain);
					iirHeadBumpC -= (iirHeadBumpC * iirHeadBumpC * iirHeadBumpC * HeadBumpFreq);
					iirHeadBumpC = (randD * iirHeadBumpA) + (randD * iirHeadBumpB) + (invrandD * iirHeadBumpC);
					if (iirHeadBumpC > 0) iirHeadBumpC -= clamp;
					if (iirHeadBumpC < 0) iirHeadBumpC += clamp;
					HeadBump = iirHeadBumpC;
					break;
			}

			// Calculate drive samples based off what we've done so far		
			iirSampleW = (iirSampleW * (1.0 - iirAmount)) + (HeadBump * iirAmount); 			HeadBump -= iirSampleW;
			iirSampleX = (iirSampleX * (1.0 - iirAmount)) + (HeadBump * iirAmount); 			HeadBump -= iirSampleX;

			// Create SubBump sample from our head bump to modify further
			SubBump = HeadBump;
			iirSampleY = (iirSampleY * (1.0 - iirAmount)) + (SubBump * iirAmount);			SubBump -= iirSampleY;

			// Calculate sub drive samples based off what we've done so far		
			iirDriveSampleC = (iirDriveSampleC * (1.0 - HeadBumpFreq)) + (SubBump * HeadBumpFreq);			SubBump = iirDriveSampleC;
			iirDriveSampleD = (iirDriveSampleD * (1.0 - HeadBumpFreq)) + (SubBump * HeadBumpFreq);			SubBump = iirDriveSampleD;

			SubBump = fabs(SubBump);
			// Flip the bump sample per SubOctave true/false for the half-frequency
			if (SubOctave == false) { SubBump = -SubBump; }

			// Note the randD/invrandD is what is flipping from positive to negative here
			// This means bflip = 1 A gets inverted
			// This means bflip = 2 B gets inverted
			// This means bflip = 3 C gets inverted
			// This creates a lower octave using math jank on the multiplication depending on sample
			switch (bflip)
			{
			case 1:
				iirSubBumpA += SubBump;// * BassGain);
				iirSubBumpA -= (iirSubBumpA * iirSubBumpA * iirSubBumpA * HeadBumpFreq);
				iirSubBumpA = (invrandD * iirSubBumpA) + (randD * iirSubBumpB) + (randD * iirSubBumpC);
				if (iirSubBumpA > 0) iirSubBumpA -= clamp;
				if (iirSubBumpA < 0) iirSubBumpA += clamp;
				SubBump = iirSubBumpA;
				break;
			case 2:
				iirSubBumpB += SubBump;// * BassGain);
				iirSubBumpB -= (iirSubBumpB * iirSubBumpB * iirSubBumpB * HeadBumpFreq);
				iirSubBumpB = (randD * iirSubBumpA) + (invrandD * iirSubBumpB) + (randD * iirSubBumpC);
				if (iirSubBumpB > 0) iirSubBumpB -= clamp;
				if (iirSubBumpB < 0) iirSubBumpB += clamp;
				SubBump = iirSubBumpB;
				break;
			case 3:
				iirSubBumpC += SubBump;// * BassGain);
				iirSubBumpC -= (iirSubBumpC * iirSubBumpC * iirSubBumpC * HeadBumpFreq);
				iirSubBumpC = (randD * iirSubBumpA) + (randD * iirSubBumpB) + (invrandD * iirSubBumpC);
				if (iirSubBumpC > 0) iirSubBumpC -= clamp;
				if (iirSubBumpC < 0) iirSubBumpC += clamp;
				SubBump = iirSubBumpC;
				break;
			}

			// Resample to reduce the sub bump further
			iirSampleZ = (iirSampleZ * (1.0 - HeadBumpFreq)) + (SubBump * HeadBumpFreq); 			SubBump = iirSampleZ;
			iirDriveSampleE = (iirDriveSampleE * (1.0 - iirAmount)) + (SubBump * iirAmount); 			SubBump = iirDriveSampleE;
			iirDriveSampleF = (iirDriveSampleF * (1.0 - iirAmount)) + (SubBump * iirAmount); 			SubBump = iirDriveSampleF;

			// Sub gain only
			inputSampleL += (SubBump * SubOutGain);
			inputSampleR += (SubBump * SubOutGain);
		}

		bflip++;
		if (bflip < 1 || bflip > 3) bflip = 1;

		//////////////////////////////////////////////
		//	TILT EQ CODE
		//////////////////////////////////////////////
		// If gain applied
		if (engageEQ)
		{
			// Normalize high/low values
			if (fabs(inputSampleL) < 1.18e-23) inputSampleL = fpdL * 1.18e-17;
			if (fabs(inputSampleR) < 1.18e-23) inputSampleR = fpdR * 1.18e-17;

			// rounding of inputs for filters to 3 places
			lastF = std::round(lastF * 1000.0f) / 1000.0f;
			F = std::round(F * 1000.0f) / 1000.0f;
			previousC = std::round(previousC * 1000.0f) / 1000.0f;
			C = std::round(C * 1000.0f) / 1000.0f;

			// If our split frequency or gain has changed
			if ((lastF != F) || (previousC != C))
			{
				// Setup tilt filter values
				float filterAmp = 6 / log(2);
				float sr3 = 3 * sampleRateVal;
				float gfactor = 5;

				// Build filter off lowpass signal gains
				if (lowGain > 0) {
					gain1 = lowGain * -gfactor;
					gain2 = lowGain;
				}
				else {
					// I had to swap gfactor from it's original gain2 implementation to weight based off bass side
					gain1 = -lowGain * gfactor;
					gain2 = lowGain;
				};
				lowGainT = exp(gain1 / filterAmp) - 1;
				highGainT = exp(gain2 / filterAmp) - 1;

				// tilt filter
				float omega = 2 * M_PI * SplitFrequency;
				float n = 1 / (sr3 + omega);
				a0 = 2 * omega * n;
				b1 = (sr3 - omega) * n;

				lastF = F;
				previousC = C;
			}

			lp_outL = (a0 * inputSampleL + b1 * lp_outL);
			inputSampleL = inputSampleL + lowGainT * lp_outL + highGainT * (inputSampleL - lp_outL) + denorm;

			lp_outR = (a0 * inputSampleR + b1 * lp_outR);
			inputSampleR = inputSampleR + lowGainT * lp_outR + highGainT * (inputSampleR - lp_outR) + +denorm;
		}

		//////////////////////////////////////////////
		// END TILT EQ
		//////////////////////////////////////////////

		//	Lowpass is after all processing like the compressor that might produce hash
		if (engageLowpass)
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

			lowpassSampleLG = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			lowpassSampleRG = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);

			inputSampleL = (lowpassSampleLG * (1.0 - iirAmountC)) + (inputSampleL * iirAmountC);
			inputSampleR = (lowpassSampleRG * (1.0 - iirAmountC)) + (inputSampleR * iirAmountC);
		}

		// Remove DC Offset with single pole HP
		hp_a0 = 1;
		hp_b0 = 1.0;
		hp_b1 = -1.0;
		hp_a1 = -0.995;

		// Calculated below by Ardura in advance!
		// double sqrt2 = 1.41421356237;
		// double corner_frequency = 5.0 / sqrt2;
		// double hp_gain = 1 / sqrt(1 + (5.0 / (corner_frequency)) ^ 2);
		double hp_gain = 0.577350269190468;

		// Apply the 1 pole HP
		inputSampleL = hp_gain * inputSampleL;
		tempL = hp_b0 * inputSampleL + hp_b1 * inputLPrev - hp_a1 * outputLPrev;
		inputLPrev = inputSampleL;
		outputLPrev = tempL;
		inputSampleL = tempL;

		// Apply the 1 pole HP
		inputSampleR = hp_gain * inputSampleR;
		tempL = hp_b0 * inputSampleR + hp_b1 * inputRPrev - hp_a1 * outputRPrev;
		inputRPrev = inputSampleR;
		outputRPrev = tempL;
		inputSampleR = tempL;

		//built in output gain
		if (outputgain != 1.0) {
			inputSampleL *= outputgain;
			inputSampleR *= outputgain;
		}

		*out1 = inputSampleL;
		*out2 = inputSampleR;

		*in1++;
		*in2++;
		*out1++;
		*out2++;
	}
}


//void subhoofer::processDoubleReplacing(double** inputs, double** outputs, VstInt32 sampleFrames)
