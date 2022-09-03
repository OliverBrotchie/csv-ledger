use clap::Parser;
use csv_ledger_lib::{ledger::Ledger, LedgerErr};

use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    process::ExitCode,
};
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The path to the input CSV File.
    path: PathBuf,

    #[clap(short = 'o', long = "output")]
    /// A path to save the output a a file. By default, the output will be printed to stdout.
    output: Option<PathBuf>,
}

impl Args {
    /// Parse cli args or read mocked test enviroment variables.
    /// Whilst this method is ugly, it allows for higher code coverage than using `try_parse` alone.
    fn parse_input() -> Result<Args, clap::Error> {
        if cfg!(feature = "test_args") && env::var("CSV_LEDGER_TEST_ARGS").is_ok() {
            match env::var("CSV_LEDGER_PATH") {
                Ok(p) => Ok(Args {
                    path: p.into(),
                    output: env::var("CSV_LEDGER_OUTPUT").ok().map(|s| s.into()),
                }),
                Err(_) => Err(clap::Error::with_description(
                    "CSV_LEDGER_PATH environment variable not set.".to_string(),
                    clap::ErrorKind::MissingRequiredArgument,
                )),
            }
        } else {
            Args::try_parse()
        }
    }
}

fn main() -> ExitCode {
    let args = match Args::parse_input() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = perform_parse_and_output(args.path, args.output) {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[inline]
/// Run the main functionality of the CLI.
pub fn perform_parse_and_output(path: PathBuf, output: Option<PathBuf>) -> Result<(), LedgerErr> {
    // Open the csv file
    let file = File::open(path).map_err(LedgerErr::Opening)?;

    // Create a new ledger and consume the csv file
    let mut ledger = Ledger::default();
    ledger.consume_csv(BufReader::new(file))?;

    // Output the result
    if let Some(output_path) = output {
        fs::write(output_path, ledger.to_string()).map_err(LedgerErr::Saving)?;
    } else {
        println!("{}", ledger);
    }

    Ok(())
}

#[cfg(test)]
mod perform_parse_and_output {
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn ok_stdout() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let input = "type, client, tx, amount\ndeposit, 1, 1, 1.0";

        fs::write(&path, input).expect("Failed to create temporary file");

        let result = super::perform_parse_and_output(path.clone().into(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn ok_output_file() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let output = dir.path().join("test_output.csv");
        let input = "type, client, tx, amount\ndeposit, 1, 1, 1.0";

        fs::write(&path, input).expect("Unable to write file");

        let result =
            super::perform_parse_and_output(path.clone().into(), Some(output.clone().into()));

        result.unwrap();
        assert!(Path::new(&output).is_file());
    }

    #[test]
    fn err_read_file() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("/foo/test.csv");

        let result = super::perform_parse_and_output(path.clone().into(), None);
        assert!(result.is_err());
    }

    #[test]
    fn err_consume() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let input = "";

        fs::write(&path, input).expect("Failed to create temporary file");

        let result = super::perform_parse_and_output(path.clone().into(), None);
        assert!(result.is_err());
    }

    #[test]
    fn err_output_file() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let output = dir.path().join("/example/test_output.csv");
        let input = "type, client, tx, amount\ndeposit, 1, 1, 1.0";

        fs::write(&path, input).expect("Unable to write file");

        let result =
            super::perform_parse_and_output(path.clone().into(), Some(output.clone().into()));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod args {
    use super::Args;
    use clap::Parser;

    #[test]
    fn debug() {
        let args = Args {
            path: "./tests/test.csv".into(),
            output: Some("./tests/test_output.csv".into()),
        };

        assert_eq!(
            format!("{:?}", args),
            "Args { path: \"./tests/test.csv\", output: Some(\"./tests/test_output.csv\") }"
        );
    }

    #[test]
    fn parse_err() {
        Args::try_parse_from(["foo.csv"]).unwrap_err();
    }
}

// Needed to up the code coverage of main
#[cfg(all(test, feature = "test_args"))]
mod main {
    use crate::main;
    use std::{env, fs};
    use tempfile::tempdir;

    fn reset_args() {
        env::remove_var("CSV_LEDGER_TEST_ARGS");
        env::remove_var("CSV_LEDGER_OUTPUT");
        env::remove_var("CSV_LEDGER_PATH");
    }

    #[test]
    fn ok_stdout() {
        reset_args();
        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let input = "type, client, tx, amount\ndeposit, 1, 1, 1.0";

        fs::write(&path, input).expect("Unable to write file");

        env::set_var("CSV_LEDGER_TEST_ARGS", "true");
        env::set_var("CSV_LEDGER_PATH", path);
        main();
    }

    #[test]
    fn ok_file() {
        reset_args();

        let dir = tempdir().expect("Failed to create temporary directory");
        let path = dir.path().join("test.csv");
        let output = dir.path().join("test_output.csv");
        let input = "type, client, tx, amount\ndeposit, 1, 1, 1.0";

        fs::write(&path, input).expect("Unable to write file");

        env::set_var("CSV_LEDGER_TEST_ARGS", "true");
        env::set_var("CSV_LEDGER_PATH", path);
        env::set_var("CSV_LEDGER_OUTPUT", output);
        main();
    }

    #[test]
    fn err_invalid_path() {
        reset_args();
        let dir = tempdir().expect("Failed to create temporary directory");
        env::set_var("CSV_LEDGER_TEST_ARGS", "true");
        env::set_var("CSV_LEDGER_PATH", dir.path().join("foo.csv"));
        main();
    }

    #[test]
    fn err_missing_path() {
        reset_args();
        env::set_var("CSV_LEDGER_TEST_ARGS", "true");
        main();
    }

    #[test]
    fn err_default_args() {
        reset_args();
        main();
    }
}
