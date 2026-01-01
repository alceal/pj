use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::Write;

use crate::config::Config;

const EDITOR_OPTIONS: &[&str] = &["none", "code", "zed", "vim", "nvim", "cursor", "nano", "emacs"];

struct ConfigEditor {
    config: Config,
    selected_row: usize,
    editor_index: usize,
    custom_editor: Option<String>,
}

impl ConfigEditor {
    fn new(config: Config) -> Self {
        let (editor_index, custom_editor) = if let Some(idx) = EDITOR_OPTIONS
            .iter()
            .position(|&e| e == config.editor)
        {
            (idx, None)
        } else {
            (EDITOR_OPTIONS.len(), Some(config.editor.clone()))
        };

        Self {
            config,
            selected_row: 0,
            editor_index,
            custom_editor,
        }
    }

    fn cycle_editor(&mut self, forward: bool) {
        let max_index = EDITOR_OPTIONS.len(); // includes "custom" as last option
        if forward {
            self.editor_index = (self.editor_index + 1) % (max_index + 1);
        } else {
            if self.editor_index == 0 {
                self.editor_index = max_index;
            } else {
                self.editor_index -= 1;
            }
        }
    }

    fn cycle_current(&mut self, forward: bool) {
        match self.selected_row {
            0 => self.cycle_editor(forward),
            1 => self.config.cd_on_select = !self.config.cd_on_select,
            2 => self.config.git_init_on_add = !self.config.git_init_on_add,
            3 => self.config.gh_create_on_add = !self.config.gh_create_on_add,
            _ => {}
        }
    }

    fn move_selection(&mut self, down: bool) {
        if down {
            self.selected_row = (self.selected_row + 1) % 4;
        } else {
            if self.selected_row == 0 {
                self.selected_row = 3;
            } else {
                self.selected_row -= 1;
            }
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> Result<()> {
        execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;

        // Header
        execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print("Configuration Editor"),
            ResetColor,
            Print(" (Enter: save | Esc: cancel | Tab/Arrows: change)")
        )?;

        let rows = [
            ("editor", self.format_editor_value()),
            ("cd_on_select", self.config.cd_on_select.to_string()),
            ("git_init_on_add", self.config.git_init_on_add.to_string()),
            ("gh_create_on_add", self.config.gh_create_on_add.to_string()),
        ];

        for (i, (name, value)) in rows.iter().enumerate() {
            let is_selected = i == self.selected_row;
            let row = (i + 2) as u16; // Start at row 2 (after header + blank line)

            execute!(out, MoveTo(0, row))?;

            if is_selected {
                execute!(
                    out,
                    SetForegroundColor(Color::Green),
                    Print("> "),
                    ResetColor
                )?;
            } else {
                execute!(out, Print("  "))?;
            }

            execute!(out, Print(format!("{:<18}", name)))?;

            if is_selected {
                execute!(
                    out,
                    SetForegroundColor(Color::Yellow),
                    Print(format!("< {} >", value)),
                    ResetColor
                )?;
            } else {
                execute!(out, Print(format!("  {}  ", value)))?;
            }
        }

        out.flush()?;
        Ok(())
    }

    fn format_editor_value(&self) -> String {
        if self.editor_index < EDITOR_OPTIONS.len() {
            EDITOR_OPTIONS[self.editor_index].to_string()
        } else {
            format!("custom: {}", self.custom_editor.as_deref().unwrap_or(""))
        }
    }

    fn build_config(&self) -> Config {
        let editor = if self.editor_index < EDITOR_OPTIONS.len() {
            EDITOR_OPTIONS[self.editor_index].to_string()
        } else {
            self.custom_editor.clone().unwrap_or_default()
        };

        Config {
            editor,
            cd_on_select: self.config.cd_on_select,
            git_init_on_add: self.config.git_init_on_add,
            gh_create_on_add: self.config.gh_create_on_add,
        }
    }
}

fn prompt_custom_editor<W: Write>(out: &mut W) -> Result<Option<String>> {
    execute!(
        out,
        MoveTo(0, 7),
        Clear(ClearType::FromCursorDown),
        Print("\nEnter custom editor command: "),
        Show
    )?;
    out.flush()?;

    terminal::disable_raw_mode()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    terminal::enable_raw_mode()?;
    execute!(out, Hide)?;

    let input = input.trim().to_string();
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

pub fn run() -> Result<()> {
    let config = Config::load()?;
    let mut editor = ConfigEditor::new(config);
    let mut stderr = std::io::stderr();

    terminal::enable_raw_mode()?;
    execute!(stderr, Hide, Clear(ClearType::All), MoveTo(0, 0))?;
    stderr.flush()?;

    let result = run_editor_loop(&mut editor, &mut stderr);

    // Cleanup
    execute!(stderr, Show, Clear(ClearType::All), MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;

    match result {
        Ok(Some(new_config)) => {
            new_config.save()?;
            eprintln!("Configuration saved.");
        }
        Ok(None) => {
            eprintln!("Configuration unchanged.");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn run_editor_loop<W: Write>(
    editor: &mut ConfigEditor,
    out: &mut W,
) -> Result<Option<Config>> {
    loop {
        editor.render(out)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter => {
                    // If on editor row and it's custom, prompt for input
                    if editor.selected_row == 0 && editor.editor_index >= EDITOR_OPTIONS.len() {
                        if let Some(custom) = prompt_custom_editor(out)? {
                            editor.custom_editor = Some(custom);
                        }
                        continue;
                    }
                    return Ok(Some(editor.build_config()));
                }
                KeyCode::Up => editor.move_selection(false),
                KeyCode::Down => editor.move_selection(true),
                KeyCode::Left => editor.cycle_current(false),
                KeyCode::Right | KeyCode::Tab => editor.cycle_current(true),
                _ => {}
            }
        }
    }
}
