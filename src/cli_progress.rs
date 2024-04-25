use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
pub struct ProgressBar {}

const DEFAULT_MAX_WIDTH: u16 = 79;
const DEFAULT_TEMPLATE: &'static str = "Working: {percentage} {progress} {description}";
const DEFAULT_START_CHAR: char = '[';
const DEFAULT_END_CHAR: char = ']';
const DEFAULT_FULL_CHAR: char = '#';
const DEFAULT_EMPTY_CHAR: char = '-';

impl ProgressBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, percentage: usize, description: &str) -> Result<()> {
        let template = String::from(DEFAULT_TEMPLATE);
        let (cols, _) = crossterm::terminal::size().context("get terminal size")?;

        let cols = cols.min(DEFAULT_MAX_WIDTH);

        let preprocessed_template = template
            .replace("{percentage}", &format!("{percentage}"))
            .replace("{description}", description);

        let progress_len = cols as usize - (preprocessed_template.len() - 10);

        let mut progress = String::new();
        progress.push(DEFAULT_START_CHAR);
        for _ in 0..(percentage as f32 / 100.0 * cols as f32).floor() as usize {
            progress.push(DEFAULT_FULL_CHAR);
        }
        for _ in 0..progress_len - 2 - (percentage as f32 / 100.0 * cols as f32).floor() as usize {
            progress.push(DEFAULT_EMPTY_CHAR);
        }
        progress.push(DEFAULT_END_CHAR);

        let mut stdout = std::io::stdout();
        stdout
            .queue(cursor::SavePosition)
            .context("store cursor position before rendering")?;
        stdout
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))
            .context("clear the old progressbar")?;
        stdout
            .write_all(
                preprocessed_template
                    .replace("{progress}", &progress)
                    .as_bytes(),
            )
            .context("print progress bar")?;
        stdout
            .queue(cursor::RestorePosition)
            .context("move back to initial position")?;
        stdout.flush().context("flush stdout")?;

        Ok(())
    }
}
