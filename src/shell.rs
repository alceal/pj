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
    case "$1" in
        -a|--add|-i|--init|--config|-h|--help|-V|--version)
            command pj "$@"
            return $?
            ;;
    esac
    local output
    output=$(command pj "$@")
    local exit_code=$?
    local dir="" ai_cmd=""
    while IFS= read -r line; do
        if [[ "$line" == __PJ_AI__:* ]]; then
            ai_cmd="${line#__PJ_AI__:}"
        else
            dir="$line"
        fi
    done <<< "$output"
    if [[ $exit_code -eq 0 && -n "$dir" && -d "$dir" ]]; then
        builtin cd "$dir"
    elif [[ $exit_code -ne 130 && -n "$dir" ]]; then
        echo "$dir"
    fi
    if [[ $exit_code -eq 0 && -n "$ai_cmd" ]]; then
        eval "$ai_cmd"
    fi
}
"#
            }
            Shell::Sh => {
                r#"
# pj - Project Launcher shell integration
pj() {
    case "$1" in
        -a|--add|-i|--init|--config|-h|--help|-V|--version)
            command pj "$@"
            return $?
            ;;
    esac
    output=$(command pj "$@")
    exit_code=$?
    dir="" ai_cmd=""
    _pj_ifs="$IFS"
    IFS='
'
    for line in $output; do
        case "$line" in
            __PJ_AI__:*) ai_cmd="${line#__PJ_AI__:}" ;;
            *) dir="$line" ;;
        esac
    done
    IFS="$_pj_ifs"
    if [ $exit_code -eq 0 ] && [ -n "$dir" ] && [ -d "$dir" ]; then
        cd "$dir"
    elif [ $exit_code -ne 130 ] && [ -n "$dir" ]; then
        echo "$dir"
    fi
    if [ $exit_code -eq 0 ] && [ -n "$ai_cmd" ]; then
        eval "$ai_cmd"
    fi
}
"#
            }
            Shell::Fish => {
                r#"
# pj - Project Launcher shell integration
function pj
    switch $argv[1]
        case -a --add -i --init --config -h --help -V --version
            command pj $argv
            return $status
    end
    set -l output (command pj $argv)
    set -l exit_code $status
    set -l dir ""
    set -l ai_cmd ""
    for line in $output
        if string match -q "__PJ_AI__:*" $line
            set ai_cmd (string replace "__PJ_AI__:" "" $line)
        else
            set dir $line
        end
    end
    if test $exit_code -eq 0 -a -n "$dir" -a -d "$dir"
        cd $dir
    else if test $exit_code -ne 130 -a -n "$dir"
        echo $dir
    end
    if test $exit_code -eq 0 -a -n "$ai_cmd"
        eval $ai_cmd
    end
end
"#
            }
        }
    }

    fn end_marker(&self) -> &'static str {
        match self {
            Shell::Fish => "end\n",
            _ => "}\n",
        }
    }

    pub fn install_function(&self) -> Result<()> {
        let rc_file = self.rc_file()?;
        let function_code = self.function_code();
        let marker = "# pj - Project Launcher shell integration";

        if let Some(parent) = rc_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if rc_file.exists() {
            let content = fs::read_to_string(&rc_file)?;
            if let Some(start) = content.find(marker) {
                // Find the end of the existing function
                let after_marker = &content[start..];
                let end_marker = self.end_marker();
                if let Some(end_offset) = after_marker.rfind(end_marker) {
                    let end = start + end_offset + end_marker.len();
                    let mut new_content = String::new();
                    new_content.push_str(&content[..start]);
                    new_content.push_str(function_code.trim());
                    new_content.push('\n');
                    new_content.push_str(&content[end..]);
                    fs::write(&rc_file, new_content)?;
                    return Ok(());
                }
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
