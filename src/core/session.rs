use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::encoding::{LineEnding, TextEncoding};
use crate::core::language::Language;

/// Serializable snapshot of one open tab in the zero-prompt Session Vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTab {
    pub id: u64,
    pub title: String,
    pub path: Option<PathBuf>,
    pub encoding: TextEncoding,
    pub line_ending: LineEnding,
    pub language: Language,
    pub is_dirty: bool,
    pub bookmarks: Vec<usize>,
    /// Backup filename inside the vault's `buffers/` directory (stored for unsaved or dirty tabs).
    pub backup_file: Option<String>,
}

/// Complete application session state restored automatically on startup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionState {
    pub active_tab_index: usize,
    pub workspace_root: Option<PathBuf>,
    pub theme_name: String,
    pub font_size: u16,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub tabs: Vec<SessionTab>,
}

/// Notepad++-style zero-prompt session vault storing unsaved scratchpads and tab metadata.
#[derive(Debug, Clone)]
pub struct SessionVault {
    root_dir: PathBuf,
}

impl SessionVault {
    pub fn default_location() -> Self {
        let base = dirs::data_local_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("aetheris").join("session"))
    }

    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn manifest_path(&self) -> PathBuf {
        self.root_dir.join("session.json")
    }

    fn buffers_dir(&self) -> PathBuf {
        self.root_dir.join("buffers")
    }

    /// Persist the session manifest and any unsaved/dirty buffer contents atomically.
    pub fn save(&self, state: &SessionState, buffers: &[(u64, String)]) -> io::Result<()> {
        fs::create_dir_all(self.buffers_dir())?;

        // Write active buffer snapshots
        let mut active_files = std::collections::HashSet::new();
        for (id, content) in buffers {
            let fname = format!("tab-{id}.txt");
            let path = self.buffers_dir().join(&fname);
            fs::write(path, content.as_bytes())?;
            active_files.insert(fname);
        }

        // Prune orphaned buffer backups from closed tabs
        if let Ok(entries) = fs::read_dir(self.buffers_dir()) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && !active_files.contains(name)
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let tmp = self.root_dir.join("session.json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(tmp, self.manifest_path())?;
        Ok(())
    }

    /// Load previously persisted session manifest, if present.
    pub fn load(&self) -> io::Result<Option<SessionState>> {
        let manifest = self.manifest_path();
        if !manifest.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(manifest)?;
        let state: SessionState = serde_json::from_str(&raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(state))
    }

    /// Load a backed-up unsaved buffer for a restored tab.
    pub fn load_buffer(&self, backup_file: &str) -> io::Result<String> {
        fs::read_to_string(self.buffers_dir().join(backup_file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_restores_unsaved_scratchpad_session() {
        let tmp = tempfile::tempdir().unwrap();
        let vault = SessionVault::new(tmp.path().join("vault"));

        let state = SessionState {
            active_tab_index: 0,
            workspace_root: Some(PathBuf::from("/tmp/project")),
            theme_name: "Aetheris Midnight".into(),
            font_size: 14,
            word_wrap: true,
            show_line_numbers: true,
            tabs: vec![SessionTab {
                id: 1,
                title: "Scratch 1".into(),
                path: None,
                encoding: TextEncoding::Utf8,
                line_ending: LineEnding::Lf,
                language: Language::Markdown,
                is_dirty: true,
                bookmarks: vec![0, 2],
                backup_file: Some("tab-1.txt".into()),
            }],
        };

        vault
            .save(&state, &[(1, "unsaved notes\nline 2\nline 3".into())])
            .unwrap();

        let restored = vault.load().unwrap().expect("session should exist");
        assert_eq!(restored, state);
        let text = vault.load_buffer("tab-1.txt").unwrap();
        assert_eq!(text, "unsaved notes\nline 2\nline 3");
    }
}
