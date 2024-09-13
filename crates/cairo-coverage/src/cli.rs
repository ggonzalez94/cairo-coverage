use anyhow::{ensure, Result};
use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    /// Paths to the .json files with trace data.
    #[arg(value_parser = parse_trace_file, num_args = 1.., required = true)]
    pub trace_files: Vec<Utf8PathBuf>,

    /// Path to the output file.
    #[arg(short, long, default_value = "coverage.lcov")]
    pub output_path: Utf8PathBuf,

    /// Include additional components in the coverage report.
    #[arg(long, short, num_args = 1..)]
    pub include: Vec<IncludedComponent>,
}

#[derive(ValueEnum, Debug, Clone, Eq, PartialEq)]
pub enum IncludedComponent {
    /// Run coverage on functions marked with `#[test]` attribute
    TestFunctions,
    /// Run coverage on macros and generated code by them. This includes inline macros, attribute macros, and derive macros.
    Macros,
}

fn parse_trace_file(path: &str) -> Result<Utf8PathBuf> {
    let trace_file = Utf8PathBuf::from(path);

    ensure!(trace_file.exists(), "Trace file does not exist");
    ensure!(trace_file.is_file(), "Trace file is not a file");
    ensure!(
        matches!(trace_file.extension(), Some("json")),
        "Trace file must have a JSON extension"
    );

    Ok(trace_file)
}
