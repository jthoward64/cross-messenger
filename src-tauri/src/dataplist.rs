use std::{fs::File, io::BufReader, path::Path};

use serde::Deserialize;

extern crate plist;

#[derive(Deserialize)]
pub struct Plist {
    pub iokit: IOKit,
    // root_disk_uuid: String,
}

#[derive(Deserialize)]
pub struct IOKit {
    // #[serde(rename = "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:MLB")]
    // mlb: String,
    // #[serde(rename = "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:ROM")]
    // rom: String,
    // #[serde(rename = "Fyp98tpgj")]
    // fyp98tpgj: String,
    // #[serde(rename = "Gq3489ugfi")]
    // gq3489ugfi: String,
    // #[serde(rename = "IOMACAddress")]
    // iomacaddress: String,
    #[serde(rename = "IOPlatformSerialNumber")]
    pub ioplatformserialnumber: String,
    // #[serde(rename = "IOPlatformUUID")]
    // ioplatformuuid: String,
    // #[serde(rename = "abKPld1EcMni")]
    // abkpld1ecmni: String,
    // #[serde(rename = "board-id")]
    // board_id: String,
    // #[serde(rename = "kbjfrfpoJU")]
    // kbjfrfpoju: String,
    // #[serde(rename = "oycqAZloTNDm")]
    // oycqazlotndm: String,
    // #[serde(rename = "product-name")]
    // product_name: String,
}

#[derive(Debug)]
pub enum PlistError {
    IOError(std::io::Error),
    PlistError(plist::Error),
}

impl From<std::io::Error> for PlistError {
    fn from(error: std::io::Error) -> Self {
        PlistError::IOError(error)
    }
}

impl From<plist::Error> for PlistError {
    fn from(error: plist::Error) -> Self {
        PlistError::PlistError(error)
    }
}

pub fn parse_plist(plist_path: &Path) -> Result<Plist, PlistError> {
    let plist_file = File::open(plist_path)?;
    let plist_reader = BufReader::new(plist_file);
    let plist: Plist = plist::from_reader(plist_reader)?;
    Ok(plist)
}
