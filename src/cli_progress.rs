use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};

#[derive(Debug)]
pub struct ProgressBar {
    pub max_width: u16,
    pub template: String,
    pub start_char: char,
    pub end_char: char,
    pub full_char: char,
    pub empty_char: char,
}

const DEFAULT_MAX_WIDTH: u16 = 79;
const DEFAULT_TEMPLATE: &'static str = "Working: {percentage}% {progress} {description}";
const DEFAULT_START_CHAR: char = '[';
const DEFAULT_END_CHAR: char = ']';
const DEFAULT_FULL_CHAR: char = '#';
const DEFAULT_EMPTY_CHAR: char = '-';

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            max_width: DEFAULT_MAX_WIDTH,
            template: String::from(DEFAULT_TEMPLATE),
            start_char: DEFAULT_START_CHAR,
            end_char: DEFAULT_END_CHAR,
            full_char: DEFAULT_FULL_CHAR,
            empty_char: DEFAULT_EMPTY_CHAR,
        }
    }
}

impl ProgressBar {
    pub fn update(&self, percentage: f32, description: &str) -> Result<()> {
        let (cols, _) = crossterm::terminal::size().context("get terminal size")?;

        let cols = cols.min(self.max_width);

        let preprocessed_template = self
            .template
            .replace("{percentage}", &format!("{percentage:.2}"))
            .replace("{description}", description);

        let progress_len = if cols as usize <= preprocessed_template.len() - 10 {
            2
        } else {
            cols as usize - (preprocessed_template.len() - 10)
        };

        let mut progress = String::new();
        progress.push(self.start_char);
        for _ in 0..(percentage / 100f32 * progress_len as f32 - 2f32).floor() as usize {
            progress.push(self.full_char);
        }
        for _ in 0..progress_len
            - 2
            - (percentage / 100f32 * (progress_len as f32 - 2f32) as f32).floor() as usize
        {
            progress.push(self.empty_char);
        }
        progress.push(self.end_char);

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
