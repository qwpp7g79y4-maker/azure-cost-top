use std::io::Write;
use std::process::ExitCode;

mod stdout_probe;

fn main() -> ExitCode {
    if stdout_probe::stdout_was_closed_at_start() {
        // Rust has already replaced the closed descriptor with /dev/null, so the
        // write would "succeed" and hide the lost output. Refuse instead.
        let _ = writeln!(
            std::io::stderr(),
            "azure-cost-top: standard output is closed"
        );
        return ExitCode::from(2);
    }
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
