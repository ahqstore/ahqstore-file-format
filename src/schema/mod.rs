use ahqstore_types::*;
use std::collections::HashMap;

#[non_exhaustive]
pub struct Schema {
  pub ver: u16,
  pub data: AppFileType,
}

#[non_exhaustive]
pub struct BinStruct {
  pub data: HashMap<u8, Vec<u8>>,
  pub icon: Vec<u8>,
  pub name: String,
  pub install: InstallerOptions,
}

#[non_exhaustive]
pub enum AppFileType {
  Bin(BinStruct),
  Dat(AHQStoreApplication),
  ODat(String),
}
