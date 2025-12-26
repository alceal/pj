use anyhow::{Context, Result};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Sh,
    Fish,
}

impl Shell {
    pub fn detect() -> Option<Self> {
        let shell_path = env::var("SHELL").ok()?;
        if shell_path.ends_with("bash") {
            Some(Shell::Bash)
        } else if shell_path.ends_with("zsh") {
            Some(Shell::Zsh)
        } else if shell_path.ends_with("fish") {
            Some(Shell::Fish)
        } else if shell_path.ends_with("sh") {
            Some(Shell::Sh)
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Sh => "sh",
            Shell::Fish => "fish",
        }
    }

    pub fn rc_file(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(match self {
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => home.join(".zshrc"),
            Shell::Sh => home.join(".profile"),
            Shell::Fish => home.join(".config/fish/config.fish"),
        })
    }

    pub fn function_code(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Zsh => {
                r#"
# pj - Project Launcher shell integration
pj() {
    local result
    result=$(command pj "$@")
    local exit_code=$?
    if [[ $exit_code -eq 0 && -n "$result" ]]; then
        cd "$result"
    elif [[ $exit_code -ne 130 ]]; then
        echo "$result"
    fi
}
"#
            }
            Shell::Sh => {
                r#"
# pj - Project Launcher shell integration
pj() {
    result=$(command pj "$@")
    exit_code=$?
    if [ $exit_code -eq 0 ] && [ -n "$result" ]; then
        cd "$result"
    elif [ $exit_code -ne 130 ]; then
        echo "$result"
    fi
}
"#
            }
            Shell::Fish => {
                r#"
# pj - Project Launcher shell integration
function pj
    set -l result (command pj $argv)
    set -l exit_code $status
    if test $exit_code -eq 0 -a -n "$result"
        cd $result
    else if test $exit_code -ne 130
        echo $result
    end
end
"#
            }
        }
    }

    pub fn install_function(&self) -> Result<()> {
        let rc_file = self.rc_file()?;
        let function_code = self.function_code();

        if let Some(parent) = rc_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if rc_file.exists() {
            let content = fs::read_to_string(&rc_file)?;
            if content.contains("# pj - Project Launcher shell integration") {
                return Ok(());
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_file)?;

        writeln!(file, "{}", function_code)?;
        Ok(())
    }
}
