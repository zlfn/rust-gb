//! Terminal output: cargo-style status lines, step spinners, and the final
//! per-bank fill bars.

use anstyle::{AnsiColor, Style};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::time::Duration;

fn green_bold() -> Style {
    Style::new().fg_color(Some(AnsiColor::Green.into())).bold()
}

/// Print a right-aligned bold-green verb followed by a message (cargo's style).
pub fn status(verb: &str, msg: &str) {
    let g = green_bold();
    let mut out = anstream::stdout();
    let _ = writeln!(out, "{}{verb:>12}{} {msg}", g.render(), g.render_reset());
}

/// Start a spinner for a slow step, labelled with a right-aligned bold-green verb.
pub fn spinner(verb: &str, msg: &str) -> ProgressBar {
    let g = green_bold();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{prefix} {spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
    );
    pb.set_prefix(format!("{}{verb:>12}{}", g.render(), g.render_reset()));
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Render the per-bank fill bars. Each bank shows a 20-cell bar, its byte usage,
/// and the modules placed in it, wrapping a long module list under the bar.
pub fn bank_bars(banks: &[gb_bank_pack::BankInfo], bank_size: usize) {
    let width = textwrap::termwidth();
    for b in banks {
        let filled = if bank_size == 0 {
            0
        } else {
            (b.used * 20).div_ceil(bank_size).min(20)
        };
        let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
        let prefix = format!("   bank {:<3} {bar} {:>6}/{}   ", b.bank, b.used, bank_size);

        if b.modules.is_empty() {
            println!("{}", prefix.trim_end());
            continue;
        }
        let pad = " ".repeat(prefix.chars().count());
        let opts = textwrap::Options::new(width)
            .initial_indent(&prefix)
            .subsequent_indent(&pad);
        println!("{}", textwrap::fill(&b.modules.join(", "), opts));
    }
}
