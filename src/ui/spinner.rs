//! Loading spinner shown while fetching documentation over the network,
//! the Rust equivalent of `rich.progress`/`console.status` in `pyhelp`.

use super::Theme;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Spinner {
    bar: Option<ProgressBar>,
}

impl Spinner {
    /// Start a spinner with `message`. In `--quiet`/`--plain`/non-TTY mode
    /// this degrades to a single plain line on stderr instead of an
    /// animated spinner, so piped output and CI logs stay clean.
    pub fn start(theme: &Theme, message: &str) -> Self {
        if !theme.color || theme.quiet {
            if !theme.quiet {
                eprintln!("{}", theme.decorate(message));
            }
            return Spinner { bar: None };
        }

        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(80));
        let template = ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());
        bar.set_style(template);
        bar.set_message(theme.decorate(message));
        Spinner { bar: Some(bar) }
    }

    /// Stop and clear the spinner line.
    pub fn finish(self) {
        if let Some(bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}
