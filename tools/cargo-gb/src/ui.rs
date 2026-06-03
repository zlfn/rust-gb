//! Terminal output: cargo-style status lines, step spinners, and the final
//! per-bank fill bars.

use anstyle::{AnsiColor, Style};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::time::Duration;

fn green_bold() -> Style {
    Style::new().fg_color(Some(AnsiColor::Green.into())).bold()
}

fn cyan_bold() -> Style {
    Style::new().fg_color(Some(AnsiColor::Cyan.into())).bold()
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

/// Render the per-bank fill bars. Bank 0 (the resident region) leads with just
/// its usage; each banked bank also lists the modules placed in it, wrapping a
/// long module list under the bar.
pub fn bank_bars(bank0_used: Option<usize>, banks: &[gb_bank_pack::BankInfo], bank_size: usize) {
    let width = textwrap::termwidth();
    if let Some(used) = bank0_used {
        fill_bar("BANK 0", used, bank_size, &[], width);
    }
    for b in banks {
        fill_bar(&format!("BANK {}", b.bank), b.used, bank_size, &b.modules, width);
    }
}

/// A single fill bar for a non-banked ROM's fixed region.
pub fn rom_bar(used: usize, limit: usize) {
    fill_bar("ROM", used, limit, &[], textwrap::termwidth());
}

/// A fill bar row. The label is bold-green and right-aligned to 12, matching the
/// status verbs, then a 20-cell bar, the byte usage, and any modules (wrapped
/// under the bar when the list is long).
fn fill_bar(label: &str, used: usize, size: usize, modules: &[String], width: usize) {
    let g = cyan_bold();
    let filled = if size == 0 {
        0
    } else {
        (used * 20).div_ceil(size).min(20)
    };
    let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
    let nums = format!("{used:>6}/{size}");

    let visible_prefix = format!("{label:>12} {bar} {nums}   ");
    let colored_prefix = format!(
        "{}{label:>12}{} {bar} {nums}   ",
        g.render(),
        g.render_reset()
    );

    let mut out = anstream::stdout();
    if modules.is_empty() {
        let _ = writeln!(out, "{}", colored_prefix.trim_end());
        return;
    }
    let prefix_w = visible_prefix.chars().count();
    let avail = width.saturating_sub(prefix_w).max(12);
    let pad = " ".repeat(prefix_w);
    for (i, line) in textwrap::wrap(&modules.join(", "), avail).iter().enumerate() {
        if i == 0 {
            let _ = writeln!(out, "{colored_prefix}{line}");
        } else {
            let _ = writeln!(out, "{pad}{line}");
        }
    }
}
