use anyhow::Result;
use dialoguer::{Input, Select};

use crate::config::Config;
use crate::github::is_gh_installed;
use crate::shell::Shell;

pub fn run() -> Result<()> {
    let existing_config = Config::load().ok();
    let shell = Shell::detect();

    if let Some(s) = &shell {
        eprintln!("Detected shell: {}", s.name());
    } else {
        eprintln!("Could not detect shell, defaulting to bash-compatible");
    }

    let editor_options = vec!["none", "code", "zed", "cursor", "nvim", "vim", "nano", "emacs", "other"];
    let editor_selection = match Select::new()
        .with_prompt("Which editor would you like to use? (Esc to cancel)")
        .items(&editor_options)
        .default(1)
        .interact_opt()?
    {
        Some(sel) => sel,
        None => {
            eprintln!("Setup cancelled.");
            return Ok(());
        }
    };

    let editor = if editor_selection == editor_options.len() - 1 {
        Input::<String>::new()
            .with_prompt("Enter your editor command")
            .interact_text()?
    } else {
        editor_options[editor_selection].to_string()
    };

    let cd_on_select = match Select::new()
        .with_prompt("Change directory when selecting a project?")
        .items(&["yes", "no"])
        .default(0)
        .interact_opt()?
    {
        Some(sel) => sel == 0,
        None => {
            eprintln!("Setup cancelled.");
            return Ok(());
        }
    };

    let mut git_init_on_add = match Select::new()
        .with_prompt("Initialize git repo when adding projects?")
        .items(&["yes", "no"])
        .default(0)
        .interact_opt()?
    {
        Some(sel) => sel == 0,
        None => {
            eprintln!("Setup cancelled.");
            return Ok(());
        }
    };

    let should_ask_github =
        git_init_on_add || existing_config.as_ref().map_or(false, |c| c.git_init_on_add);

    let gh_create_on_add = if should_ask_github {
        let gh_hint = if is_gh_installed() {
            "Create GitHub remote when adding a project?"
        } else {
            "Create GitHub remote when adding a project? (gh CLI not installed)"
        };
        match Select::new()
            .with_prompt(gh_hint)
            .items(&["yes", "no"])
            .default(1)
            .interact_opt()?
        {
            Some(sel) => sel == 0,
            None => {
                eprintln!("Setup cancelled.");
                return Ok(());
            }
        }
    } else {
        false
    };

    if gh_create_on_add && !git_init_on_add {
        eprintln!("Warning: GitHub remote creation requires git initialization.");
        eprintln!("Enabling git_init_on_add automatically.");
        git_init_on_add = true;
    }

    let config = Config {
        editor,
        cd_on_select,
        git_init_on_add,
        gh_create_on_add,
    };

    config.save()?;
    eprintln!(
        "Configuration saved to {}",
        Config::config_path()?.display()
    );

    if cd_on_select {
        let shell = shell.unwrap_or(Shell::Bash);
        shell.install_function()?;
        eprintln!("Shell function added to {}", shell.rc_file()?.display());
        eprintln!(
            "Please restart your shell or run: source {}",
            shell.rc_file()?.display()
        );
    }

    Ok(())
}
