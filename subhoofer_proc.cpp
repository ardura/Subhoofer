/* ========================================
 *  EQ - EQ.h
 *  Copyright (c) 2016 airwindows, Airwindows uses the MIT license
 * ======================================== */

#ifndef __subhoofer_H
#include "subhoofer.h"
#endif
#include <windows.h>
#include <debugapi.h>

void subhoofer::processReplacing(float** inputs, float** outputs, VstInt32 sampleFrames)
{
	float* in1 = inputs[0];
	float* in2 = inputs[1];
	float* out1 = outputs[0];
	float* out2 = outputs[1];

	double overallscale = 1.0;
	overallscale /= 44100.0;
	overallscale *= getSampleRate();

	double clamp = 0.0;

	// Samples to hold input and EQ changes
	double midSampleL = 0.0;
	double bassSampleL = 0.0;

	double midSampleR = 0.0;
	double bassSampleR = 0.0;

	// Sub Voicing knob
	double HeadBump = 0.0;
	double HeadBumpFreq = (((A) * 0.1) + 0.02) / overallscale;

	// Sub Amt knob
	//double BassGain = C * 0.1;
	double BassGain = 0.7;

	//double HeadBumpFreq = (((A * A * 380.0) + 20.0)) / overallscale;
	double iirAmount = HeadBumpFreq  / 44.1;

	// Sub Gain knob
	double SubOutGain = B*6;
	double SubBump = 0.0;

	// Mid Push knob
	double lowGain = (C * 24.0) - 12.0;
	//double lowGain = pow(10.0, ((C * 12) - 6) / 20.0);
	// Mid Freq Knob
	double iirAmountB = (((F * F * 770.0) + 30.0) * 10) / overallscale;
	iirMidBumpLA = 0.0;
	iirMidBumpLB = 0.0;
	iirMidBumpLC = 0.0;
	iirMidBumpRA = 0.0;
	iirMidBumpRB = 0.0;
	iirMidBumpRC = 0.0;
	MidBumpL = 0.0;
	MidBumpR = 0.0;
	MidSampleA = 0.0;
	MidSampleB = 0.0;
	MidSampleC = 0.0;
	MidSampleD = 0.0;
	iirAmountB = iirAmountB / 44.1;


	bool engageEQ = true;
	if (0.0 == lowGain) engageEQ = false;

		
	lowGain = pow(10.0, lowGain / 20.0) - 1.0;
	//convert to 0 to X multiplier with 1.0 being O db
	//minus one gives nearly -1 to ? (should top out at 1)
	//calibrate so that X db roughly equals X db with maximum topping out at 1 internally

	// Lowpass knob
	double lp;
	double iirAmountC = (4 * D * D);
	if (iirAmountC > 1.0) iirAmountC = 1.0;
	bool engageLowpass = false;
	if ((D * D * 4) < 4) engageLowpass = true;
	
	double bridgerectifier;

	double outC = fabs(lowGain);

	double outputgain = pow(10.0, ((H * 36.0) - 18.0) / 20.0);

	while (--sampleFrames >= 0)
	{
		double inputSampleL = *in1;
		double inputSampleR = *in2;

		if (fabs(inputSampleL) < 1.18e-23) inputSampleL = fpdL * 1.18e-17;
		if (fabs(inputSampleR) < 1.18e-23) inputSampleR = fpdR * 1.18e-17;



		randD = (double(fpdL) / UINT32_MAX);
		invrandD = (1 - randD);
		randD /= 2.0;



		//////////////////////////////////////////////////
		// Sub calculation code
		//////////////////////////////////////////////////

		// Calculate the lowpass to be used
		lp = (inputSampleL + inputSampleR) / 2.0;
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
		//set up the thing to choke off oscillations- belt and suspenders affair


		if (lp > 0)
		{
			//OutputDebugStringW(L"lp is positive.\n");

			if (WasNegative)
			{
				SubOctave = !SubOctave;
			}
			WasNegative = false;
		}
		else
		{
			//OutputDebugStringW(L"lp is negative OH MIRA MIRA MIRA MIRA MIRA MIRA MIRA MIRA MIRA MIRA.\n");
			WasNegative = true;
		}


		// This calculates a low pass, then different samples from it and substracts from the low pass even more
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

		iirSampleJA = (iirSampleJA * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJA;
		iirSampleJB = (iirSampleJB * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJB;
		iirSampleJC = (iirSampleJC * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJC;
		iirSampleJD = (iirSampleJD * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJD;
		iirSampleJE = (iirSampleJE * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJE;
		iirSampleJF = (iirSampleJF * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJF;
		iirSampleJG = (iirSampleJG * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJG;
		iirSampleJH = (iirSampleJH * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJH;
		iirSampleJI = (iirSampleJI * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJI;
		iirSampleJJ = (iirSampleJJ * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJJ;
		iirSampleJK = (iirSampleJK * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJK;
		iirSampleJL = (iirSampleJL * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJL;
		iirSampleJM = (iirSampleJM * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJM;
		iirSampleJN = (iirSampleJN * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJN;
		iirSampleJO = (iirSampleJO * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJO;
		iirSampleJP = (iirSampleJP * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJP;
		iirSampleJQ = (iirSampleJQ * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJQ;
		iirSampleJR = (iirSampleJR * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJR;
		iirSampleJS = (iirSampleJS * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJS;
		iirSampleJT = (iirSampleJT * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJT;
		iirSampleJU = (iirSampleJU * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJU;
		iirSampleJV = (iirSampleJV * (1.0 - iirAmount)) + (lp * iirAmount);			lp -= iirSampleJV;


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

		//HeadBump = (lp * HeadBumpFreq);

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
		if (SubOctave == false) { SubBump = -SubBump; }

		// Note the rand is what is flipping from positive to negative here
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

		//inputSampleL += (HeadBump * BassOutGain);
		//inputSampleR += (HeadBump * BassOutGain);


		//////////////////////////////////////////////////
		//////////////////////////////////////////////////
		//////////////////////////////////////////////////




	//////////////////////////////////////////////////////
	//	MID CODE
	//////////////////////////////////////////////////////

		if (engageEQ)
		{
			switch (bflip)
			{
			case 1:
				iirMidBumpLA += (inputSampleL);
				iirMidBumpLA -= (iirMidBumpLA * iirMidBumpLA * iirMidBumpLA * iirAmountB);
				iirMidBumpLA = (invrandD * iirMidBumpLA) + (randD * iirMidBumpLB) + (randD * iirMidBumpLC);
				if (iirMidBumpLA > 0) iirMidBumpLA -= clamp;
				if (iirMidBumpLA < 0) iirMidBumpLA += clamp;
				MidBumpL = iirMidBumpLA;

				iirMidBumpRA += (inputSampleR);
				iirMidBumpRA -= (iirMidBumpRA * iirMidBumpRA * iirMidBumpRA * iirAmountB);
				iirMidBumpRA = (invrandD * iirMidBumpRA) + (randD * iirMidBumpRB) + (randD * iirMidBumpRC);
				if (iirMidBumpRA > 0) iirMidBumpRA -= clamp;
				if (iirMidBumpRA < 0) iirMidBumpRA += clamp;
				MidBumpR = iirMidBumpRA;
				break;
			case 2:
				iirMidBumpLB += (inputSampleL);
				iirMidBumpLB -= (iirMidBumpLB * iirMidBumpLB * iirMidBumpLB * iirAmountB);
				iirMidBumpLB = (randD * iirMidBumpLA) + (invrandD * iirMidBumpLB) + (randD * iirMidBumpLC);
				if (iirMidBumpLB > 0) iirMidBumpLB -= clamp;
				if (iirMidBumpLB < 0) iirMidBumpLB += clamp;
				MidBumpL = iirMidBumpLB;

				iirMidBumpRB += (inputSampleR);
				iirMidBumpRB -= (iirMidBumpRB * iirMidBumpRB * iirMidBumpRB * iirAmountB);
				iirMidBumpRB = (randD * iirMidBumpRA) + (invrandD * iirMidBumpRB) + (randD * iirMidBumpRC);
				if (iirMidBumpRB > 0) iirMidBumpRB -= clamp;
				if (iirMidBumpRB < 0) iirMidBumpRB += clamp;
				MidBumpR = iirMidBumpLB;
				break;
			case 3:
				iirMidBumpLC += (inputSampleL);
				iirMidBumpLC -= (iirMidBumpLC * iirMidBumpLC * iirMidBumpLC * iirAmountB);
				iirMidBumpLC = (randD * iirMidBumpLA) + (randD * iirMidBumpLB) + (invrandD * iirMidBumpLC);
				if (iirMidBumpLC > 0) iirMidBumpLC -= clamp;
				if (iirMidBumpLC < 0) iirMidBumpLC += clamp;
				MidBumpL = iirMidBumpLC;

				iirMidBumpRC += (inputSampleR);
				iirMidBumpRC -= (iirMidBumpRC * iirMidBumpRC * iirMidBumpRC * iirAmountB);
				iirMidBumpRC = (randD * iirMidBumpRA) + (randD * iirMidBumpRB) + (invrandD * iirMidBumpRC);
				if (iirMidBumpRC > 0) iirMidBumpRC -= clamp;
				if (iirMidBumpRC < 0) iirMidBumpRC += clamp;
				MidBumpR = iirMidBumpRC;
				break;
			}

			//HeadBump = (lp * HeadBumpFreq);

			// Calculate drive samples based off what we've done so far		
			MidSampleA = (MidSampleA * (1.0 - iirAmountB)) + (MidBumpL * iirAmountB); 			MidBumpL -= MidSampleA;
			MidSampleB = (MidSampleB * (1.0 - iirAmountB)) + (MidBumpL * iirAmountB); 			MidBumpL -= MidSampleB;

			MidSampleC = (MidSampleC * (1.0 - iirAmountB)) + (MidBumpR * iirAmountB); 			MidBumpR -= MidSampleC;
			MidSampleD = (MidSampleD * (1.0 - iirAmountB)) + (MidBumpR * iirAmountB); 			MidBumpR -= MidSampleD;
			/// <summary>
			/// ///////////////////////////////////////////////////////////////////////////



			inputSampleL += (MidBumpL * lowGain);
			inputSampleR += (MidBumpR * lowGain);

			//////////////////////////////////////////////
			// END MID
			////////////////////////////////////////////
		}




		//	Lowpass is after all processing like the compressor that might produce hash
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

		flip = !flip;
		bflip++;
		if (bflip < 1 || bflip > 3) bflip = 1;

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


//void subhoofer::processDoubleReplacing(double** inputs, double** outputs, VstInt32 sampleFrames)
