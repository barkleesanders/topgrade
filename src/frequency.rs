//! Per-step update frequency tracking.
//!
//! Stores timestamps of the last run for each step in a JSON file at
//! `~/.local/state/topgrade/last_run.json` (XDG state dir on Unix,
//! config dir on Windows).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::debug;

use crate::config::StepFrequency;
use crate::step::Step;

/// Returns the path to the state file.
fn state_file_path() -> PathBuf {
    #[cfg(unix)]
    {
        let state_dir = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| crate::HOME_DIR.join(".local/state"));
        state_dir.join("topgrade/last_run.json")
    }

    #[cfg(windows)]
    {
        crate::WINDOWS_DIRS.data_local_dir().join("topgrade/last_run.json")
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Load the last-run timestamps from the state file.
fn load_state() -> HashMap<String, u64> {
    let path = state_file_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Save the last-run timestamps to the state file.
fn save_state(state: &HashMap<String, u64>) {
    let path = state_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        fs::write(&path, json).ok();
    }
}

/// Check if enough time has passed since the last run for the given step and frequency.
/// Returns `true` if the step should run.
pub fn should_run_by_frequency(step: Step, frequency: StepFrequency) -> bool {
    let interval = match frequency.interval_secs() {
        Some(i) => i,
        None => return true, // StepFrequency::Always
    };

    let state = load_state();
    let key = step.to_string();
    let now = now_secs();

    match state.get(&key) {
        Some(&last_run) => {
            let elapsed = now.saturating_sub(last_run);
            if elapsed >= interval {
                debug!("Step {key} last ran {elapsed}s ago (interval={interval}s), will run");
                true
            } else {
                debug!(
                    "Step {key} last ran {elapsed}s ago (interval={interval}s), skipping (next run in {}s)",
                    interval - elapsed
                );
                false
            }
        }
        None => {
            debug!("Step {key} has never run, will run");
            true
        }
    }
}

/// Record that a step just ran successfully.
pub fn record_step_run(step: Step) {
    let mut state = load_state();
    state.insert(step.to_string(), now_secs());
    save_state(&state);
}
