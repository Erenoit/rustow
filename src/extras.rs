use crate::options::Options;
use std::{
    borrow::Cow, fs, io::{self, Write, ErrorKind}, path::PathBuf,
    os::unix::{self, fs::MetadataExt}
};

// TODO: When "io_error_more" becomes stable chaeck for commented errors as well

#[inline(always)]
pub fn create_symlink(original: &PathBuf, destination: &PathBuf, options: &Options) -> bool {
    if let Err(why) = unix::fs::symlink(original, destination) {
        match why.kind() {
            ErrorKind::PermissionDenied => {
                println!("Couldn't create symlink for '{}'. Permission denied.", get_name(original));
            }
            ErrorKind::AlreadyExists => {
                println!("Couldn't create symlink for '{}'. Already exist.", get_name(original));
                unreachable_msg();
                unreachable!();
            }
            /*
            ErrorKind::StorageFull => {
                println!("Couldn't create symlink for '{}'. Storage full.", get_name(original));
                println!("Terminating...");
                process::exit(1);
            }
            ErrorKind::FilesystemQuotaExceeded => {
                println!("Couldn't create symlink for '{}'. Filesystem quota exceeded.", get_name(original));
                println!("Terminating...");
                process::exit(1);
            }
            ErrorKind::FileTooLarge => {
                println!("Couldn't create symlink for '{}'. File too large.", get_name(original));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::InvalidFilename => {
                println!("Couldn't create symlink for '{}'. Invalid filename.", get_name(original));
                unreachable_msg();
                unreachable!();
            }
            */
            _ => println!("Unknown error occured {:?}", why.kind()),
        }

        return false;
    }

    return true;
}

#[inline(always)]
pub fn remove_symlink(path: &PathBuf, options: &Options) -> bool {
    if let Err(why) = fs::remove_file(path) {
        match why.kind() {
            ErrorKind::PermissionDenied => {
                println!("Couldn't remove symlink '{}'. Permission denied.", get_name(path));
            }
            ErrorKind::NotFound => {
                println!("Couldn't remove symlink for '{}'. Not found.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            /*
            ErrorKind::IsADirectory => {
                println!("Couldn't remove symlink for '{}'. Is a directory.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::InvalidFilename => {
                println!("Couldn't remove symlink for '{}'. Invalid filename.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            */
            _ => println!("Unknown error occured {:?}", why.kind()),
        }

        return false;
    }

    return true;
}

#[inline(always)]
pub fn create_dir(path: &PathBuf, options: &Options) -> bool {
    if let Err(why) = fs::create_dir_all(path) {
        match why.kind() {
            ErrorKind::PermissionDenied => {
                println!("Couldn't create directory '{}'. Permission denied.", get_name(path));
            }
            ErrorKind::AlreadyExists => {
                println!("Couldn't create directory '{}'. Already exists.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            /*
            ErrorKind::StorageFull => {
                println!("Couldn't create directory '{}'. Storage full.", get_name(path));
                println!("Terminating...");
                process::exit(1);
            }
            ErrorKind::FilesystemQuotaExceeded => {
                println!("Couldn't create directory '{}'. Filesystem quota exceeded.", get_name(path));
                println!("Terminating...");
                process::exit(1);
            }
            ErrorKind::FileTooLarge => {
                println!("Couldn't create directory '{}'. File too large.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::InvalidFilename => {
                println!("Couldn't create directory '{}'. Invalid filename.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            */
            _ => println!("Unknown error occured {:?}", why.kind()),
        }

        return false;
    }

    return true;
}

#[inline(always)]
pub fn remove_dir(path: &PathBuf, options: &Options) -> bool {
    if let Err(why) = fs::remove_dir_all(path) {
        match why.kind() {
            ErrorKind::PermissionDenied => {
                println!("Couldn't remove directory '{}'. Permission denied.", get_name(path));
            }
            ErrorKind::NotFound => {
                println!("Couldn't remove directory '{}'. Not found.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            /*
            ErrorKind::NotADirectory => {
                println!("Couldn't remove directory '{}'. Not a directory.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::DirectoryNotEmpty => {
                println!("Couldn't remove directory '{}'. Directory not empty.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::InvalidFilename => {
                println!("Couldn't remove directory '{}'. Invalid filename.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            */
            _ => println!("Unknown error occured {:?}", why.kind()),
        }

        return false;
    }

    return true;
}

#[inline(always)]
pub fn remove_file(path: &PathBuf, options: &Options) -> bool {
    if let Err(why) = fs::remove_file(path) {
        match why.kind() {
            ErrorKind::PermissionDenied => {
                println!("Couldn't remove file '{}'. Permission denied.", get_name(path));
            }
            ErrorKind::NotFound => {
                println!("Couldn't remove file '{}'. Not found.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            /*
            ErrorKind::IsADirectory => {
                println!("Couldn't remove file '{}'. is a directory.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            ErrorKind::InvalidFilename => {
                println!("Couldn't remove file '{}'. Invalid filename.", get_name(path));
                unreachable_msg();
                unreachable!();
            }
            */
            _ => println!("Unknown error occured {:?}", why.kind()),
        }

        return false;
    }

    return true;
}

#[inline(always)]
fn unreachable_msg() {
    println!(r#"
Version: 0.2-beta
If you saw this message please report to 'https://gitlab.com/Erenoit/rustow' with:");
    1. This whole error message
    2. File structure you tried to run this program
    3. Additional informations may be helpful
"#);
}

/*
 * Returns file/directory name as str
 * if path is "/", it returns "filesystem root"
 */
#[inline(always)]
pub fn get_name(path: &PathBuf) -> Cow<'_, str> {
    if let Some(name) = path.file_name() {
        name.to_string_lossy()
    } else {
        Cow::from("filesystem root")
    }
}

/*
 * Writes the message to stdout and takes input from user
 * If the input can be interpreted as "Yes", returns true
 * otherwise returns false
 */
#[inline(always)]
pub fn prompt(message: String, is_yes_default: bool) -> bool {
    let yes_no_prompt = if is_yes_default { "(Y/n)" } else { "(y/N)" };
    print!("{message} {yes_no_prompt}: ");
    io::stdout().flush().expect("Failed to print.");

    let mut buffer = String::new();
    if let Err(_e) = io::stdin().read_line(&mut buffer) {
        println!("An error accured while taking input. Program will continue with \"No\" option.");
        return false;
    }
    let answer = buffer.trim().to_lowercase();

    return (is_yes_default && answer == "") || answer == "y" || answer == "yes";
}

/*
 * Check if file/folder belongs to root
 * On error return false
 */
#[inline(always)]
pub fn is_root_file(path: &PathBuf) -> bool {
    match dbg!(path).metadata() {
        Ok(metadata) => {
            if dbg!(metadata.uid()) == 0 {
                // TODO: Also check for child files and write permissions
                return true;
            } else {
                return false;
            }
        }
        Err(why) => {
            dbg!("failed in metadata {}", why);
            return false;
        }
    }
}

