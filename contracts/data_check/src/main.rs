#![feature(cstr_bytes)]
#![no_std]
#![cfg_attr(not(test), no_main)]

mod error;

#[cfg(test)]
extern crate alloc;

use ckb_std::ckb_constants::Source;
use ckb_std::debug;
#[cfg(not(test))]
use ckb_std::default_alloc;
use ckb_std::high_level::load_cell_data;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();
use crate::error::Error;
pub fn program_entry() -> i8 {
    match check_data() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

fn check_data() -> Result<(), Error> {

    let check_first_data = "apple".as_bytes();
    let check_second_data = "tomato".as_bytes();


    let first_data = match load_cell_data(0, Source::Output) {
        Ok(data) => data,
        Err(err) => return Err(err.into()),
    };


    debug!("first_data is {:?}", first_data);
    debug!("check_first_data is {:?}", check_first_data);

    let second_data = match load_cell_data(1, Source::Output) {
        Ok(data) => data,
        Err(err) => return Err(err.into()),
    };
    debug!("second_data is {:?}", second_data);
    debug!("check_second_data is {:?}", check_second_data);

    if first_data == check_first_data && second_data == check_second_data {
        return Ok(());
    }
    return Err(Error::Verify);
}
