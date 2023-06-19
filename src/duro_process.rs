use std::f32::consts::PI;
use nih_plug::{util::{self}, prelude::Enum};

#[derive(Enum, PartialEq, Eq, Debug, Copy, Clone)]
pub enum ConsoleMode {
    #[name = "Bypass"]
    BYPASS,
    #[name = "Neve Inspired"]
    NEVE,
    #[name = "API Inspired"]
    API,
    #[name = "Precision Inspired"]
    PRECISION,
    #[name = "Leaf Console"]
    LEAF,
    #[name = "Vine Console"]
    VINE,
    #[name = "Duro Console"]
    DURO,
}

#[derive(Enum, PartialEq, Eq, Debug, Copy, Clone)]
pub enum SaturationModeEnum {
    #[name = "No Saturation"]
    NONESAT,
    #[name = "Tape Saturation"]
    TAPESAT,
    #[name = "Candle"]
    CANDLE,
    #[name = "Chebyshev"]
    CHEBYSHEV,
    #[name = "\"Leaf\""]
    LEAF,
    #[name = "Digital Clip"]
    DIGITAL,
    #[name = "Golden Cubic"]
    GOLDENCUBIC,
    #[name = "Transformer"]
    TRANSFORMER,
    #[name = "Odd Harmonics"]
    ODDHARMONICS,
    #[name = "Fourth Harmonics"]
    FORTHHARM,
}

/**************************************************
 * EQ Filter Algorithm
 **************************************************/


// Storing how I created the array of coefficients - not used in program
#[allow(dead_code)]
pub fn gen_coefficients(boost_db: f32, sr: f32, bands: Vec<f32>) -> Vec<(f32, f32, f32, f32, f32)> {
    // How many bands to split the signal into
    //let bands: Vec<f32> = vec![280.0,800.0];
    let mut eq_coefficients: Vec<(f32, f32, f32, f32, f32)> = Vec::new();

    for band in bands {
        let center_freq = band;
        let omega = 2.0 * std::f32::consts::PI * center_freq / sr;
        let alpha = 0.0;
        let cos_omega = omega.cos();
        let a0 = 1.0 + alpha / boost_db;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / boost_db;
        let b0 = (1.0 + cos_omega) / 2.0 / boost_db;
        let b1 = -(1.0 + cos_omega) / boost_db;
        let b2 = (1.0 + cos_omega) / 2.0 / boost_db;
        eq_coefficients.push((b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0));
    }
    eq_coefficients
    //println!("NEW COEFFICIENTS");
    //println!("{:?}", eq_coefficients);
}

/**************************************************
 * Saturation Algorithms
 **************************************************/

// tape saturation using transfer function
fn tape_saturation(input_signal: f32, drive: f32, threshold: f32) -> f32 {
    let idrive = if drive == 0.0 {0.0001} else {drive};
    // Define the transfer curve for the tape saturation effect
    let transfer = |x: f32| -> f32 {
        (x * idrive).tanh() / (threshold * idrive).tanh()
    };
    // Apply the transfer curve to the input sample
    let output_sample = transfer(input_signal);
    // soft clip the output
    let mut normalized_output_sample = output_sample / (1.0 + output_sample.abs());
    // Lower this signal because it is LOUDER than the original
    normalized_output_sample *= util::db_to_gain(-12.0);
    normalized_output_sample
}

fn digital_saturation(sample: f32, threshold: f32, drive: f32) -> f32 
{
    let clipped = if sample.abs() > threshold {
        sample.signum() * threshold // Clip the signal if it exceeds the threshold
    } else {
        sample
    };
    let output = sample * (1.0 - drive) + clipped * drive; // Mix original signal with clipped signal
    output
}

// Chebyshev polynomial saturation (Thanks to AI help)
fn chebyshev_tape(sample: f32, threshold: f32, drive: f32) -> f32 {
    // saturation limit value
    let k = if sample.abs() > threshold {
        threshold / sample.abs()
    } else {
        1.0
    };
    // normalized input
    let x = sample * k / (1.0 + drive);
    // Calculate the Chebyshev values
    let x2 = x * x;
    let x3 = x * x2;
    let x5 = x3 * x2;
    let x6 = x3 * x3;
    let y = x
        - 0.166667 * x3
        + 0.00833333 * x5
        - 0.000198413 * x6;
    y / (1.0 + y.abs()) // Soft clip output
}

// Golden ratio based saturation with cubic curve
fn golden_cubic(sample: f32, threshold: f32, drive: f32) -> f32 
{
    let golden_ratio = 1.61803398875;
    let abs_input = sample.abs();
    // If we are above the threshold, multiply by the golden and cube the excess sample
    let output = if abs_input > threshold {
        let sign = sample.signum();
        let excess = abs_input - threshold;
        let shaped_excess = threshold * golden_ratio * excess.powi(3); // apply cubic function multiplied by golden ratio
        sign * (threshold + shaped_excess)
    } else {
        sample
    };
    // Apply soft clip to the output
    
    let sc_threshold = 1.0 - drive - 0.0001;
    let sign = output.signum();
    let clipped = (1.0 - (-output.abs()).exp()) / (1.0 - (-sc_threshold).exp());
    let mut temp = output + sign * clipped * sc_threshold;

    // Lower a pinch because it is louder than the original
    temp *= util::db_to_gain(-3.0);
    temp
}

