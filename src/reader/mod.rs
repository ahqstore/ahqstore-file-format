use std::{
  collections::HashMap,
  error::Error as FmtErr,
  fmt::Display,
  io::{Error, Read}
};

use ahqstore_types::{InstallerOptions, InstallerOptionsLinux, InstallerOptionsWin32};

use crate::{AppFileType, BinStruct, Schema, VER};

pub struct ExtReader<T: Read> {
  data: T,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ParserError {
  IoError(Error),
  InvalidVersion(u16, u16),
  SerdeParseError(serde_json::Error),
  Invalid,
}

impl From<serde_json::Error> for ParserError {
  fn from(value: serde_json::Error) -> Self {
    ParserError::SerdeParseError(value)
  }
}

impl From<Error> for ParserError {
  fn from(value: Error) -> Self {
    ParserError::IoError(value)
  }
}

impl Display for ParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl FmtErr for ParserError {
  fn description(&self) -> &str {
    match self {
      Self::InvalidVersion(exp, found) => {
        format!("Invalid Version, expected {exp} found {found}").leak()
      }
      Self::IoError(_) | Self::SerdeParseError(_) | Self::Invalid => "Invalid Data",
      _ => "Unknown",
    }
  }
}

impl<T: Read> ExtReader<T> {
  pub fn new(file: T) -> ExtReader<T> {
    ExtReader { data: file }
  }

  pub fn parse(mut self) -> Result<Schema, ParserError> {
    let mut ver = [0u8; 2];
    self.data.read_exact(&mut ver)?;

    let ver = u16::from_be_bytes(ver);
    if ver != VER {
      return Err(ParserError::InvalidVersion(ver, VER));
    }

    let mut typ = [0u8; 2];
    self.data.read_exact(&mut typ)?;
    let typ = u16::from_be_bytes(typ);

    match typ {
      1 => {
        let mut data = Schema {
          data: AppFileType::Bin(BinStruct {
            data: HashMap::new(),
            icon: vec![],
            install: InstallerOptions {
              linux: Some(InstallerOptionsLinux { assetId: 0 }),
              win32: Some(InstallerOptionsWin32 {
                assetId: 0,
                exec: None,
                installerArgs: None,
              }),
            },
            name: "".into(),
          }),
          ver: 0,
        };

        let get_bytes = |r: &mut T, len| {
          let mut data: Vec<u8> = vec![];
          for _ in 0usize..len {
            let mut buf = [0];

            r.read_exact(&mut buf)?;

            data.push(buf[0]);
          }

          return Ok::<Vec<u8>, ParserError>(data);
        };

        if let AppFileType::Bin(d) = &mut data.data {
          loop {
            let mut buf = [0u8; 1];
            self.data.read_exact(&mut buf)?;

            if buf[0] == u8::MAX {
              break;
            }

            let id = buf[0];
            let mut len = [0u8; 8];
            self.data.read_exact(&mut len)?;

            let len = usize::from_be_bytes(len);

            d.data.insert(id, get_bytes(&mut self.data, len)?);
          }

          let mut buf = [0u8; 8];
          self.data.read_exact(&mut buf)?;

          d.icon = get_bytes(&mut self.data, usize::from_be_bytes(buf))?;
          
          let mut buf = [0u8; 8];
          self.data.read_exact(&mut buf)?;
          let buf = usize::from_be_bytes(buf);

          if buf > 0 {
            d.install = serde_json::from_str(&String::from_utf8(get_bytes(&mut self.data, buf)?).map_err(|_| ParserError::Invalid)?)?;
          }

          let mut buf = [0u8; 8];
          self.data.read_exact(&mut buf)?;
          let buf = usize::from_be_bytes(buf);

          if buf > 0 {
            d.name = String::from_utf8(get_bytes(&mut self.data, buf)?).map_err(|_| ParserError::Invalid)?;
          }
        }

        return Ok(data);
      }
      2 => {
        let mut buf = String::new();
        self.data.read_to_string(&mut buf)?;

        return Ok(Schema {
          ver,
          data: AppFileType::Dat(serde_json::from_str(&buf)?),
        });
      }
      3 => {
        let mut buf = String::new();
        self.data.read_to_string(&mut buf)?;

        return Ok(Schema {
          ver,
          data: AppFileType::ODat(buf),
        });
      }
      x => {
        return Err(ParserError::InvalidVersion(typ, x));
      }
    }
  }
}
