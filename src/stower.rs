use std::{
    env,
    fs,
    io::{self, Result},
    os::unix::{self, fs::MetadataExt},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::cmd::Args;

// TODO: make simulate keep trck of changes so it will generate more realistic
// simulation

macro_rules! print_verbose {
    ($self:ident, $($arg:tt)*) => {
        if $self.verbose || $self.simulate {
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
    #[allow(dead_code)]
    stow_dir:       PathBuf,
    target_dir:     PathBuf,
    simulate:       bool,
    verbose:        bool,
    special_paths:  bool,
    security_check: bool,
    replace_name:   Option<(String, String)>,
    stow:           Vec<PathBuf>,
    unstow:         Vec<PathBuf>,
    restow:         Vec<PathBuf>,
    adopt:          Vec<PathBuf>,
}

impl Stower {
    pub fn new(options: Args) -> Result<Self> {
        let full_stow_path = fs::canonicalize(&options.stow_dir)?;
        let full_target_path = fs::canonicalize(&options.target_dir)?;

        Ok(Self {
            stow_dir:       full_stow_path.clone(),
            target_dir:     full_target_path,
            simulate:       options.simulate,
            verbose:        options.verbose,
            special_paths:  !options.no_special_paths,
            security_check: !options.no_security_check,
            replace_name:   if options.replace_name.is_empty() {
                None
            } else {
                Some((
                    options.replace_name[0].clone(),
                    options.replace_name[1].clone(),
                ))
            },
            stow:           Self::ready_directories(full_stow_path.clone(), options.stow),
            unstow:         Self::ready_directories(full_stow_path.clone(), options.unstow),
            restow:         Self::ready_directories(full_stow_path.clone(), options.restow),
            adopt:          Self::ready_directories(full_stow_path, options.adopt),
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
                &self.target_dir,
                Self::unstow,
                Some(Self::unstow_extra),
                self.special_paths,
            )
            .ok();
        });

        self.restow.iter().for_each(|package| {
            self.handle_directory(
                package,
                &self.target_dir,
                Self::unstow,
                Some(Self::unstow_extra),
                self.special_paths,
            )
            .ok();
            self.handle_directory(
                package,
                &self.target_dir,
                Self::stow,
                None,
                self.special_paths,
            )
            .ok();
        });

        self.stow.iter().for_each(|package| {
            self.handle_directory(
                package,
                &self.target_dir,
                Self::stow,
                None,
                self.special_paths,
            )
            .ok();
        });

        self.adopt.iter().for_each(|package| {
            self.handle_directory(
                package,
                &self.target_dir,
                Self::adopt,
                None,
                self.special_paths,
            )
            .ok();
            self.handle_directory(
                package,
                &self.target_dir,
                Self::stow,
                None,
                self.special_paths,
            )
            .ok();
        });
    }

    fn handle_directory(
        &self,
        directory: &Path,
        destination: &Path,
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

        let mut new_destination = destination.to_path_buf();
        subdirs.filter_map(|e| e.ok()).for_each(|element| {
            new_destination.push(element.file_name());
            action_func(
                self,
                &element.path(),
                &new_destination,
                use_special_paths,
            )
            .ok();
            new_destination.pop();
        });

        if let Some(extra) = extra_func {
            extra(self, destination).ok();
        }

        Ok(())
    }

    fn stow(&self, original: &Path, destination: &Path, use_special_paths: bool) -> Result<()> {
        let Some(destination) =
            self.handle_destination(original, destination, use_special_paths)?
        else {
            return Ok(());
        };

        let Some(file_name) = original.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file name",
            ));
        };

        if original.is_symlink() {
            print_verbose!(
                self,
                "{} is symlink. Skipping...",
                file_name.to_string_lossy()
            );

            Ok(())
        } else if destination.is_symlink() {
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

                    self.handle_directory(&real_dest, &destination, Self::stow, None, false)?;
                    self.handle_directory(original, &destination, Self::stow, None, false)
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
                self.handle_directory(original, &destination, Self::stow, None, false)
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
        let Some(destination) =
            self.handle_destination(original, destination, use_special_paths)?
        else {
            return Ok(());
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

        if original.is_symlink() {
            print_verbose!(
                self,
                "{} is symlink. Skipping...",
                file_name.to_string_lossy()
            );

            Ok(())
        } else if destination.is_symlink() {
            self.remove_symlink(&destination)
        } else if destination.is_dir() && original.is_dir() {
            self.handle_directory(
                original,
                &destination,
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
        let Some(destination) =
            self.handle_destination(original, destination, use_special_paths)?
        else {
            return Ok(());
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
            self.handle_directory(original, &destination, Self::adopt, None, false)
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
        print_verbose!(
            self,
            "Creating symlink: {} -> {}",
            destination.to_string_lossy(),
            original.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        unix::fs::symlink(original, destination)
    }

    fn remove_symlink(&self, target: &Path) -> Result<()> {
        print_verbose!(
            self,
            "Removing symlink: {}",
            target.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        fs::remove_file(target)
    }

    fn create_dir(&self, target: &Path) -> Result<()> {
        print_verbose!(
            self,
            "Creating directory: {}",
            target.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        fs::create_dir_all(target)
    }

    fn remove_dir(&self, target: &Path) -> Result<()> {
        print_verbose!(
            self,
            "Removing directory: {}",
            target.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        fs::remove_dir_all(target)
    }

    fn remove_file(&self, target: &Path) -> Result<()> {
        print_verbose!(
            self,
            "Removing file: {}",
            target.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        fs::remove_file(target)
    }

    fn move_file(&self, original: &Path, destination: &Path) -> Result<()> {
        print_verbose!(
            self,
            "Moving file: {} -> {}",
            original.to_string_lossy(),
            destination.to_string_lossy()
        );

        if self.simulate {
            return Ok(());
        }

        if fs::rename(original, destination).is_err() {
            fs::copy(original, destination)?;
            fs::remove_file(original)?;
        }

        Ok(())
    }

    fn handle_special_paths(&self, original: &Path, destination: &Path) -> Option<PathBuf> {
        if !self.special_paths {
            return Some(destination.to_path_buf());
        }

        let Some(file_name) = original.file_name() else {
            return None;
        };

        match file_name.to_string_lossy().as_ref() {
            "@home" => {
                let Ok(home_path) = env::var("HOME") else {
                    println!("Couldn't find HOME variable.");
                    return None;
                };

                let home_path = PathBuf::from(home_path);

                if home_path.exists() {
                    Some(home_path)
                } else {
                    Some(destination.to_path_buf())
                }
            },
            "@root" =>
                if self.is_root_user_file(original) {
                    Some(PathBuf::from("/"))
                } else {
                    println!(
                        r#"
For security reasons, all the files/folders including and followed by @root file must be owned by root.
This requred to prevent giving others access to important system files by mistake.
(Because the owner of the file will be the user who runs Rustow)
Also creating a symlink to a path followed by @root generally needs root access anyway.
If you want to stow something inside your home folder, use @home instead.
Use --no-security-check flag to prevent from this error
"#
                    );

                    None
                },
            _ => Some(destination.to_path_buf()),
        }
    }

    fn is_root_user_file(&self, path: &Path) -> bool {
        if !self.security_check {
            return true;
        }

        // FIXME: we may not get the metadata because we do not have enough permmissions
        match path.metadata() {
            Ok(metadata) =>
                if metadata.uid() == 0 {
                    if path.is_dir() {
                        let subdirs = fs::read_dir(path);
                        if subdirs.is_err() {
                            return false;
                        }

                        // FIXME: we may not read because we do not have enough permmissions
                        subdirs
                            .unwrap()
                            .filter_map(|e| e.ok())
                            .map(|element| self.is_root_user_file(&element.path()))
                            .all(|x| x)
                    } else {
                        true
                    }
                } else {
                    false
                },
            Err(why) => {
                dbg!("failed in metadata {}", why);
                false
            },
        }
    }

    fn handle_destination(
        &self,
        original: &Path,
        destination: &Path,
        use_special_paths: bool,
    ) -> Result<Option<PathBuf>> {
        let mut destination = if use_special_paths {
            if let Some(path) = self.handle_special_paths(original, destination) {
                path
            } else {
                print_verbose!(
                    self,
                    "Got error while handling special paths for {}. Skipping...",
                    original.display()
                );
                return Ok(None);
            }
        } else {
            destination.to_path_buf()
        };

        if let Some((ref find, ref relpace)) = self.replace_name {
            let Ok(re) = Regex::new(find) else {
                print_verbose!(self, "Invalid regex: {}", find);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid regex",
                ));
            };

            let name = destination
                .file_name()
                .expect("Cannot fail")
                .to_string_lossy()
                .to_string();

            let new_name = re.replace_all(&name, relpace.as_str());

            destination.set_file_name(new_name.to_string());
        }

        Ok(Some(destination))
    }
}
