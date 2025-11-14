//! Helper crate for running scripts within the `egui` repo

#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::exit)]

mod deny;
mod ios;
pub(crate) mod utils;

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let arg_strings: Vec<_> = std::env::args().skip(1).collect();
    let args: Vec<_> = arg_strings.iter().map(String::as_str).collect();

    match args.as_slice() {
        &[] | &["-h"] | &["--help"] => print_help(),
        &["deny", ..] => deny::deny(&args[1..])?,
        &["ios-sim-smoke"] => ios::sim_smoke()?,
        &["ios-sim-launch"] => ios::sim_launch()?,
        &["ios-run-bundle", ..] => ios::bundle_run(&args[1..])?,
        c => Err(format!("Invalid arguments {c:?}"))?,
    }
    Ok(())
}

fn print_help() {
    let help = "
    xtask help

    Subcommands
    deny: Run cargo-deny for all targets
    ios-sim-smoke: Build the eframe iOS runner and compile the SwiftUI host with xcodebuild
    ios-sim-launch: Build, install, and launch the RunnerSmoke host in the simulator (requires CoreSimulator)
    ios-run-bundle: Use ios-cargo (cargo-bundle) to build/install the egui demo bundle

    Options
    -h, --help: print help and exit
        ";
    println!("{help}");
}
