use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum Errors {
    ChecksumError,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::ChecksumError => write!(f, "Checksum Failed!"),
        }
    }
}

impl Error for Errors {}

pub fn checksum(arr: [u8; 8]) -> Result<[u8; 8], Errors> {
    let checksum: u16 = arr[0..3].iter().map(|i| *i as u16).sum::<u16>() & 0xff;
    if (arr[3] as u16 != checksum) | is_encrypted(arr) {
        Err(Errors::ChecksumError)
    } else {
        Ok(arr)
    }
}

pub fn is_encrypted(arr: [u8; 8]) -> bool {
    // see: https://github.com/heinemml/CO2Meter/issues/4
    // some (newer?) devices don't encrypt the data, if byte 4 != 0x0d
    // assume encrypted data
    // ??? might result in wrong data sometimes?!!!
    arr[4] != 0x0d
}

pub fn decrypt<'a>(key: &[u8; 8], data: [u8; 8]) -> [u8; 8] {
    let cstate: [u8; 8] = [0x48, 0x74, 0x65, 0x6D, 0x70, 0x39, 0x39, 0x65];
    let shuffle: [usize; 8] = [2, 4, 0, 7, 1, 6, 5, 3];

    let mut phase1: [u8; 8] = Default::default();
    shuffle
        .iter()
        .enumerate()
        .for_each(|(n, &v)| phase1[v] = data[n]);

    let mut phase2: [u8; 8] = Default::default();
    (0..8).for_each(|i| phase2[i] = phase1[i] ^ key[i]);

    let mut phase3: [u8; 8] = Default::default();
    (0..8).for_each(|i| phase3[i] = ((phase2[i] >> 3) | (phase2[(i + 8 - 1) % 8] << 5)) & 0xff);

    let mut ctmp: [u8; 8] = Default::default();
    (0..8).for_each(|i| ctmp[i] = ((cstate[i] >> 4) | (cstate[i] << 4)) & 0xff);

    let mut decrypted: [u8; 8] = Default::default();
    (0..8)
        .map(|i| (0x100u16 + phase3[i] as u16 - ctmp[i] as u16) & 0xffu16)
        .enumerate()
        .for_each(|(i, b)| decrypted[i] = b as u8);

    decrypted
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Metrics {
    Temp { value: f32, unit: String },
    CO2 { value: i32, unit: String },
    Hum { value: f32, unit: String },
}

pub fn parse_record(record: Record) -> Option<Metrics> {
    match record.key {
        0x42 => Some(Metrics::Temp {
            value: record.value as f32 / 16.0 - 273.15,
            unit: String::from("°C"),
        }),
        0x44 => Some(Metrics::Hum {
            value: record.value as f32 / 100.0,
            unit: String::from("%"),
        }),
        0x50 => Some(Metrics::CO2 {
            value: record.value as i32,
            unit: String::from("PPM"),
        }),
        _ => None,
    }
}

#[derive(Debug)]
pub struct Record {
    pub key: u8,
    pub value: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub vid: u16,
    pub pid: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vid: 0x04d9,
            pid: 0xa052,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{decrypt, is_encrypted};

    #[test]
    fn test_decrypt() {
        let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let key: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];
        let expected: [u8; 8] = [29, 25, 234, 11, 153, 45, 237, 42];
        assert_eq!(decrypt(&key, data), expected);
    }

    #[test]
    fn test_is_encrypted() {
        let encrypted: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let not_encrypted: [u8; 8] = [1, 2, 3, 4, 0x0d, 6, 7, 8];
        assert!(is_encrypted(encrypted));
        assert!(!is_encrypted(not_encrypted));
    }
}
