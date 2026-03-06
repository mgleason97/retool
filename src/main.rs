mod profile;
mod state;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

/// Skill profile switcher
#[derive(Parser)]
#[command(name = "rt", version, about = "Skill profile switcher")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Print current profile name
    #[arg(short = 'c', long = "current")]
    current: bool,

    /// Delete a profile
    #[arg(short = 'd', long = "delete", value_name = "NAME")]
    delete: Option<String>,

    /// Profile name to switch to
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new profile by snapshotting skills
    Create {
        /// Profile name
        name: String,
        /// Source path (defaults to ~/.agents/skills)
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    // `rt -` must be intercepted before clap since `-` is not a valid clap flag
    let raw: Vec<String> = std::env::args().collect();
    if raw.len() == 2 && raw[1] == "-" {
        return cmd_switch_previous();
    }

    let cli = Cli::parse();

    if cli.current {
        return cmd_current();
    }

    if let Some(name) = cli.delete {
        return cmd_delete(&name);
    }

    match cli.command {
        Some(Commands::Create { name, path }) => cmd_create(&name, path.as_deref()),
        None => match cli.profile {
            Some(name) => cmd_switch(&name),
            None => cmd_list(),
        },
    }
}

fn cmd_list() -> Result<()> {
    let profiles = profile::list_profiles()?;
    let current = profile::current_profile();

    if profiles.is_empty() {
        println!("{}", "No profiles found. Create one with: rt create <name>".dimmed());
        return Ok(());
    }

    for name in &profiles {
        if current.as_deref() == Some(name.as_str()) {
            println!("{}", format!("* {}", name).green().bold());
        } else {
            println!("  {}", name);
        }
    }
    Ok(())
}

fn cmd_switch(name: &str) -> Result<()> {
    profile::switch_to(name)?;
    println!("Switched to profile '{}'", name.green().bold());
    Ok(())
}

fn cmd_switch_previous() -> Result<()> {
    match state::read_previous() {
        Some(prev) => {
            profile::switch_to(&prev)?;
            println!("Switched to profile '{}'", prev.green().bold());
            Ok(())
        }
        None => {
            eprintln!("{}", "No previous profile recorded.".red());
            std::process::exit(1);
        }
    }
}

fn cmd_current() -> Result<()> {
    match profile::current_profile() {
        Some(name) => println!("{}", name),
        None => {
            eprintln!("{}", "No active profile.".red());
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_create(name: &str, path: Option<&std::path::Path>) -> Result<()> {
    profile::create_from_path(name, path)
}

fn cmd_delete(name: &str) -> Result<()> {
    profile::delete_profile(name)?;
    println!("Deleted profile '{}'", name);
    Ok(())
}