// Add 5 odd harmonics to the signal at a strength
fn odd_saturation_with_threshold(signal: f32, harmonic_strength: f32, threshold: f32) -> f32 {
    let num_harmonics: usize = 5;
    let mut summed = signal;

    for j in 1..=num_harmonics {
        let harmonic = (2 * j - 1) as f32;
        let harmonic_component = harmonic_strength * (signal * harmonic).sin();

        if harmonic_component.abs() > threshold {
            // Calculate the reduction factor based on the threshold
            let reduction_factor = threshold / harmonic_component.abs();
            let reduced_harmonic_component = harmonic_component * reduction_factor;
            summed += reduced_harmonic_component;
        }
    }
    summed
}

// Add X harmonics to signal
fn add_x_harmonics(signal: f32, harmonic_strength: f32, threshold: f32, harmonic_num: i32) -> f32 {
    let num_harmonics: usize = 10;
    let mut summed = signal;

    for j in 1..=num_harmonics {
        let harmonic = harmonic_num as f32 * j as f32;
        let harmonic_component = harmonic_strength * (signal * harmonic).sin();

        if harmonic_component.abs() > threshold {
            // Calculate the reduction factor based on the threshold
            let reduction_factor = threshold / harmonic_component.abs();
            let reduced_harmonic_component = harmonic_component * reduction_factor;
            summed += reduced_harmonic_component;
        }
    }

    summed
}

// Add soft compressed candle saturation idea to signal
fn candle_saturation(signal: f32, drive: f32, threshold: f32) -> f32 {
    let saturation_amount = (signal - threshold).max(0.0) * drive;
    let compressed_saturation = saturation_amount / (1.0 + saturation_amount.abs());
    let saturated_signal = signal + compressed_saturation;
    saturated_signal
}

// "Leaf" Saturation designed by Ardura
fn leaf_saturation(input_signal: f32, threshold: f32, drive: f32) -> f32 {
    let range = 6.0;
    let min_value = 1.0;
    let drive_db = min_value + drive * range;
    let signal_holder = input_signal * util::db_to_gain(drive_db);
    
    let curve = (signal_holder / 999.0).powf(2.0);

    let mut y = signal_holder / threshold;
    y = (2.0 / PI) * y.atan();
    (threshold + (1.0 - threshold) * curve) * y
}

fn transformer_saturation(sample: f32, threshold: f32, drive: f32) -> f32 {
    let shape = 2.0 - (drive * 2.0).min(2.0).max(0.0);

    let input_level = sample.abs();
    let output_gain = if input_level < threshold {
        1.0
    } else {
        let gain_reduction = (input_level - threshold) / (1.0 - threshold);
        let input_gain = 1.0 + (drive - 1.0) * gain_reduction.powf(5.0);
        let shaped_gain = (input_gain.tanh() / shape).max(0.0).min(drive);

        // Adjust the gain based on the input level
        let reduced_gain = shaped_gain * (1.0 - input_level);

        reduced_gain
    };

    // Apply the gain to the input signal and saturate it
    let output = sample * output_gain;
    let output = if output.abs() > 1.0 {
        output.signum()
    } else {
        output
    };

    output
}



/****************************************************************************************
 *  Feedback Delay Network Processor
 ****************************************************************************************/
    /*
    difference equation
    y[n] = x[n] - 0.00010904511 * y[n-50] - 0.04816142 * y[n-1] - 0.00320949 * y[n-2] - 0.0028770843 * y[n-3] - 0.0025967543 * y[n-4] - 0.0023367526 * y[n-5] - 0.0021145213 * y[n-6] - 0.0019009487 * y[n-7] - 0.0017166135 * y[n-8] - 0.0015247492 * y[n-9] - 0.00133025 * y[n-10] - 0.0011769092 * y[n-11] - 0.0010452771 * y[n-12] - 0.000897834 * y[n-13] - 0.0007409797 * y[n-14] - 0.00058073737 * y[n-15] - 0.00044383493 * y[n-16] - 0.0003281392 * y[n-17] - 0.0002153296 * y[n-18] - 6.399655e-5 * y[n-19] + 4.9314993e-5 * y[n-20] + 0.00013502018 * y[n-21] + 0.00020215382 * y[n-22] + 0.00023804209 * y[n-23] + 0.00026853455 * y[n-24] + 0.0003083128 * y[n-25] + 0.0003445775 * y[n-26] + 0.00036402745 * y[n-27] + 0.00039966477 * y[n-28] + 0.0004403214 * y[n-29] + 0.0004541246 * y[n-30] + 0.00046441422 * y[n-31] + 0.00048072706 * y[n-32] + 0.0004873777 * y[n-33] + 0.00046981 * y[n-34] + 0.00047457838 * y[n-35] + 0.0004539991 * y[n-36] + 0.00042601628 * y[n-37] + 0.00039803347 * y[n-38] + 0.00038385383 * y[n-39] + 0.0003287666 * y[n-40] + 0.000287859 * y[n-41] + 0.00022950921 * y[n-42] + 0.00019349548 * y[n-43] + 0.00017015554 * y[n-44] + 0.00015823462 * y[n-45] + 0.00013853372 * y[n-46] + 0.00012184442 * y[n-47] + 9.2983224e-5 * y[n-48] + 9.461451e-5 * y[n-49] + y[n-50]
    
    output = input
        - fdn[0] * outputs[i-50] 
        - fdn[1] * outputs[i-1] 
        - fdn[2] * outputs[i-2] 
        - fdn[3] * outputs[i-3] 
        - fdn[4] * outputs[i-4] 
        - fdn[5] * outputs[i-5]
        ... 
        + outputs[n-50]
     */


