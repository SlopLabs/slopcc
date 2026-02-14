use std::path::Path;

use slopcc_common::prelude::SourceMap;

use crate::cli::CliOptions;

#[derive(thiserror::Error, Debug)]
pub enum DriverError {
  #[error("{0}")]
  Source(#[from] slopcc_common::prelude::SourceError),
  #[error("tokenizer phase is not implemented yet")]
  TokenizerNotImplemented,
}

pub fn run(options: &CliOptions) -> Result<(), DriverError> {
  if options.show_version {
    println!("slopcc {}", env!("CARGO_PKG_VERSION"));
    return Ok(());
  }

  let mut sources = SourceMap::new();
  for input in &options.inputs {
    sources.add_file_from_path(Path::new(input))?;
  }

  if options.dry_run {
    for input in &options.inputs {
      println!("would compile: {}", input.display());
    }
  }

  Err(DriverError::TokenizerNotImplemented)
}
