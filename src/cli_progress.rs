use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};

const DEFAULT_MAX_WIDTH: u16 = 79;
const DEFAULT_TEMPLATE: &'static str = "Working: {percentage}% [{progress}] {description}";
const DEFAULT_FULL_CHARS: &[char] = &['#'];
const DEFAULT_EMPTY_CHAR: char = '-';

pub const UNICODE_BAR_FULL_CHARS: &[char] = &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏'];

pub struct ProgressBar {
    pub max_width: u16,
    pub template: String,
    pub full_chars: Vec<char>,
    pub empty_char: char,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            max_width: DEFAULT_MAX_WIDTH,
            template: String::from(DEFAULT_TEMPLATE),
            full_chars: Vec::from(DEFAULT_FULL_CHARS),
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
            0
        } else {
            cols as usize - (preprocessed_template.len() - 10)
        };

        let mut progress = String::new();
        let full_char_with_fraction = percentage / 100f32 * progress_len as f32;
        let full_char_num = full_char_with_fraction.floor();

        for _ in 0..full_char_num as usize {
            progress.push(
                *self
                    .full_chars
                    .first()
                    .expect("the full_char vector has at least one element."),
            );
        }

        let full_char_fraction = full_char_with_fraction - full_char_num;
        let mut fraction_render_offset: usize = 0;
        if full_char_fraction > f32::EPSILON && self.full_chars.len() > 1 {
            fraction_render_offset = 1;
            let fraction_index = self.full_chars.len()
                - 1
                - (full_char_fraction * (self.full_chars.len() - 2) as f32).round() as usize;
            progress.push(self.full_chars[fraction_index]);
        }

        for _ in 0..progress_len - full_char_num as usize - fraction_render_offset {
            progress.push(self.empty_char);
        }

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
