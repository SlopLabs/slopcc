mod cli;
mod driver;

use std::process::ExitCode;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
  let options = match cli::parse_args(std::env::args_os()) {
    Ok(options) => options,
    Err(error) => {
      eprintln!("slopcc: {error}");
      return ExitCode::from(2);
    }
  };

  match driver::run(&options) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("slopcc: {error}");
      ExitCode::from(1)
    }
  }
}
