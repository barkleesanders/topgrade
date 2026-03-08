use std::cmp::{max, min};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use chrono::{Local, Timelike};
use color_eyre::eyre;
use color_eyre::eyre::Context;
use console::{Color, Key, Term, measure_text_width, style};
use notify_rust::{Notification, Timeout};
use rust_i18n::t;
use tracing::{debug, error};
#[cfg(windows)]
use which_crate::which;

use crate::command::CommandExt;
use crate::runner::StepResult;

static TERMINAL: LazyLock<Mutex<Terminal>> = LazyLock::new(|| Mutex::new(Terminal::new()));

#[cfg(unix)]
pub fn shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
}

#[cfg(windows)]
pub fn shell() -> &'static str {
    which("pwsh").map(|_| "pwsh").unwrap_or("powershell")
}

#[allow(clippy::disallowed_methods)]
pub fn run_shell() -> eyre::Result<()> {
    Command::new(shell()).env("IN_TOPGRADE", "1").status_checked()
}

struct Terminal {
    width: Option<u16>,
    prefix: String,
    term: Term,
    set_title: bool,
    display_time: bool,
    desktop_notification: bool,
    show_step_ids: bool,
    current_step_id: Option<String>,
    separator_color: Option<Color>,
}

impl Terminal {
    fn new() -> Self {
        let term = Term::stdout();
        Self {
            width: term.size_checked().map(|(_, w)| w),
            term,
            prefix: env::var("TOPGRADE_PREFIX").map_or_else(|_| String::new(), |prefix| format!("({prefix}) ")),
            set_title: true,
            display_time: true,
            desktop_notification: false,
            show_step_ids: false,
            current_step_id: None,
            separator_color: None,
        }
    }

    fn set_show_step_ids(&mut self, show: bool) {
        self.show_step_ids = show;
    }

    fn set_separator_color(&mut self, color: Option<Color>) {
        self.separator_color = color;
    }

    fn set_current_step_id(&mut self, step_id: Option<String>) {
        self.current_step_id = step_id;
    }

    fn set_desktop_notifications(&mut self, desktop_notifications: bool) {
        self.desktop_notification = desktop_notifications;
    }

    fn set_title(&mut self, set_title: bool) {
        self.set_title = set_title;
    }

    fn display_time(&mut self, display_time: bool) {
        self.display_time = display_time;
    }

    fn notify_desktop<P: AsRef<str>>(&self, message: P, timeout: Option<Duration>) {
        debug!("Desktop notification: {}", message.as_ref());
        let mut notification = Notification::new();
        notification
            .summary("Topgrade")
            .body(message.as_ref())
            .appname("topgrade");

        if let Some(timeout) = timeout {
            notification.timeout(Timeout::Milliseconds(timeout.as_millis() as u32));
        }
        notification.show().ok();
    }

    fn print_separator<P: AsRef<str>>(&mut self, message: P) {
        if self.set_title {
            self.term
                .set_title(format!("{}Topgrade - {}", self.prefix, message.as_ref()));
        }

        if self.desktop_notification {
            self.notify_desktop(message.as_ref(), Some(Duration::from_secs(5)));
        }

        // Append the step ID (e.g., "[brew_cask]") when --show-step-ids is enabled
        let display_message = if self.show_step_ids {
            if let Some(ref step_id) = self.current_step_id {
                format!("{} [{}]", message.as_ref(), step_id)
            } else {
                String::from(message.as_ref())
            }
        } else {
            String::from(message.as_ref())
        };

        let now = Local::now();
        let message = if self.display_time {
            format!(
                "{}{:02}:{:02}:{:02} - {}",
                self.prefix,
                now.hour(),
                now.minute(),
                now.second(),
                display_message
            )
        } else {
            display_message
        };

        match self.width {
            Some(width) => {
                let styled = style(format!(
                    "\n── {} {:─^border$}",
                    message,
                    "",
                    border = max(
                        2,
                        min(80, width as usize)
                            .checked_sub(4)
                            .and_then(|e| e.checked_sub(measure_text_width(&message)))
                            .unwrap_or(0)
                    )
                ))
                .bold();
                let styled = if let Some(color) = self.separator_color {
                    styled.fg(color)
                } else {
                    styled
                };
                self.term.write_fmt(format_args!("{styled}\n")).ok();
            }
            None => {
                self.term.write_fmt(format_args!("―― {message} ――\n")).ok();
            }
        }
    }

    #[allow(dead_code)]
    fn print_error<P: AsRef<str>, Q: AsRef<str>>(&mut self, key: Q, message: P) {
        let key = key.as_ref();
        let message = message.as_ref();
        self.term
            .write_fmt(format_args!(
                "{} {}",
                style(format!("{}", t!("{key} failed:", key = key))).red().bold(),
                message
            ))
            .ok();
    }

