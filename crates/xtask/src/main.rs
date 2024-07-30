use std::{path::PathBuf, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    signal,
};

const HELP: &str = "\
App

USAGE:
  app [OPTIONS] --number NUMBER [INPUT]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --number NUMBER       Sets a number
  --opt-number NUMBER   Sets an optional number
  --width WIDTH         Sets width [default: 10]
  --output PATH         Sets an output path

ARGS:
  <INPUT>
";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut pargs = pico_args::Arguments::from_env();
    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let maybe_subcommand = pargs.subcommand()?;
    if maybe_subcommand.is_none() {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let subcommand = maybe_subcommand.unwrap();
    match subcommand.as_ref() {
        "dev" => run_dev_server().await?,
        _ => println!("Unexpected subcommand: {}", subcommand),
    };

    Ok(())
}

async fn run_dev_server() -> anyhow::Result<()> {
    // Spawn the process
    let mut service_handle = Command::new("cargo")
        .args(["run", "--bin", "service"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    redirect_pipes("service", &mut service_handle);

    let service_path: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "..", "service"].iter().collect();
    std::env::set_current_dir(service_path)?;

    let mut tailwind_handle =
        Command::new("/Users/dimaafanasev/tools/tailwindcss")
            // FIXME(hqhs): panics
            // .args(&["--help"])
            .args(["-o", "static/styles.css", "--watch"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

    redirect_pipes("tailwind", &mut tailwind_handle);

    signal::ctrl_c().await.expect("failed to listen for event");

    service_handle.kill().await?;
    tailwind_handle.kill().await?;

    service_handle.wait().await?;
    tailwind_handle.wait().await?;

    Ok(())
}

fn redirect_pipes(name: &'static str, child: &mut Child) {
    // Set up output handling
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Spawn tasks to handle output
    let _stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Some(line) =
            reader.next_line().await.expect("Failed to read stdout")
        {
            println!("{name} STDOUT: {}", line);
        }
    });

    let _stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Some(line) =
            reader.next_line().await.expect("Failed to read stderr")
        {
            eprintln!("{name} STDERR: {}", line);
        }
    });
}
