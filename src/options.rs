use std::{env, path::PathBuf};

pub struct Options {
    pub stow_dir: PathBuf,
    pub target_dir: PathBuf,
    pub special_keywords: bool,
    pub security_check: bool,
    pub verbose: bool,
    pub simulate: bool,
}

impl Options {
    pub fn default() -> Self {
        let working_dir = env::current_dir().expect("Working directory couldn't found.");
        let mut parent_dir = working_dir.clone();
        parent_dir.pop();

        Self {
            stow_dir: working_dir,
            target_dir: parent_dir,
            special_keywords: true,
            security_check: true,
            verbose: false,
            simulate: false,
        }
    }
}