    #[allow(dead_code)]
    fn print_warning<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        self.term
            .write_fmt(format_args!("{}\n", style(message).yellow().bold()))
            .ok();
    }

    #[allow(dead_code)]
    fn print_info<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        self.term
            .write_fmt(format_args!("{}\n", style(message).blue().bold()))
            .ok();
    }

    fn print_result<P: AsRef<str>>(&mut self, key: P, result: &StepResult) {
        let key = key.as_ref();

        self.term
            .write_fmt(format_args!(
                "{}: {}\n",
                key,
                match result {
                    StepResult::Success(updated) => {
                        let mut s = format!("{}", style(t!("OK")).bold().green());
                        if let Some(updated) = updated {
                            s.push_str(&format!(": {updated}"));
                        }
                        s
                    }
                    StepResult::Failure => format!("{}", style(t!("FAILED")).bold().red()),
                    StepResult::Ignored => format!("{}", style(t!("IGNORED")).bold().yellow()),
                    StepResult::SkippedMissingSudo => format!(
                        "{}: {}",
                        style(t!("SKIPPED")).bold().yellow(),
                        t!("Could not find sudo")
                    ),
                    StepResult::Skipped(reason) => format!("{}: {}", style(t!("SKIPPED")).bold().blue(), reason),
                }
            ))
            .ok();
    }

    #[allow(dead_code)]
    fn prompt_yesno(&mut self, question: &str) -> Result<bool, io::Error> {
        self.term
            .write_fmt(format_args!(
                "{}",
                style(format!("{question} {}", t!("(Y)es/(N)o"))).yellow().bold()
            ))
            .ok();

        loop {
            match self.term.read_char()? {
                'y' | 'Y' => break Ok(true),
                'n' | 'N' | '\r' | '\n' => break Ok(false),
                _ => (),
            }
        }
    }

    fn should_retry(&mut self, step_name: &str) -> eyre::Result<ShouldRetry> {
        if self.width.is_none() {
            return Ok(ShouldRetry::No);
        }

        if self.set_title {
            self.term.set_title(format!("Topgrade - {}", t!("Awaiting user")));
        }

        if self.desktop_notification {
            self.notify_desktop(format!("{}", t!("{step_name} failed", step_name = step_name)), None);
        }

        let prompt_inner = style(format!("{}{}", self.prefix, t!("Retry? (y)es/(N)o/(s)hell/(q)uit")))
            .yellow()
            .bold();

        self.term.write_fmt(format_args!("\n{prompt_inner}")).ok();

        let answer = loop {
            match self.term.read_key() {
                Ok(Key::Char('y' | 'Y')) => break Ok(ShouldRetry::Yes),
                Ok(Key::Char('s' | 'S')) => {
                    println!(
                        "\n\n{}\n",
                        t!("Dropping you to shell. Fix what you need and then exit the shell.")
                    );
                    if let Err(err) = run_shell().context("Failed to run shell") {
                        self.term.write_fmt(format_args!("{err:?}\n{prompt_inner}")).ok();
                    } else {
                        break Ok(ShouldRetry::Yes);
                    }
                }
                Ok(Key::Char('n' | 'N') | Key::Enter) => break Ok(ShouldRetry::No),
                Err(e) => {
                    error!("Error reading from terminal: {}", e);
                    break Ok(ShouldRetry::No);
                }
                Ok(Key::Char('q' | 'Q')) => {
                    break Ok(ShouldRetry::Quit);
                }
                _ => (),
            }
        };

        self.term.write_str("\n").ok();

        answer
    }

    fn get_char(&self) -> Result<Key, io::Error> {
        self.term.read_key()
    }
}

#[derive(Clone, Copy)]
pub enum ShouldRetry {
    Yes,
    No,
    Quit,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

pub fn should_retry(step_name: &str) -> eyre::Result<ShouldRetry> {
    TERMINAL.lock().unwrap().should_retry(step_name)
}

pub fn print_separator<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_separator(message);
}

#[allow(dead_code)]
pub fn print_error<P: AsRef<str>, Q: AsRef<str>>(key: Q, message: P) {
    TERMINAL.lock().unwrap().print_error(key, message);
}

#[allow(dead_code)]
pub fn print_warning<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_warning(message);
}

#[allow(dead_code)]
pub fn print_info<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_info(message);
}

pub fn print_result<P: AsRef<str>>(key: P, result: &StepResult) {
    TERMINAL.lock().unwrap().print_result(key, result);
}

/// Tells whether the terminal is dumb.
pub fn is_dumb() -> bool {
    TERMINAL.lock().unwrap().width.is_none()
}

pub fn get_key() -> Result<Key, io::Error> {
    TERMINAL.lock().unwrap().get_char()
}

pub fn set_title(set_title: bool) {
    TERMINAL.lock().unwrap().set_title(set_title);
}

pub fn set_desktop_notifications(desktop_notifications: bool) {
    TERMINAL
        .lock()
        .unwrap()
        .set_desktop_notifications(desktop_notifications);
}

#[allow(dead_code)]
pub fn prompt_yesno(question: &str) -> Result<bool, io::Error> {
    TERMINAL.lock().unwrap().prompt_yesno(question)
}

pub fn notify_desktop<P: AsRef<str>>(message: P, timeout: Option<Duration>) {
    TERMINAL.lock().unwrap().notify_desktop(message, timeout);
}

pub fn display_time(display_time: bool) {
    TERMINAL.lock().unwrap().display_time(display_time);
}

pub fn set_show_step_ids(show: bool) {
    TERMINAL.lock().unwrap().set_show_step_ids(show);
}

pub fn set_current_step_id(step_id: Option<String>) {
    TERMINAL.lock().unwrap().set_current_step_id(step_id);
}

/// Parse a color name string into a console::Color.
pub fn parse_color(name: &str) -> Option<Color> {
    match name.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        _ => {
            // Try parsing as a 256-color index (e.g., "208" for orange)
            name.parse::<u8>().ok().map(Color::Color256)
        }
    }
}

pub fn set_separator_color(color: Option<Color>) {
    TERMINAL.lock().unwrap().set_separator_color(color);
}

/// Print a summary of all updated components collected during the run.
pub fn print_updated_components_summary(report: &[(impl AsRef<str>, StepResult)]) {
    let mut any_updates = false;

    for (key, result) in report {
        if let StepResult::Success(Some(updated)) = result
            && !updated.0.is_empty()
        {
            if !any_updates {
                any_updates = true;
                print_separator("Updated Components");
            }
            println!("{}:", key.as_ref());
            for component in &updated.0 {
                println!("  - {component}");
            }
        }
    }
}
