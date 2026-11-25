//! `azure-cost-top`: top costs per meter, resource group and tag from an
//! Azure Cost Management CSV export. Offline; no Azure SDK.

pub mod cli;
pub mod csv;
pub mod date;
pub mod error;
pub mod export;
pub mod money;
pub mod render;
pub mod report;
pub mod tags;

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;

pub use error::Error;

/// Run the tool: parse `args` (without the program name), read the export from
/// a file or `stdin`, and write the output to `out`.
pub fn run<I>(args: I, stdin: &mut dyn Read, out: &mut dyn Write) -> Result<(), Error>
where
    I: IntoIterator<Item = OsString>,
{
    let run = match cli::parse_args(args)? {
        cli::Command::Help => return write_out(out, cli::HELP),
        cli::Command::Version => {
            return write_out(
                out,
                &format!("azure-cost-top {}\n", env!("CARGO_PKG_VERSION")),
            )
        }
        cli::Command::Run(r) => r,
    };
    let bytes = if run.input == Path::new("-") {
        let mut buf = Vec::new();
        stdin
            .read_to_end(&mut buf)
            .map_err(|e| Error::input(format!("cannot read standard input: {e}")))?;
        buf
    } else {
        std::fs::read(&run.input)
            .map_err(|e| Error::input(format!("cannot read {}: {e}", run.input.display())))?
    };
    let name = label(&run.input);
    let text =
        String::from_utf8(bytes).map_err(|_| Error::input(format!("{name} is not valid UTF-8")))?;
    let table = export::Table::parse(&text)?;
    let report = report::build(&name, &table, &run.options)?;
    write_out(out, &render::render(&report, run.format))
}

/// The name shown in the output and in errors. Lossy, only for display.
fn label(input: &Path) -> String {
    if input == Path::new("-") {
        "stdin".to_string()
    } else {
        input.to_string_lossy().into_owned()
    }
}

fn write_out(out: &mut dyn Write, text: &str) -> Result<(), Error> {
    match out.write_all(text.as_bytes()).and_then(|_| out.flush()) {
        Ok(()) => Ok(()),
        // `azure-cost-top x.csv | head` closes the pipe early; that is fine.
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(Error::input(format!("cannot write output: {e}"))),
    }
}
