use std::{
    fs,
    io::{self, Result},
    path::{Path, PathBuf},
};

use crate::cmd::Args;

macro_rules! print_verbose {
    ($self:ident, $($arg:tt)*) => {
        if $self.verbose {
            println!($($arg)*);
        }
    };
}

macro_rules! prompt {
    ($self:ident, $default:expr, $($arg:tt)*) => {
        {
            use std::io::{self, Write};

            print!("{} {}: ", format!($($arg)*), if $default { "[Y/n]" } else { "[y/N]" });
            io::stdout().flush().expect("Failed to print.");

            let mut buffer = String::new();
            if let Err(_e) = io::stdin().read_line(&mut buffer) {
                println!("An error accured while taking input. Program will continue with \"No\" option.");
                false
            } else {
                let answer = buffer.trim().to_lowercase();

                ($default && answer.is_empty()) || answer == "y" || answer == "yes"
            }
        }
    };
}

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

    pub fn run(self) {
        self.unstow.iter().for_each(|package| {
            self.handle_directory(
                package,
                Self::unstow,
                Some(Self::unstow_extra),
                !self.no_special_paths,
            )
            .ok();
        });

        self.restow.iter().for_each(|package| {
            self.handle_directory(
                package,
                Self::unstow,
                Some(Self::unstow_extra),
                !self.no_special_paths,
            )
            .ok();
            self.handle_directory(package, Self::stow, None, !self.no_special_paths)
                .ok();
        });

        self.stow.iter().for_each(|package| {
            self.handle_directory(package, Self::stow, None, !self.no_special_paths)
                .ok();
        });

        self.adopt.iter().for_each(|package| {
            self.handle_directory(package, Self::adopt, None, !self.no_special_paths)
                .ok();
            self.handle_directory(package, Self::stow, None, !self.no_special_paths)
                .ok();
        });
    }

    fn handle_directory(
        &self,
        directory: &Path,
        action_func: fn(&Self, &Path, &Path, bool) -> Result<()>,
        extra_func: Option<fn(&Self, &Path) -> Result<()>>,
        use_special_paths: bool,
    ) -> Result<()> {
        let Some(_folder_name) = directory.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid directory name",
            ));
        };

        let subdirs = fs::read_dir(directory)?;

        let mut destination = self.target_dir.clone();
        subdirs.filter_map(|e| e.ok()).for_each(|element| {
            destination.push(element.file_name());
            action_func(
                self,
                &element.path(),
                &destination,
                use_special_paths,
            )
            .ok();
            destination.pop();
        });

        if let Some(extra) = extra_func {
            extra(self, &destination).ok();
        }

        Ok(())
    }

    fn stow(&self, original: &Path, destination: &Path, use_special_paths: bool) -> Result<()> {
        let destination = if use_special_paths {
            self.handle_special_paths(original, destination)
        } else {
            destination.to_path_buf()
        };

        let Some(file_name) = original.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file name",
            ));
        };

        if destination.is_symlink() {
            if destination.is_dir() {
                if let Ok(real_dest) = fs::canonicalize(&destination) {
                    if real_dest == original {
                        print_verbose!(
                            self,
                            "{} is already stowed. Skipping...",
                            file_name.to_string_lossy()
                        );
                        return Ok(());
                    }

                    self.remove_symlink(&destination)?;
                    self.create_dir(&destination)?;

                    self.handle_directory(&real_dest, Self::stow, None, false)?;
                    self.handle_directory(original, Self::stow, None, false)
                } else {
                    let is_accepted = prompt!(
                        self,
                        false,
                        "There is an invalid symlink on {}. Would you like to delete it and replace with new symlink",
                        file_name.to_string_lossy()
                        );

                    if is_accepted {
                        self.remove_symlink(&destination)?;
                        self.create_symlink(original, &destination)
                    } else {
                        print_verbose!(
                            self,
                            "{} is already stowed. Skipping...",
                            file_name.to_string_lossy()
                        );
                        Ok(())
                    }
                }
            } else {
                print_verbose!(
                    self,
                    "{} is already symlink. Skipping...",
                    file_name.to_string_lossy()
                );

                Ok(())
            }
        } else if destination.exists() {
            if destination.is_dir() {
                self.handle_directory(original, Self::stow, None, false)
            } else {
                let is_accepted = prompt!(
                    self,
                    false,
                    "{} already exists, would you like to delete it and replace with symlink",
                    file_name.to_string_lossy()
                );

                if is_accepted {
                    self.remove_file(&destination)?;
                    self.create_symlink(original, &destination)?;
                }

                Ok(())
            }
        } else {
            self.create_symlink(original, &destination)
        }
    }

    fn unstow(&self, original: &Path, destination: &Path, use_special_paths: bool) -> Result<()> {
        let destination = if use_special_paths {
            self.handle_special_paths(original, destination)
        } else {
            destination.to_path_buf()
        };

        let Some(file_name) = original.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file name",
            ));
        };

        if !destination.exists() {
            print_verbose!(
                self,
                "{} does not exist. Skipping...",
                destination.display()
            );
            return Ok(());
        }

        if destination.is_symlink() {
            self.remove_symlink(&destination)
        } else if destination.is_dir() && original.is_dir() {
            self.handle_directory(
                original,
                Self::unstow,
                Some(Self::unstow_extra),
                false, // specail paths only works in first level
            )
        } else {
            print_verbose!(
                self,
                "{} exists but it is not a symlink. Skipping...",
                file_name.to_string_lossy()
            );

            Ok(())
        }
    }

    fn adopt(&self, original: &Path, destination: &Path, use_special_paths: bool) -> Result<()> {
        let destination = if use_special_paths {
            self.handle_special_paths(original, destination)
        } else {
            destination.to_path_buf()
        };

        let Some(file_name) = original.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file name",
            ));
        };

        if !destination.exists() {
            print_verbose!(
                self,
                "{} does not exist. Skipping...",
                destination.display()
            );
            return Ok(());
        }

        if destination.is_symlink() {
            print_verbose!(
                self,
                "{} is already symlink. Skipping...",
                destination.display()
            );

            Ok(())
        } else if original.is_symlink() {
            print_verbose!(
                self,
                "{} is symlink but destination is not. Skipping...",
                file_name.to_string_lossy()
            );

            Ok(())
        } else if destination.is_dir() && original.is_dir() {
            self.handle_directory(original, Self::adopt, None, false)
        } else if destination.is_file() && original.is_file() {
            let mut backup_path = PathBuf::from("/tmp/rustow-backup");
            self.create_dir(&backup_path)?;

            backup_path.push(file_name);
            self.move_file(original, &backup_path)?;
            if self.move_file(&destination, original).is_ok() {
                self.remove_file(&backup_path)
            } else {
                self.move_file(&backup_path, original)
            }
        } else {
            print_verbose!(self, "Original and target are not same type (one is file but other is directory). Skipping...");
            Ok(())
        }
    }

    fn unstow_extra(&self, target: &Path) -> Result<()> {
        let mut dir_items = fs::read_dir(target)?;
        if dir_items.next().is_none() {
            self.remove_dir(target)?;
        }

        Ok(())
    }

    fn create_symlink(&self, original: &Path, destination: &Path) -> Result<()> {
        todo!("Create Symlink --------------------------------------------------------");
    }

    fn remove_symlink(&self, target: &Path) -> Result<()> {
        todo!("Remove Symlink --------------------------------------------------------");
    }

    fn create_dir(&self, target: &Path) -> Result<()> {
        todo!("Create Dir ------------------------------------------------------------");
    }

    fn remove_dir(&self, target: &Path) -> Result<()> {
        todo!("Remove Dir ------------------------------------------------------------");
    }

    fn remove_file(&self, target: &Path) -> Result<()> {
        todo!("Remove File -----------------------------------------------------------");
    }

    fn move_file(&self, original: &Path, destination: &Path) -> Result<()> {
        todo!("Move File -------------------------------------------------------------");
    }

    fn handle_special_paths(&self, original: &Path, destination: &Path) -> PathBuf {
        todo!("Handle Special Path ----------------------------------------------------");
    }
}