/**************************************************
 * Duro Console
 **************************************************/

pub struct Console {
    threshold: f32,
    drive: f32,
    console_type: crate::duro_process::ConsoleMode,
    sample_rate: f32,
    duro_array: [f32; 12],
    leaf_array: [f32; 20],
    vine_array: [f32; 21],
    neve_array: [f32; 34],
    prec_array: [f32; 34],
    api_array: [f32; 34],
}

#[allow(unused_variables)]
impl Console {
    pub fn new(
        threshold: f32,
        ratio: i32,
        console_type: crate::duro_process::ConsoleMode,
        sample_rate: f32,
    ) -> Self {
        Self {
            threshold,
            drive: 0.0,
            console_type: crate::duro_process::ConsoleMode::BYPASS,
            sample_rate,
            duro_array: [0.0; 12],
            leaf_array: [0.0; 20],
            vine_array: [0.0; 21],
            neve_array: [0.0; 34],
            prec_array: [0.0; 34],
            api_array: [0.0; 34],
        }
    }

    pub fn update_vals(&mut self, threshold: f32, drive: f32, console_type: crate::duro_process::ConsoleMode, sample_rate: f32) {
        self.threshold = threshold;
        self.drive = drive;
        self.sample_rate = sample_rate;
        self.console_type = console_type;
    }

