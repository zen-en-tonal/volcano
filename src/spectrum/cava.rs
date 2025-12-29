#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{error::Error, fmt::Display};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// A wrapper around the Cava audio visualizer.
pub struct Cava {
    plan: *mut cava_plan,
}

impl Cava {
    /// Initializes a new Cava instance.
    pub fn new(
        number_of_bars: i32,
        rate: u32,
        channels: i32,
        autosens: i32,
        noise_reduction: f64,
        low_cut_off: i32,
        high_cut_off: i32,
    ) -> Result<Self, CavaError> {
        let plan = unsafe {
            cava_init(
                number_of_bars,
                rate,
                channels,
                autosens,
                noise_reduction,
                low_cut_off,
                high_cut_off,
            )
        };

        if plan.is_null() {
            Err(CavaError::InitializationFailed)
        } else {
            Ok(Cava { plan })
        }
    }

    /// Execute the Cava processing on the input audio data.
    pub fn execute(&self, cava_in: &mut [f32], cava_out: &mut [f64]) {
        let samples = cava_in.len();
        let mut cast = vec![0.0; samples];
        for i in 0..samples {
            cast[i] = cava_in[i] as f64;
        }

        unsafe {
            cava_execute(
                cast.as_mut_ptr(),
                samples as i32,
                cava_out.as_mut_ptr(),
                self.plan,
            );
        }
    }
}

impl Drop for Cava {
    fn drop(&mut self) {
        unsafe {
            cava_destroy(self.plan);
        }
    }
}

/// An error that can occur when initializing or using Cava.
#[derive(Debug)]
pub enum CavaError {
    InitializationFailed,
}

impl Error for CavaError {}

impl Display for CavaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CavaError::InitializationFailed => write!(f, "Cava initialization failed"),
        }
    }
}
