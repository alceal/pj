use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

enum Multiplexer {
    Tmux,
    Cmux,
}

fn detect_multiplexer() -> Option<Multiplexer> {
    if std::env::var("CMUX_SOCKET").is_ok() || std::env::var("CMUX_SOCKET_PATH").is_ok() {
        return Some(Multiplexer::Cmux);
    }
    if std::env::var("TMUX").is_ok() {
        return Some(Multiplexer::Tmux);
    }
    None
}

fn is_terminal_editor(editor: &str) -> bool {
    Path::new(editor)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "vim" || name == "nvim")
        .unwrap_or(false)
}

fn open_in_tmux_split(editor: &str, path: &Path) -> Result<(), String> {
    let path_str = path.display().to_string();
    let cmd = format!("{} \"{}\"", editor, path_str);

    Command::new("tmux")
        .args(["split-window", "-h", &cmd])
        .status()
        .map_err(|e| format!("Failed to run tmux split-window: {}", e))?;

    Ok(())
}

fn open_in_cmux_split(editor: &str, path: &Path) -> Result<(), String> {
    let output = Command::new("cmux")
        .args(["new-split", "right"])
        .output()
        .map_err(|e| format!("Failed to run cmux new-split: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "cmux new-split failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let surface_ref = stdout
        .split_whitespace()
        .find(|token| token.starts_with("surface:"))
        .ok_or_else(|| format!("Failed to parse surface ref from cmux output: {}", stdout))?
        .to_string();

    // Wait for the new pane's shell to initialize before sending keystrokes
    thread::sleep(Duration::from_millis(200));

    let path_str = path.display().to_string();
    let send_cmd = format!("{} \"{}\"\\n", editor, path_str);

    let send_output = Command::new("cmux")
        .args(["send", "--surface", &surface_ref, "--", &send_cmd])
        .output()
        .map_err(|e| format!("Failed to run cmux send: {}", e))?;

    if !send_output.status.success() {
        return Err(format!(
            "cmux send failed: {}",
            String::from_utf8_lossy(&send_output.stderr)
        ));
    }

    Ok(())
}

pub fn try_open_in_split(editor: &str, path: &Path) -> bool {
    if !is_terminal_editor(editor) {
        return false;
    }

    let multiplexer = match detect_multiplexer() {
        Some(m) => m,
        None => return false,
    };

    let result = match multiplexer {
        Multiplexer::Tmux => open_in_tmux_split(editor, path),
        Multiplexer::Cmux => open_in_cmux_split(editor, path),
    };

    if let Err(e) = result {
        eprintln!("Warning: failed to open editor in split pane: {}", e);
        return false;
    }

    true
}
