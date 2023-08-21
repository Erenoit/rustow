use std::{
    fs,
    io::{self, Result},
    path::{Path, PathBuf},
};

use crate::cmd::Args;

pub struct Stower {
    stow_dir:          PathBuf,
    target_dir:        PathBuf,
    simulate:          bool,
    verbose:           bool,
    no_special_paths:  bool,
    no_security_check: bool,
    stow:              Vec<PathBuf>,
    unstow:            Vec<PathBuf>,
    restow:            Vec<PathBuf>,
    adopt:             Vec<PathBuf>,
}

impl Stower {
    pub fn new(options: Args) -> Result<Self> {
        let full_stow_path = fs::canonicalize(&options.stow_dir)?;
        let full_target_path = fs::canonicalize(&options.target_dir)?;

        Ok(Self {
            stow_dir:          full_stow_path.clone(),
            target_dir:        full_target_path,
            simulate:          options.simulate,
            verbose:           options.verbose,
            no_special_paths:  options.no_special_paths,
            no_security_check: options.no_security_check,
            stow:              Self::ready_directories(full_stow_path.clone(), options.stow),
            unstow:            Self::ready_directories(full_stow_path.clone(), options.unstow),
            restow:            Self::ready_directories(full_stow_path.clone(), options.restow),
            adopt:             Self::ready_directories(full_stow_path, options.adopt),
        })
    }

    fn ready_directories(stow_dir: PathBuf, dirs: Vec<PathBuf>) -> Vec<PathBuf> {
        dirs.into_iter()
            .filter(|package| {
                // The only resons this may fail is given 'filesystem root', '.', or '..' and we
                // do not want them
                let Some(name) = package.file_name() else {
                    return false;
                };

                // Reasons to not include dot files is that they should be hidden and some files
                // like `.git` gets in the way when user does `rustow -S *`
                !name.to_string_lossy().starts_with('.')
            })
            .map(|package| stow_dir.join(package))
            .filter(|package| package.is_dir())
            .collect()
    }
}
