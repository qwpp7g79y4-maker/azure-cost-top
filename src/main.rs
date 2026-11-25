use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match azure_cost_top::run(args, &mut stdin.lock(), &mut stdout.lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // A closed or full stderr must not hide the exit code, so no eprintln!
            // (it panics on a write error).
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "azure-cost-top: {e}");
            if matches!(e, azure_cost_top::Error::Usage(_)) {
                let _ = writeln!(stderr, "Try 'azure-cost-top --help'.");
            }
            ExitCode::from(e.exit_code())
        }
    }
}
