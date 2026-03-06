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

    /// Delete a profile
    #[arg(short = 'd', long = "delete", value_name = "NAME")]
    delete: Option<String>,

    /// Skip updating ~/.agents/skills
    #[arg(long, global = true)]
    no_agents: bool,

    /// Skip updating ~/.claude/skills
    #[arg(long, global = true)]
    no_claude: bool,

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
    /// List skills in the current profile
    Skills,
}

fn main() -> Result<()> {
    // `rt -` must be intercepted before clap since `-` is not a valid clap flag
    let raw: Vec<String> = std::env::args().collect();
    if raw.get(1).map(|s| s.as_str()) == Some("-") {
        let no_agents = raw.iter().any(|s| s == "--no-agents");
        let no_claude = raw.iter().any(|s| s == "--no-claude");
        let targets = profile::Targets { agents: !no_agents, claude: !no_claude };
        return cmd_switch_previous(&targets);
    }

    let cli = Cli::parse();
    let targets = profile::Targets { agents: !cli.no_agents, claude: !cli.no_claude };

    if let Some(name) = cli.delete {
        return cmd_delete(&name);
    }

    match cli.command {
        Some(Commands::Create { name, path }) => cmd_create(&name, path.as_deref()),
        Some(Commands::Skills) => cmd_skills(),
        None => match cli.profile {
            Some(name) => cmd_switch(&name, &targets),
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

fn cmd_switch(name: &str, targets: &profile::Targets) -> Result<()> {
    profile::switch_to(name, targets)?;
    println!("Switched to profile '{}'", name.green().bold());
    Ok(())
}

fn cmd_switch_previous(targets: &profile::Targets) -> Result<()> {
    match state::read_previous() {
        Some(prev) => {
            profile::switch_to(&prev, targets)?;
            println!("Switched to profile '{}'", prev.green().bold());
            Ok(())
        }
        None => {
            eprintln!("{}", "No previous profile recorded.".red());
            std::process::exit(1);
        }
    }
}

fn cmd_create(name: &str, path: Option<&std::path::Path>) -> Result<()> {
    profile::create_from_path(name, path)
}

fn cmd_skills() -> Result<()> {
    let current = profile::current_profile();
    match current {
        None => {
            eprintln!("{}", "No active profile.".red());
            std::process::exit(1);
        }
        Some(ref name) => println!("{}", name.green().bold()),
    }
    let skills = profile::list_skills()?;
    if skills.is_empty() {
        println!("{}", "  (no skills)".dimmed());
    } else {
        for s in &skills {
            println!("  {}", s);
        }
    }
    Ok(())
}

fn cmd_delete(name: &str) -> Result<()> {
    profile::delete_profile(name)?;
    println!("Deleted profile '{}'", name);
    Ok(())
}
