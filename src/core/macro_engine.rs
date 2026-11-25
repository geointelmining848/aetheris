use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::search::{SearchOptions, replace_all};
use crate::core::transforms::{TransformOp, apply_transform};

/// A single replayable step captured by the Aetheris Macro Recorder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroStep {
    Transform(TransformOp),
    ReplaceAll {
        query: String,
        replacement: String,
        case_sensitive: bool,
        whole_word: bool,
        use_regex: bool,
    },
    AppendText(String),
    PrependLine(String),
    AppendLine(String),
}

/// A named, persistable macro consisting of ordered steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedMacro {
    pub name: String,
    pub steps: Vec<MacroStep>,
}

/// Notepad++-style macro recorder, player, and on-disk macro library.
#[derive(Debug, Clone, Default)]
pub struct MacroRecorder {
    recording: bool,
    current_steps: Vec<MacroStep>,
    saved: Vec<SavedMacro>,
}

impl MacroRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn current_steps(&self) -> &[MacroStep] {
        &self.current_steps
    }

    pub fn saved_macros(&self) -> &[SavedMacro] {
        &self.saved
    }

    pub fn start_recording(&mut self) {
        self.recording = true;
        self.current_steps.clear();
    }

    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    pub fn record(&mut self, step: MacroStep) {
        if self.recording {
            self.current_steps.push(step);
        }
    }

    pub fn save_current(&mut self, name: impl Into<String>) -> bool {
        if self.current_steps.is_empty() {
            return false;
        }
        let name = name.into();
        self.saved.retain(|m| m.name != name);
        self.saved.push(SavedMacro {
            name,
            steps: self.current_steps.clone(),
        });
        true
    }

    /// Replay a sequence of macro steps `times` times on a text buffer.
    pub fn replay_steps(
        initial: &str,
        steps: &[MacroStep],
        times: usize,
    ) -> Result<String, String> {
        let mut current = initial.to_owned();
        for _ in 0..times.max(1) {
            for step in steps {
                current = apply_step(&current, step)?;
            }
        }
        Ok(current)
    }

    pub fn replay_current(&self, initial: &str, times: usize) -> Result<String, String> {
        if self.current_steps.is_empty() {
            return Err("No macro recorded yet".into());
        }
        Self::replay_steps(initial, &self.current_steps, times)
    }

    pub fn default_storage_path() -> PathBuf {
        dirs::data_local_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aetheris")
            .join("macros.json")
    }

    pub fn persist_to(&self, path: &PathBuf) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.saved)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn load_from(&mut self, path: &PathBuf) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(path)?;
        self.saved = serde_json::from_str(&raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(())
    }
}

fn apply_step(input: &str, step: &MacroStep) -> Result<String, String> {
    match step {
        MacroStep::Transform(op) => apply_transform(input, *op),
        MacroStep::ReplaceAll {
            query,
            replacement,
            case_sensitive,
            whole_word,
            use_regex,
        } => {
            let opts = SearchOptions {
                case_sensitive: *case_sensitive,
                whole_word: *whole_word,
                use_regex: *use_regex,
            };
            let (out, _) = replace_all(input, query, replacement, &opts)?;
            Ok(out)
        }
        MacroStep::AppendText(s) => Ok(format!("{input}{s}")),
        MacroStep::PrependLine(prefix) => Ok(input
            .lines()
            .map(|l| format!("{prefix}{l}"))
            .collect::<Vec<_>>()
            .join("\n")),
        MacroStep::AppendLine(suffix) => Ok(input
            .lines()
            .map(|l| format!("{l}{suffix}"))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_replays_and_persists_macros() {
        let mut rec = MacroRecorder::new();
        rec.start_recording();
        rec.record(MacroStep::Transform(TransformOp::TrimBothWhitespace));
        rec.record(MacroStep::Transform(TransformOp::RemoveEmptyLines));
        rec.record(MacroStep::Transform(TransformOp::SortLinesAsc));
        rec.record(MacroStep::Transform(TransformOp::DeduplicateLines));
        rec.stop_recording();

        let cleaned = rec
            .replay_current("  zeta \n\n  alpha  \nalpha\n  beta ", 1)
            .unwrap();
        assert_eq!(cleaned, "alpha\nbeta\nzeta");

        assert!(rec.save_current("Clean & Sort"));
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("macros.json");
        rec.persist_to(&file).unwrap();

        let mut loaded = MacroRecorder::new();
        loaded.load_from(&file).unwrap();
        assert_eq!(loaded.saved_macros().len(), 1);
        assert_eq!(loaded.saved_macros()[0].name, "Clean & Sort");
    }
}
