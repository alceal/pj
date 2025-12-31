mod commands;
mod config;
mod frecency;
mod github;
mod projects;
mod shell;
mod tui;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pj")]
#[command(about = "Project Launcher CLI tool with fuzzy matching")]
#[command(version)]
struct Cli {
    /// Filter terms for project selection (smart matching against path + tags)
    #[arg(value_name = "FILTER")]
    filters: Vec<String>,

    /// Add current directory to tracked projects
    #[arg(short = 'a', long = "add")]
    add: bool,

    /// Open skim (multi-select) to remove projects
    #[arg(long = "rm", conflicts_with_all = ["add", "init", "list", "rm_missing"])]
    rm: bool,

    /// Remove all projects with non-existent paths
    #[arg(long = "rm-missing", conflicts_with_all = ["add", "init", "list", "rm"])]
    rm_missing: bool,

    /// Add tags to project(s). Format: -t tag1,tag2 [path]
    /// When used with -a, adds tags to the newly added project
    #[arg(short = 't', long = "tags", value_name = "TAGS", num_args = 0..=1)]
    tags: Option<Option<String>>,

    /// Remove tags from project(s). Format: --rm-tags tag1,tag2 [path]
    #[arg(long = "rm-tags", value_name = "TAGS", num_args = 0..=1, conflicts_with_all = ["add", "rm", "rm_missing", "init", "list"])]
    rm_tags: Option<Option<String>>,

    /// Override editor for this invocation
    #[arg(short = 'e', long = "editor")]
    editor: Option<String>,

    /// Don't open editor (just cd if enabled)
    #[arg(long = "no-editor")]
    no_editor: bool,

    /// Force cd to selected project
    #[arg(long = "cd", conflicts_with = "no_cd")]
    force_cd: bool,

    /// Skip cd to selected project
    #[arg(long = "no-cd", conflicts_with = "force_cd")]
    no_cd: bool,

    /// Interactive setup wizard
    #[arg(long = "init", conflicts_with_all = ["add", "rm", "rm_missing", "list"])]
    init: bool,

    /// List all tracked projects (table output)
    #[arg(long = "list", conflicts_with_all = ["add", "rm", "rm_missing", "init"])]
    list: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = if cli.init {
        commands::init::run()
    } else if cli.list {
        commands::list::run()
    } else if cli.rm {
        commands::rm::run(false)
    } else if cli.rm_missing {
        commands::rm::run(true)
    } else if cli.add {
        // -a/--add: Add current directory
        // If -t is also present, those are tags for the new project
        let tags_for_add = cli.tags.flatten();
        commands::add::run(tags_for_add)
    } else if cli.tags.is_some() {
        // --tags without --add: tag management operation
        let tags_value = cli.tags.unwrap();
        let path = cli.filters.first().map(|s| PathBuf::from(s));
        commands::tag::run(tags_value, path, false)
    } else if cli.rm_tags.is_some() {
        // --rm-tags: remove tags operation
        let tags_value = cli.rm_tags.unwrap();
        let path = cli.filters.first().map(|s| PathBuf::from(s));
        commands::tag::run(tags_value, path, true)
    } else {
        // Project selection mode
        let cd_override = if cli.force_cd {
            Some(true)
        } else if cli.no_cd {
            Some(false)
        } else {
            None
        };
        let editor_override = if cli.no_editor {
            Some(String::new())
        } else {
            cli.editor
        };
        commands::select::run(cli.filters, editor_override, cd_override)
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
