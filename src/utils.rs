use crate::err::{self, ErrorCaller};
use std::*;

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub fn get(url: &str) -> Result<Vec<u8>> {
    let resp = reqwest::blocking::get(url).e()?;
    let resp_code = resp.status();
    if resp_code != 200 {
        return Err(err::new(format!("{}|{}", resp.text().e()?, resp_code)))?;
    }
    let resp_byte = resp.bytes().e()?.to_vec();
    Ok(resp_byte)
}
