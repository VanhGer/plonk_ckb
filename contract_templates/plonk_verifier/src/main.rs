//! Generated by capsule
//!
//! `main.rs` is used to define rust lang items and modules.
//! See `entry.rs` for the `main` function.
//! See `error.rs` for the `Error` type.

#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![no_std]
#![cfg_attr(not(test), no_main)]

mod error;
mod challenge;
mod verify;
mod data_structures;

#[cfg(test)]
extern crate alloc;

use ark_serialize::CanonicalDeserialize;
use ark_std::iterable::Iterable;
use ckb_std::ckb_constants::Source;
use ckb_std::debug;
#[cfg(not(test))]
use ckb_std::default_alloc;
use ckb_std::high_level::load_cell_data;
use sha2::Sha256;
use crate::data_structures::{CommonPreprocessedInput, Proof, Srs};

use crate::error::Error;

const CPI: &[u8] = include_bytes!("cpi.bin");
const SRS: &[u8] = include_bytes!("srs.bin");

#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!(4*1024, 2048*1024,64);

/// program entry
fn program_entry() -> i8 {
    debug!("load proof_data!");
    let proof_data = match load_cell_data(0, Source::Output) {
        Ok(data) => data,
        Err(_) => return Error::LoadingCell as i8,
    };
    debug!("proof_data is {:?}", proof_data.len());

    debug!("deserialize proof!");
    let proof = match Proof::deserialize_uncompressed_unchecked(&proof_data[..]) {
        Ok(data) => data,
        Err(_) => return Error::Encoding as i8,
    };
    //

    debug!("deserialize cpi");

    let cpi = match CommonPreprocessedInput::deserialize_uncompressed_unchecked(CPI) {
        Ok(data) => data,
        Err(e) => {
            debug!("{:?}",e);
            return Error::Encoding as i8;
        }
    };

    // return 0;
    debug!("deserialize srs");
    debug!("SRS.len() = {:#?}", SRS.len());
    let srs = match Srs::deserialize_uncompressed_unchecked(SRS) {
        Ok(data) => data,
        Err(e) => {
            debug!("{:#?}", e);
            return Error::Encoding as i8;
        }
    };

    debug!("verify");
    // Call main function and return error code
    match verify::plonk_verify::<Sha256>(proof, cpi, srs) {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
    // 0
}