    pub fn duro_process(&mut self, sample: f32, sat_type: crate::duro_process::SaturationModeEnum, console_type: crate::duro_process::ConsoleMode) -> f32 
    {
        // Initialize our return value
        let mut consoled_sample = 0.0;
        self.drive = if self.drive == 0.0 {0.000001} else {self.drive};
        
        // Initialize Feedback Delay Network Processors
        if console_type == crate::duro_process::ConsoleMode::BYPASS
        {
            // Do nothing
            consoled_sample = sample;
        }
        // Airwindows inspired console jank creating some console model
        else if console_type == crate::duro_process::ConsoleMode::DURO
        {
            let mut temp_sample = sample;

            self.duro_array[11] = self.duro_array[10];
            self.duro_array[10] = self.duro_array[9];
            self.duro_array[9] = self.duro_array[8];
            self.duro_array[8] = self.duro_array[7];
            self.duro_array[7] = self.duro_array[6];
            self.duro_array[6] = self.duro_array[5];
            self.duro_array[5] = self.duro_array[4];
            self.duro_array[4] = self.duro_array[3];
            self.duro_array[3] = self.duro_array[2];
            self.duro_array[2] = self.duro_array[1];
            self.duro_array[1] = self.duro_array[0];
            self.duro_array[0] = temp_sample * self.drive;
            
            // Transfer function Direct form? - left rotated ascii num rage,43121,12257 then +- rand
            temp_sample += self.duro_array[1] * (0.12463 + 0.0009082*(self.duro_array[1].abs()));
            temp_sample -= self.duro_array[2] * (0.24631 + 0.0007892*(self.duro_array[2].abs()));
            temp_sample += self.duro_array[3] * (0.46312 - 0.0004984*(self.duro_array[3].abs()));
            temp_sample += self.duro_array[4] * (0.63124 + 0.0008833*(self.duro_array[4].abs()));
            temp_sample -= self.duro_array[5] * (0.31246 + 0.0007061*(self.duro_array[5].abs()));
            temp_sample += self.duro_array[6] *  (0.43121 - 0.0003605*(self.duro_array[6].abs()));
            temp_sample -= self.duro_array[7] *  (0.31214 + 0.0008056*(self.duro_array[7].abs()));
            temp_sample += self.duro_array[8] *  (0.12143 + 0.0006117*(self.duro_array[8].abs()));
            temp_sample -= self.duro_array[9] *  (0.21431 + 0.0005775*(self.duro_array[9].abs()));
            temp_sample += self.duro_array[10] * (0.14312 + 0.0002237*(self.duro_array[10].abs()));
            temp_sample -= self.duro_array[11] * (0.12257 + 0.0005422*(self.duro_array[11].abs()));

            consoled_sample = temp_sample;
        }
        // Airwindows inspired console jank creating random console
        else if console_type == crate::duro_process::ConsoleMode::LEAF
        {
            let mut temp_sample = sample;

            self.leaf_array[19] = self.leaf_array[18];
            self.leaf_array[18] = self.leaf_array[17];
            self.leaf_array[17] = self.leaf_array[16];
            self.leaf_array[16] = self.leaf_array[15];
            self.leaf_array[15] = self.leaf_array[14];
            self.leaf_array[14] = self.leaf_array[13];
            self.leaf_array[13] = self.leaf_array[12];
            self.leaf_array[12] = self.leaf_array[11];
            self.leaf_array[11] = self.leaf_array[10];
            self.leaf_array[10] = self.leaf_array[9];
            self.leaf_array[9] = self.leaf_array[8];
            self.leaf_array[8] = self.leaf_array[7];
            self.leaf_array[7] = self.leaf_array[6];
            self.leaf_array[6] = self.leaf_array[5];
            self.leaf_array[5] = self.leaf_array[4];
            self.leaf_array[4] = self.leaf_array[3];
            self.leaf_array[3] = self.leaf_array[2];
            self.leaf_array[2] = self.leaf_array[1];
            self.leaf_array[1] = self.leaf_array[0];
            self.leaf_array[0] = temp_sample * self.drive;
            
            // Transfer function Direct form?
            temp_sample += self.leaf_array[1] * (0.20641 - 0.0007895*(self.leaf_array[1].abs()));
            temp_sample -= self.leaf_array[2] * (0.34072 + 0.0004034*(self.leaf_array[2].abs()));
            temp_sample += self.leaf_array[3] * (0.43302 - 0.0003548*(self.leaf_array[3].abs()));
            temp_sample += self.leaf_array[4] * (0.14097 - 0.0003409*(self.leaf_array[4].abs()));
            temp_sample -= self.leaf_array[5] * (0.00658 + 0.0003328*(self.leaf_array[5].abs()));
            temp_sample += self.leaf_array[6] * (0.58875 - 0.0003155*(self.leaf_array[6].abs()));
            temp_sample -= self.leaf_array[7] * (0.28183 + 0.0003045*(self.leaf_array[7].abs()));
            temp_sample += self.leaf_array[8] * (0.00555 - 0.0003044*(self.leaf_array[8].abs()));
            temp_sample -= self.leaf_array[9] * (0.024370 + 0.0003031*(self.leaf_array[9].abs()));
            temp_sample += self.leaf_array[10] * (0.00401 - 0.0003029*(self.leaf_array[10].abs()));
            temp_sample -= self.leaf_array[11] * (0.01781 + 0.0004679*(self.leaf_array[11].abs()));
            temp_sample += self.leaf_array[12] * (0.03884 - 0.0003539*(self.leaf_array[12].abs()));
            temp_sample += self.leaf_array[13] * (0.00221 + 0.0008839*(self.leaf_array[13].abs()));
            temp_sample += self.leaf_array[14] * (0.00451 - 0.0009749*(self.leaf_array[14].abs()));
            temp_sample += self.leaf_array[15] * (0.03501 - 0.0003122*(self.leaf_array[15].abs()));
            temp_sample += self.leaf_array[16] * (0.06568 - 0.0002257*(self.leaf_array[16].abs()));
            temp_sample += self.leaf_array[17] * (0.05355 - 0.0009112*(self.leaf_array[17].abs()));
            temp_sample += self.leaf_array[18] * (0.00522 - 0.0001911*(self.leaf_array[18].abs()));
            temp_sample += self.leaf_array[19] * (0.03569 - 0.0001945*(self.leaf_array[19].abs()));

            consoled_sample = temp_sample;
        }
        else if console_type == crate::duro_process::ConsoleMode::NEVE {
            let mut temp_sample = sample;

            self.neve_array[33] = self.neve_array[32];
            self.neve_array[32] = self.neve_array[31]; 
			self.neve_array[31] = self.neve_array[30];
            self.neve_array[30] = self.neve_array[29];
            self.neve_array[29] = self.neve_array[28];
            self.neve_array[28] = self.neve_array[27]; 
            self.neve_array[27] = self.neve_array[26]; 
            self.neve_array[26] = self.neve_array[25]; 
            self.neve_array[25] = self.neve_array[24]; 
            self.neve_array[24] = self.neve_array[23]; 
			self.neve_array[23] = self.neve_array[22];
            self.neve_array[22] = self.neve_array[21]; 
            self.neve_array[21] = self.neve_array[20]; 
            self.neve_array[20] = self.neve_array[19]; 
            self.neve_array[19] = self.neve_array[18]; 
            self.neve_array[18] = self.neve_array[17]; 
            self.neve_array[17] = self.neve_array[16]; 
            self.neve_array[16] = self.neve_array[15]; 
			self.neve_array[15] = self.neve_array[14];
            self.neve_array[14] = self.neve_array[13]; 
            self.neve_array[13] = self.neve_array[12]; 
            self.neve_array[12] = self.neve_array[11]; 
            self.neve_array[11] = self.neve_array[10];
            self.neve_array[10] = self.neve_array[9]; 
            self.neve_array[9] = self.neve_array[8]; 
            self.neve_array[8] = self.neve_array[7]; 
			self.neve_array[7] = self.neve_array[6]; 
            self.neve_array[6] = self.neve_array[5]; 
            self.neve_array[5] = self.neve_array[4]; 
            self.neve_array[4] = self.neve_array[3]; 
            self.neve_array[3] = self.neve_array[2]; 
            self.neve_array[2] = self.neve_array[1]; 
            self.neve_array[1] = self.neve_array[0]; 
            self.neve_array[0] = temp_sample * self.drive;
			
			temp_sample += self.neve_array[1] * (0.20641602693167951  - (0.00078952185394898*(self.neve_array[1].abs())));
			temp_sample -= self.neve_array[2] * (0.07601816702459827  + (0.00022786334179951*(self.neve_array[2].abs())));
			temp_sample += self.neve_array[3] * (0.03929765560019285  - (0.00054517993246352*(self.neve_array[3].abs())));
			temp_sample += self.neve_array[4] * (0.00298333157711103  - (0.00033083756545638*(self.neve_array[4].abs())));
			temp_sample -= self.neve_array[5] * (0.00724006282304610  + (0.00045483683460812*(self.neve_array[5].abs())));
			temp_sample += self.neve_array[6] * (0.03073108963506036  - (0.00038190060537423*(self.neve_array[6].abs())));
			temp_sample -= self.neve_array[7] * (0.02332434692533051  + (0.00040347288688932*(self.neve_array[7].abs())));
			temp_sample += self.neve_array[8] * (0.03792606869061214  - (0.00039673687335892*(self.neve_array[8].abs())));
			temp_sample -= self.neve_array[9] * (0.02437059376675688  + (0.00037221210539535*(self.neve_array[9].abs())));
			temp_sample += self.neve_array[10] * (0.03416764311979521  - (0.00040314850796953*(self.neve_array[10].abs())));
			temp_sample -= self.neve_array[11] * (0.01761669868102127  + (0.00035989484330131*(self.neve_array[11].abs())));
			temp_sample += self.neve_array[12] * (0.02538237753523052  - (0.00040149119125394*(self.neve_array[12].abs())));
			temp_sample -= self.neve_array[13] * (0.00770737340728377  + (0.00035462118723555*(self.neve_array[13].abs())));
			temp_sample += self.neve_array[14] * (0.01580706228482803  - (0.00037563141307594*(self.neve_array[14].abs())));
			temp_sample += self.neve_array[15] * (0.00055119240005586  - (0.00035409299268971*(self.neve_array[15].abs())));
			temp_sample += self.neve_array[16] * (0.00818552143438768  - (0.00036507661042180*(self.neve_array[16].abs())));
			temp_sample += self.neve_array[17] * (0.00661842703548304  - (0.00034550528559056*(self.neve_array[17].abs())));
			temp_sample += self.neve_array[18] * (0.00362447476272098  - (0.00035553012761240*(self.neve_array[18].abs())));
			temp_sample += self.neve_array[19] * (0.00957098027225745  - (0.00034091691045338*(self.neve_array[19].abs())));
			temp_sample += self.neve_array[20] * (0.00193621774016660  - (0.00034554529131668*(self.neve_array[20].abs())));
			temp_sample += self.neve_array[21] * (0.01005433027357935  - (0.00033878223153845*(self.neve_array[21].abs())));
			temp_sample += self.neve_array[22] * (0.00221712428802004  - (0.00033481410137711*(self.neve_array[22].abs())));
			temp_sample += self.neve_array[23] * (0.00911255639207995  - (0.00033263425232666*(self.neve_array[23].abs())));
			temp_sample += self.neve_array[24] * (0.00339667169034909  - (0.00032634428038430*(self.neve_array[24].abs())));
			temp_sample += self.neve_array[25] * (0.00774096948249924  - (0.00032599868802996*(self.neve_array[25].abs())));
			temp_sample += self.neve_array[26] * (0.00463907626773794  - (0.00032131993173361*(self.neve_array[26].abs())));
			temp_sample += self.neve_array[27] * (0.00658222997260378  - (0.00032014977430211*(self.neve_array[27].abs())));
			temp_sample += self.neve_array[28] * (0.00550347079924993  - (0.00031557153256653*(self.neve_array[28].abs())));
			temp_sample += self.neve_array[29] * (0.00588754981375325  - (0.00032041307242303*(self.neve_array[29].abs())));
			temp_sample += self.neve_array[30] * (0.00590293898419892  - (0.00030457857428714*(self.neve_array[30].abs())));
			temp_sample += self.neve_array[31] * (0.00558952010441800  - (0.00030448053548086*(self.neve_array[31].abs())));
			temp_sample += self.neve_array[32] * (0.00598183557634295  - (0.00030715064323181*(self.neve_array[32].abs())));
			temp_sample += self.neve_array[33] * (0.00555223929714115  - (0.00030319367948553*(self.neve_array[33].abs())));

            consoled_sample = temp_sample;
        }
        else if console_type == crate::duro_process::ConsoleMode::API {
            let mut temp_sample = sample;

            self.api_array[33] = self.api_array[32];
            self.api_array[32] = self.api_array[31]; 
			self.api_array[31] = self.api_array[30];
            self.api_array[30] = self.api_array[29];
            self.api_array[29] = self.api_array[28];
            self.api_array[28] = self.api_array[27]; 
            self.api_array[27] = self.api_array[26]; 
            self.api_array[26] = self.api_array[25]; 
            self.api_array[25] = self.api_array[24]; 
            self.api_array[24] = self.api_array[23]; 
			self.api_array[23] = self.api_array[22];
            self.api_array[22] = self.api_array[21]; 
            self.api_array[21] = self.api_array[20]; 
            self.api_array[20] = self.api_array[19]; 
            self.api_array[19] = self.api_array[18]; 
            self.api_array[18] = self.api_array[17]; 
            self.api_array[17] = self.api_array[16]; 
            self.api_array[16] = self.api_array[15]; 
			self.api_array[15] = self.api_array[14];
            self.api_array[14] = self.api_array[13]; 
            self.api_array[13] = self.api_array[12]; 
            self.api_array[12] = self.api_array[11]; 
            self.api_array[11] = self.api_array[10];
            self.api_array[10] = self.api_array[9]; 
            self.api_array[9] = self.api_array[8]; 
            self.api_array[8] = self.api_array[7]; 
			self.api_array[7] = self.api_array[6]; 
            self.api_array[6] = self.api_array[5]; 
            self.api_array[5] = self.api_array[4]; 
            self.api_array[4] = self.api_array[3]; 
            self.api_array[3] = self.api_array[2]; 
            self.api_array[2] = self.api_array[1]; 
            self.api_array[1] = self.api_array[0]; 
            self.api_array[0] = temp_sample * self.drive;
			
			temp_sample += self.api_array[1] * (0.09299870608542582  - (0.00009582362368873*(self.api_array[1].abs())));
			temp_sample -= self.api_array[2] * (0.11947847710741009  - (0.00004500891602770*(self.api_array[2].abs())));
			temp_sample += self.api_array[3] * (0.09071606264761795  + (0.00005639498984741*(self.api_array[3].abs())));
			temp_sample -= self.api_array[4] * (0.08561982770836980  - (0.00004964855606916*(self.api_array[4].abs())));
			temp_sample += self.api_array[5] * (0.06440549220820363  + (0.00002428052139507*(self.api_array[5].abs())));
			temp_sample -= self.api_array[6] * (0.05987991812840746  + (0.00000101867082290*(self.api_array[6].abs())));
			temp_sample += self.api_array[7] * (0.03980233135839382  + (0.00003312430049041*(self.api_array[7].abs())));
			temp_sample -= self.api_array[8] * (0.03648402630896925  - (0.00002116186381142*(self.api_array[8].abs())));
			temp_sample += self.api_array[9] * (0.01826860869525248  + (0.00003115110025396*(self.api_array[9].abs())));
			temp_sample -= self.api_array[10] * (0.01723968622495364  - (0.00002450634121718*(self.api_array[10].abs())));
			temp_sample += self.api_array[11] * (0.00187588812316724  + (0.00002838206198968*(self.api_array[11].abs())));
			temp_sample -= self.api_array[12] * (0.00381796423957237  - (0.00003155815499462*(self.api_array[12].abs())));
			temp_sample -= self.api_array[13] * (0.00852092214496733  - (0.00001702651162392*(self.api_array[13].abs())));
			temp_sample += self.api_array[14] * (0.00315560292270588  + (0.00002547861676047*(self.api_array[14].abs())));
			temp_sample -= self.api_array[15] * (0.01258630914496868  - (0.00004555319243213*(self.api_array[15].abs())));
			temp_sample += self.api_array[16] * (0.00536435648963575  + (0.00001812393657101*(self.api_array[16].abs())));
			temp_sample -= self.api_array[17] * (0.01272975658159178  - (0.00004103775306121*(self.api_array[17].abs())));
			temp_sample += self.api_array[18] * (0.00403818975172755  + (0.00003764615492871*(self.api_array[18].abs())));
			temp_sample -= self.api_array[19] * (0.01042617366897483  - (0.00003605210426041*(self.api_array[19].abs())));
			temp_sample += self.api_array[20] * (0.00126599583390057  + (0.00004305458668852*(self.api_array[20].abs())));
			temp_sample -= self.api_array[21] * (0.00747876207688339  - (0.00003731207018977*(self.api_array[21].abs())));
			temp_sample -= self.api_array[22] * (0.00149873689175324  - (0.00005086601800791*(self.api_array[22].abs())));
			temp_sample -= self.api_array[23] * (0.00503221309488033  - (0.00003636086782783*(self.api_array[23].abs())));
			temp_sample -= self.api_array[24] * (0.00342998224655821  - (0.00004103091180506*(self.api_array[24].abs())));
			temp_sample -= self.api_array[25] * (0.00355585977903117  - (0.00003698982145400*(self.api_array[25].abs())));
			temp_sample -= self.api_array[26] * (0.00437201792934817  - (0.00002720235666939*(self.api_array[26].abs())));
			temp_sample -= self.api_array[27] * (0.00299217874451556  - (0.00004446954727956*(self.api_array[27].abs())));
			temp_sample -= self.api_array[28] * (0.00457924652487249  - (0.00003859065778860*(self.api_array[28].abs())));
			temp_sample -= self.api_array[29] * (0.00298182934892027  - (0.00002064710931733*(self.api_array[29].abs())));
			temp_sample -= self.api_array[30] * (0.00438838441540584  - (0.00005223008424866*(self.api_array[30].abs())));
			temp_sample -= self.api_array[31] * (0.00323984218794705  - (0.00003397987535887*(self.api_array[31].abs())));
			temp_sample -= self.api_array[32] * (0.00407693981307314  - (0.00003935772436894*(self.api_array[32].abs())));
			temp_sample -= self.api_array[33] * (0.00350435348467321  - (0.00005525463935338*(self.api_array[33].abs())));

            consoled_sample = temp_sample;
        }
        else if console_type == crate::duro_process::ConsoleMode::PRECISION {
            let mut temp_sample = sample;

            self.prec_array[33] = self.prec_array[32];
            self.prec_array[32] = self.prec_array[31]; 
			self.prec_array[31] = self.prec_array[30];
            self.prec_array[30] = self.prec_array[29];
            self.prec_array[29] = self.prec_array[28];
            self.prec_array[28] = self.prec_array[27]; 
            self.prec_array[27] = self.prec_array[26]; 
            self.prec_array[26] = self.prec_array[25]; 
            self.prec_array[25] = self.prec_array[24]; 
            self.prec_array[24] = self.prec_array[23]; 
			self.prec_array[23] = self.prec_array[22];
            self.prec_array[22] = self.prec_array[21]; 
            self.prec_array[21] = self.prec_array[20]; 
            self.prec_array[20] = self.prec_array[19]; 
            self.prec_array[19] = self.prec_array[18]; 
            self.prec_array[18] = self.prec_array[17]; 
            self.prec_array[17] = self.prec_array[16]; 
            self.prec_array[16] = self.prec_array[15]; 
			self.prec_array[15] = self.prec_array[14];
            self.prec_array[14] = self.prec_array[13]; 
            self.prec_array[13] = self.prec_array[12]; 
            self.prec_array[12] = self.prec_array[11]; 
            self.prec_array[11] = self.prec_array[10];
            self.prec_array[10] = self.prec_array[9]; 
            self.prec_array[9] = self.prec_array[8]; 
            self.prec_array[8] = self.prec_array[7]; 
			self.prec_array[7] = self.prec_array[6]; 
            self.prec_array[6] = self.prec_array[5]; 
            self.prec_array[5] = self.prec_array[4]; 
            self.prec_array[4] = self.prec_array[3]; 
            self.prec_array[3] = self.prec_array[2]; 
            self.prec_array[2] = self.prec_array[1]; 
            self.prec_array[1] = self.prec_array[0]; 
            self.prec_array[0] = temp_sample * self.drive;
			
			temp_sample += self.prec_array[1] * (0.59188440274551890  - (0.00008361469668405*(self.prec_array[1]).abs()));
			temp_sample -= self.prec_array[2] * (0.24439750948076133  + (0.00002651678396848*(self.prec_array[2]).abs()));
			temp_sample += self.prec_array[3] * (0.14109876103205621  - (0.00000840487181372*(self.prec_array[3]).abs()));
			temp_sample -= self.prec_array[4] * (0.10053507128157971  + (0.00001768100964598*(self.prec_array[4]).abs()));
			temp_sample += self.prec_array[5] * (0.05859287880626238  - (0.00000361398065989*(self.prec_array[5]).abs()));
			temp_sample -= self.prec_array[6] * (0.04337406889823660  + (0.00000735941182117*(self.prec_array[6]).abs()));
			temp_sample += self.prec_array[7] * (0.01589900680531097  + (0.00000207347387987*(self.prec_array[7]).abs()));
			temp_sample -= self.prec_array[8] * (0.01087234854973281  + (0.00000732123412029*(self.prec_array[8]).abs()));
			temp_sample -= self.prec_array[9] * (0.00845782429679176  - (0.00000133058605071*(self.prec_array[9]).abs()));
			temp_sample += self.prec_array[10] * (0.00662278586618295  - (0.00000424594730611*(self.prec_array[10]).abs()));
			temp_sample -= self.prec_array[11] * (0.02000592193760155  + (0.00000632896879068*(self.prec_array[11]).abs()));
			temp_sample += self.prec_array[12] * (0.01321157777167565  - (0.00001421171592570*(self.prec_array[12]).abs()));
			temp_sample -= self.prec_array[13] * (0.02249955362988238  + (0.00000163937127317*(self.prec_array[13]).abs()));
			temp_sample += self.prec_array[14] * (0.01196492077581504  - (0.00000535385220676*(self.prec_array[14]).abs()));
			temp_sample -= self.prec_array[15] * (0.01905917427000097  + (0.00000121672882030*(self.prec_array[15]).abs()));
			temp_sample += self.prec_array[16] * (0.00761909482108073  - (0.00000326242895115*(self.prec_array[16]).abs()));
			temp_sample -= self.prec_array[17] * (0.01362744780256239  + (0.00000359274216003*(self.prec_array[17]).abs()));
			temp_sample += self.prec_array[18] * (0.00200183122683721  - (0.00000089207452791*(self.prec_array[18]).abs()));
			temp_sample -= self.prec_array[19] * (0.00833042637239315  + (0.00000946767677294*(self.prec_array[19]).abs()));
			temp_sample -= self.prec_array[20] * (0.00258481175207224  - (0.00000087429351464*(self.prec_array[20]).abs()));
			temp_sample -= self.prec_array[21] * (0.00459744479712244  - (0.00000049519758701*(self.prec_array[21]).abs()));
			temp_sample -= self.prec_array[22] * (0.00534277030993820  + (0.00000397547847155*(self.prec_array[22]).abs()));
			temp_sample -= self.prec_array[23] * (0.00272332919605675  + (0.00000040077229097*(self.prec_array[23]).abs()));
			temp_sample -= self.prec_array[24] * (0.00637243782359372  - (0.00000139419072176*(self.prec_array[24]).abs()));
			temp_sample -= self.prec_array[25] * (0.00233001590327504  + (0.00000420129915747*(self.prec_array[25]).abs()));
			temp_sample -= self.prec_array[26] * (0.00623296727793041  + (0.00000019010664856*(self.prec_array[26]).abs()));
			temp_sample -= self.prec_array[27] * (0.00276177096376805  + (0.00000580301901385*(self.prec_array[27]).abs()));
			temp_sample -= self.prec_array[28] * (0.00559184754866264  + (0.00000080597287792*(self.prec_array[28]).abs()));
			temp_sample -= self.prec_array[29] * (0.00343180144395919  - (0.00000243701142085*(self.prec_array[29]).abs()));
			temp_sample -= self.prec_array[30] * (0.00493325428861701  + (0.00000300985740900*(self.prec_array[30]).abs()));
			temp_sample -= self.prec_array[31] * (0.00396140827680823  - (0.00000051459681789*(self.prec_array[31]).abs()));
			temp_sample -= self.prec_array[32] * (0.00448497879902493  + (0.00000744412841743*(self.prec_array[32]).abs()));
			temp_sample -= self.prec_array[33] * (0.00425146888772076  - (0.00000082346016542*(self.prec_array[33]).abs()));

            consoled_sample = temp_sample;
        }
        // Vine console - Ardura created Sound
        else if console_type == crate::duro_process::ConsoleMode::VINE
        {
            let mut temp_sample = sample;

            self.vine_array[20] = self.vine_array[19]; 
            self.vine_array[19] = self.vine_array[18]; 
            self.vine_array[18] = self.vine_array[17]; 
            self.vine_array[17] = self.vine_array[16]; 
            self.vine_array[16] = self.vine_array[15]; 
			self.vine_array[15] = self.vine_array[14];
            self.vine_array[14] = self.vine_array[13]; 
            self.vine_array[13] = self.vine_array[12]; 
            self.vine_array[12] = self.vine_array[11]; 
            self.vine_array[11] = self.vine_array[10];
            self.vine_array[10] = self.vine_array[9]; 
            self.vine_array[9] = self.vine_array[8]; 
            self.vine_array[8] = self.vine_array[7]; 
			self.vine_array[7] = self.vine_array[6]; 
            self.vine_array[6] = self.vine_array[5]; 
            self.vine_array[5] = self.vine_array[4]; 
            self.vine_array[4] = self.vine_array[3]; 
            self.vine_array[3] = self.vine_array[2]; 
            self.vine_array[2] = self.vine_array[1]; 
            self.vine_array[1] = self.vine_array[0]; 
            self.vine_array[0] = temp_sample * self.drive;

            temp_sample += self.vine_array[1] *  (0.0436325893992795  - (0.000575411073043639*(self.vine_array[1].abs())));
			temp_sample -= self.vine_array[2] *  (0.0398664344439780  + (0.000401805174100580*(self.vine_array[2].abs())));
			temp_sample += self.vine_array[3] *  (0.0423995851137356  - (0.000397649382328122*(self.vine_array[3].abs())));
			temp_sample += self.vine_array[4] *  (0.0510140498581183  - (0.000353184194832542*(self.vine_array[4].abs())));
			temp_sample -= self.vine_array[5] *  (0.0769342576758843  + (0.000291725137291541*(self.vine_array[5].abs())));
			temp_sample += self.vine_array[6] *  (0.0401007008556487  - (0.000215534790074162*(self.vine_array[6].abs())));
			temp_sample -= self.vine_array[7] *  (0.0373217915258955  + (0.000270605983316673*(self.vine_array[7].abs())));
			temp_sample += self.vine_array[8] *  (0.0287532033895229  - (0.000346928125413722*(self.vine_array[8].abs())));
			temp_sample -= self.vine_array[9] *  (0.0342323841982068  + (0.000481560773128127*(self.vine_array[9].abs())));
			temp_sample += self.vine_array[10] * (0.0324797624174322  - (0.000259173731735134*(self.vine_array[10].abs())));
			temp_sample -= self.vine_array[11] * (0.0444824631252694  + (0.000476415778491772*(self.vine_array[11].abs())));
			temp_sample += self.vine_array[12] * (0.0238301394147183  - (0.000361592576652208*(self.vine_array[12].abs())));
			temp_sample += self.vine_array[13] * (0.0232701919791822  + (0.000352466711624552*(self.vine_array[13].abs())));
			temp_sample += self.vine_array[14] * (0.0375055553818633  - (0.000326762074484403*(self.vine_array[14].abs())));
			temp_sample += self.vine_array[15] * (0.0237069785097524  - (0.000251524844696344*(self.vine_array[15].abs())));
			temp_sample += self.vine_array[16] * (0.0448529936846194  - (0.000257515409563439*(self.vine_array[16].abs())));
			temp_sample += self.vine_array[17] * (0.0355616566388069  - (0.000249453764586831*(self.vine_array[17].abs())));
			temp_sample += self.vine_array[18] * (0.0248772252552266  - (0.000323542566487897*(self.vine_array[18].abs())));
			temp_sample += self.vine_array[19] * (0.0212456446513081  - (0.000232976462487141*(self.vine_array[19].abs())));
			temp_sample += self.vine_array[20] * (0.0172523541136474  - (0.000399899365529506*(self.vine_array[20].abs())));

            // Recover some volume lost
            consoled_sample = temp_sample;
        }

        #[allow(unreachable_patterns)]
        match sat_type {
            // No saturation
            SaturationModeEnum::NONESAT => return consoled_sample,
            // adding even and odd harmonics
            SaturationModeEnum::TAPESAT => return tape_saturation(consoled_sample, self.drive, self.threshold),
            // Add 5 odd harmonics at drive strength
            SaturationModeEnum::ODDHARMONICS => return odd_saturation_with_threshold(consoled_sample, self.drive, self.threshold),
            // Add 5 even harmonics at drive strength
            SaturationModeEnum::FORTHHARM => return add_x_harmonics(consoled_sample, self.drive, self.threshold, 4),
            // Candle Saturation through soft compressor added to signal
            SaturationModeEnum::CANDLE => return candle_saturation(consoled_sample, self.drive, self.threshold),
            // Hardclipped mix with original
            SaturationModeEnum::DIGITAL => return digital_saturation(consoled_sample, self.threshold, self.drive),
            // Chebyshev polynomial saturation based off the symmetrical saturation research - pretending to be tape
            SaturationModeEnum::CHEBYSHEV => return chebyshev_tape(consoled_sample, self.threshold, self.drive),
            // Golden Cubic designed by Ardura
            SaturationModeEnum::GOLDENCUBIC => return golden_cubic(consoled_sample, self.threshold, self.drive),
            // "Leaf" saturation designed by Ardura - wildly inefficient
            SaturationModeEnum::LEAF => return leaf_saturation(consoled_sample, self.threshold, self.drive),
            // Transformer Model
            SaturationModeEnum::TRANSFORMER => return transformer_saturation(consoled_sample, self.threshold, self.drive),
            // Default to no saturation
            _ => return consoled_sample,
        }

    }
}
