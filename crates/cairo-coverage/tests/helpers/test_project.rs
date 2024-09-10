use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::fs;
use std::path::PathBuf;

pub struct TestProject {
    dir: TempDir,
    coverage_args: Vec<String>,
}

impl TestProject {
    pub fn new(test_project_name: &str) -> Self {
        let dir = TempDir::new().unwrap();

        dir.copy_from(
            format!("tests/data/{test_project_name}/"),
            &["*.toml", "*.cairo"],
        )
        .unwrap();

        Self {
            dir,
            coverage_args: vec![],
        }
    }

    pub fn output(self) -> TestProjectOutput {
        TestProjectOutput(self)
    }

    pub fn run(self) -> TestProjectOutput {
        self.generate_trace_files()
            .run_coverage()
            .run_genhtml()
            .output()
    }

    pub fn coverage_args(mut self, args: &[&str]) -> Self {
        self.coverage_args = args.iter().map(ToString::to_string).collect();
        self
    }

    fn generate_trace_files(self) -> Self {
        SnapboxCommand::new("snforge")
            .arg("test")
            .arg("--save-trace-data")
            .current_dir(&self.dir)
            .assert()
            .success();
        self
    }

    fn find_trace_files(&self) -> Vec<String> {
        let trace_path = self.dir.path().join("snfoundry_trace");
        fs::read_dir(&trace_path)
            .unwrap()
            .map(|entry| entry.unwrap().path().display().to_string())
            .collect()
    }

    fn output_lcov_path(&self) -> PathBuf {
        let output_file_name = self
            .coverage_args
            .iter()
            .position(|arg| arg == "--output-path")
            .and_then(|index| self.coverage_args.get(index + 1))
            .cloned()
            .unwrap_or_else(|| "coverage.lcov".to_string());

        self.dir.path().join(output_file_name)
    }

    fn run_coverage(self) -> Self {
        let trace_files = self.find_trace_files();
        SnapboxCommand::new(cargo_bin!("cairo-coverage"))
            .args(&trace_files)
            .args(&self.coverage_args)
            .current_dir(&self.dir)
            .assert()
            .success();
        self
    }

    fn run_genhtml(self) -> Self {
        SnapboxCommand::new("genhtml")
            .arg(self.output_lcov_path())
            .arg("--output-directory")
            .arg(self.dir.path())
            .assert()
            .success();
        self
    }
}

pub struct TestProjectOutput(TestProject);

impl TestProjectOutput {
    pub fn output_same_as_in_file(&self, expected_file: &str) {
        let content = fs::read_to_string(self.0.output_lcov_path()).unwrap();

        let expected = fs::read_to_string(format!("tests/expected_output/{expected_file}"))
            .unwrap()
            .replace(
                "{dir}",
                &self.0.dir.canonicalize().unwrap().display().to_string(),
            );
        assert_eq!(content, expected);
    }

    pub fn dir(&self) -> &TempDir {
        &self.0.dir
    }
}
