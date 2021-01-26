use std::env;

use xaction::{cargo_toml, cmd, git, section, Result};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("error: {}", err);
        std::process::exit(1)
    }
}

fn try_main() -> Result<()> {
    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(it) if it == "ci" => (),
        _ => {
            print_usage();
            Err("invalid arguments")?
        }
    }

    let cargo_toml = cargo_toml()?;

    {
        let _s = section("TEST");
        for &release in &[None, Some("--release")] {
            for &log in &[&[][..], &["--features", "log"]] {
                for &force in &[&[][..], &["--features", "force"]] {
                    cmd!(
                        "cargo test {release...} {log...} {force...}
                             --workspace -- --nocapture"
                    )
                    .run()?;
                }
            }
        }
    }

    let version = cargo_toml.version()?;
    let tag = format!("v{}", version);

    let dry_run =
        env::var("CI").is_err() || git::has_tag(&tag)? || git::current_branch()? != "master";
    xaction::set_dry_run(dry_run);

    {
        let _s = section("PUBLISH");
        cargo_toml.publish()?;
        git::tag(&tag)?;
        git::push_tags()?;
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "\
Usage: cargo run -p xtask <SUBCOMMAND>

SUBCOMMANDS:
    ci
"
    )
}