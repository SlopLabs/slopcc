use std::{
  ffi::OsString,
  path::PathBuf,
};

use clap::{
  ArgAction,
  Parser,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CompileMode {
  PreprocessOnly,
  CompileOnly,
  AssembleOnly,
  Link,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CliOptions {
  pub inputs: Vec<PathBuf>,
  pub output: Option<PathBuf>,
  pub mode: CompileMode,
  pub include_dirs: Vec<PathBuf>,
  pub defines: Vec<OsString>,
  pub undefs: Vec<OsString>,
  pub std: Option<OsString>,
  pub opt: Option<OsString>,
  pub verbose: bool,
  pub dry_run: bool,
  pub show_version: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
  #[error("{0}")]
  Clap(#[from] clap::Error),
  #[error("no input files")]
  NoInputFiles,
}

#[derive(Parser, Debug)]
#[command(
  name = "slopcc",
  disable_help_flag = true,
  disable_version_flag = true,
  trailing_var_arg = false
)]
struct ClapCli {
  #[arg(short = 'E', action = ArgAction::SetTrue)]
  preprocess_only: bool,
  #[arg(short = 'S', action = ArgAction::SetTrue)]
  compile_only: bool,
  #[arg(short = 'c', action = ArgAction::SetTrue)]
  assemble_only: bool,
  #[arg(short = 'o')]
  output: Option<PathBuf>,
  #[arg(short = 'I')]
  include_dirs: Vec<PathBuf>,
  #[arg(short = 'D')]
  defines: Vec<OsString>,
  #[arg(short = 'U')]
  undefs: Vec<OsString>,
  #[arg(long = "std")]
  std: Option<OsString>,
  #[arg(short = 'O')]
  opt: Option<OsString>,
  #[arg(short = 'v', action = ArgAction::SetTrue)]
  verbose: bool,
  #[arg(short = '#', action = ArgAction::Count)]
  dry_run_count: u8,
  #[arg(long = "version", action = ArgAction::SetTrue)]
  show_version: bool,
  #[arg(value_name = "INPUT")]
  inputs: Vec<PathBuf>,
}

pub fn parse_args<I>(args: I) -> Result<CliOptions, CliError>
where
  I: IntoIterator<Item = OsString>,
{
  let normalized = normalize_gcc_args(args.into_iter().collect());
  let parsed = ClapCli::try_parse_from(normalized)?;

  if !parsed.show_version && parsed.inputs.is_empty() {
    return Err(CliError::NoInputFiles);
  }

  let mode = if parsed.preprocess_only {
    CompileMode::PreprocessOnly
  } else if parsed.compile_only {
    CompileMode::CompileOnly
  } else if parsed.assemble_only {
    CompileMode::AssembleOnly
  } else {
    CompileMode::Link
  };

  Ok(CliOptions {
    inputs: parsed.inputs,
    output: parsed.output,
    mode,
    include_dirs: parsed.include_dirs,
    defines: parsed.defines,
    undefs: parsed.undefs,
    std: parsed.std,
    opt: parsed.opt,
    verbose: parsed.verbose,
    dry_run: parsed.dry_run_count > 0,
    show_version: parsed.show_version,
  })
}

fn normalize_gcc_args(args: Vec<OsString>) -> Vec<OsString> {
  let mut normalized = Vec::with_capacity(args.len());
  for arg in args {
    if let Some(s) = arg.to_str() {
      if let Some(rest) = s.strip_prefix("-std=") {
        let mut mapped = OsString::from("--std=");
        mapped.push(rest);
        normalized.push(mapped);
        continue;
      }
      if s == "-std" {
        normalized.push(OsString::from("--std"));
        continue;
      }
    }
    normalized.push(arg);
  }
  normalized
}

#[cfg(test)]
mod tests {
  use super::{
    parse_args,
    CliError,
    CompileMode,
  };
  use std::ffi::OsString;

  fn args(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
  }

  #[test]
  fn parses_basic_input_and_mode() {
    let opts = parse_args(args(&["slopcc", "-c", "file.c"]))
      .expect("parser should accept assemble-only mode");
    assert_eq!(opts.mode, CompileMode::AssembleOnly);
    assert_eq!(opts.inputs.len(), 1);
  }

  #[test]
  fn parses_attached_and_split_values() {
    let opts = parse_args(args(&[
      "slopcc", "-Iinc", "-I", "inc2", "-DNAME=1", "-D", "X", "-UOLD", "-U", "Y", "a.c",
    ]))
    .expect("parser should accept include/define/undef forms");
    assert_eq!(opts.include_dirs.len(), 2);
    assert_eq!(opts.defines.len(), 2);
    assert_eq!(opts.undefs.len(), 2);
  }

  #[test]
  fn parses_std_opt_output_dry_run_and_version() {
    let opts = parse_args(args(&[
      "slopcc", "-std=c11", "-O2", "-o", "out.o", "-###", "-v", "a.c",
    ]))
    .expect("parser should accept std/opt/output/dry-run/verbose");
    assert!(opts.dry_run);
    assert!(opts.verbose);
    assert!(opts.std.is_some());
    assert!(opts.opt.is_some());
    assert!(opts.output.is_some());

    let version_only =
      parse_args(args(&["slopcc", "--version"])).expect("version should not require input files");
    assert!(version_only.show_version);
  }

  #[test]
  fn missing_value_is_reported_by_clap() {
    let err =
      parse_args(args(&["slopcc", "-o"])).expect_err("missing -o value should return clap error");
    assert!(matches!(err, CliError::Clap(_)));
  }

  #[test]
  fn reports_missing_inputs_without_version() {
    let err =
      parse_args(args(&["slopcc", "-c"])).expect_err("compile mode requires at least one input");
    assert!(matches!(err, CliError::NoInputFiles));
  }
}
