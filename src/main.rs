mod commands;
mod config;
mod frecency;
mod projects;
mod shell;
mod tui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pj")]
#[command(about = "Project Launcher CLI tool with fuzzy matching")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'a', long = "add", help = "Add current directory to tracked projects")]
    add: bool,

    #[arg(short = 'e', long = "editor", help = "Override editor for this invocation")]
    editor: Option<String>,

    #[arg(short = 't', long = "tags", help = "Filter by tags or add tags (comma-separated)")]
    tags: Option<String>,

    #[arg(long = "cd", help = "Force cd to selected project")]
    force_cd: bool,

    #[arg(long = "no-cd", help = "Skip cd to selected project")]
    no_cd: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Interactive setup wizard")]
    Init,

    #[command(about = "List all tracked projects")]
    List {
        #[arg(short = 't', long = "tags", help = "Filter by tags (comma-separated)")]
        tags: Option<String>,
    },

    #[command(about = "Manage tags for tracked projects")]
    Tag {
        #[arg(help = "Project path")]
        path: Option<PathBuf>,

        #[arg(help = "Tags to add")]
        tags: Vec<String>,

        #[arg(long = "remove", short = 'r', help = "Remove tags instead of adding")]
        remove: bool,
    },

    #[command(about = "Remove a project from tracking")]
    Rm {
        #[arg(long = "missing", help = "Remove all projects with non-existent paths")]
        missing: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init) => commands::init::run(),
        Some(Commands::List { tags }) => commands::list::run(tags),
        Some(Commands::Tag { path, tags, remove }) => commands::tag::run(path, tags, remove),
        Some(Commands::Rm { missing }) => commands::rm::run(missing),
        None => {
            if cli.add {
                commands::add::run(cli.tags)
            } else {
                let cd_override = if cli.force_cd {
                    Some(true)
                } else if cli.no_cd {
                    Some(false)
                } else {
                    None
                };
                commands::select::run(cli.tags, cli.editor, cd_override)
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
