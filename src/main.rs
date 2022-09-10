#[cfg(test)]
mod test;

use std::{borrow::Cow, env, fs::{self, DirEntry}, io::{self, Write}, path::PathBuf, process, os::unix};

fn main() {
    let mut stow_dir = env::current_dir().expect("Working directory couldn't found.");
    let mut target_dir = stow_dir.clone();
    target_dir.pop();

    let (stow_l, unstow_l, restow_l, adopt_l) = handle_cmd_arguments(&mut stow_dir, &mut target_dir);

    if unstow_l.len() > 0 {
        for directory in unstow_l {
            unstow_all_inside_dir(&directory, &target_dir);
        }
    }

    if restow_l.len() > 0 {
        for directory in restow_l {
            unstow_all_inside_dir(&directory, &target_dir);
            stow_all_inside_dir(&directory, &target_dir);
        }
    }

    if stow_l.len() > 0 {
        for directory in stow_l {
            stow_all_inside_dir(&directory, &target_dir);
        }
    }

    if adopt_l.len() > 0 {
        println!("Adopt functionality is not implemented yet. Skipping...");
    }
}

/*
 * Takes command line arguments and creates Vec for each possible action (stow, unstow, restow,
 * adopt).
 * if there is an invalid flag, prompts error and exits
 * if there is an invalid argument (one doesn't match with a directory name) prompts error
 * and asks if user wants to comtinue without that argument
 */
fn handle_cmd_arguments(stow_dir: &mut PathBuf, target_dir: &mut PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut str_stow   = Vec::<String>::with_capacity(10);
    let mut str_unstow = Vec::<String>::with_capacity(10);
    let mut str_restow = Vec::<String>::with_capacity(10);
    let mut str_adopt  = Vec::<String>::with_capacity(10);

    let mut push_mode = PushMode::Stow;

    let mut args = env::args();
    args.next();             // First argument is executable name

    'args_loop: loop {
        let argument = match args.next() {
            Some(a) => a,
            None => break 'args_loop
        };

        if argument.starts_with("-") {
            match &argument[..] {
                "-S" => push_mode = PushMode::Stow,
                "-D" => push_mode = PushMode::Unstow,
                "-R" => push_mode = PushMode::Restow,
                "-A" => push_mode = PushMode::Adopt,
                "-h" | "--help" => {
                    print_help();
                    process::exit(0);
                }
                "-d" | "--stow-dir" => {
                    let new_dir = match args.next() {
                        Some(a) => PathBuf::from(a),
                        None => {
                            println!("--stow-dir option requires an argument\n");
                            process::exit(1);
                        }
                    };

                    if !new_dir.exists() || !new_dir.is_dir() {
                        println!("{} is not a valid directory in your file system.", new_dir.to_string_lossy());
                        process::exit(1);
                    }

                    *stow_dir = new_dir;
                }
                "-t" | "--target-dir" => {
                    let new_dir = match args.next() {
                        Some(a) => PathBuf::from(a),
                        None => {
                            println!("--target-dir option requires an argument\n");
                            process::exit(1);
                        }
                    };

                    if !new_dir.exists() || !new_dir.is_dir() {
                        println!("{} is not a valid directory in your file system.", new_dir.to_string_lossy());
                        process::exit(1);
                    }

                    *target_dir = new_dir;
                }
                _ => {
                    println!("Unknown argument: {argument}.");
                    println!("Use `rustow -h` for available arguments.");
                    process::exit(1);
                }
            }
        } else {
            match push_mode {
                PushMode::Stow   => str_stow.push(argument),
                PushMode::Unstow => str_unstow.push(argument),
                PushMode::Restow => str_restow.push(argument),
                PushMode::Adopt  => str_adopt.push(argument),
            }
        }
    }

    let mut stow_l   = Vec::<PathBuf>::with_capacity(str_stow.len());
    let mut unstow_l = Vec::<PathBuf>::with_capacity(str_unstow.len());
    let mut restow_l = Vec::<PathBuf>::with_capacity(str_restow.len());
    let mut adopt_l  = Vec::<PathBuf>::with_capacity(str_adopt.len());

    let directories = fs::read_dir(&stow_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .filter(|e| {                     // Only take directories
            let ftype = e.file_type();
            if ftype.is_err() { return false; }

            ftype.unwrap().is_dir()
        })
        .filter(|e| {                     // Remove files starts with dot
            !e.file_name().to_string_lossy().starts_with(".")
        })
        .collect::<Vec<_>>();

        validate_directories(&str_stow,   &mut stow_l,   &directories);
        validate_directories(&str_unstow, &mut unstow_l, &directories);
        validate_directories(&str_restow, &mut restow_l, &directories);
        validate_directories(&str_adopt,  &mut adopt_l,  &directories);

    return (stow_l, unstow_l, restow_l, adopt_l)  ;
}

/*
 * Takes valid directories as one of its parameters
 * Checks every string in str_vec with names of every element in valid_directories
 * If mathes, adds whole path to target_vec
 * If does not match at all, asks user if they want to continue without that argument
 * If user says no, terminates the program
 */
fn validate_directories(str_vec: &Vec<String>, target_vec: &mut Vec<PathBuf>, valid_directories: &Vec<DirEntry>) {
    for arg in str_vec {
        let mut is_valid = false;
        for e in valid_directories {
            if e.file_name().to_string_lossy() == arg[..] {
                is_valid = true;
                target_vec.push(e.path());
            }
        }

        if !is_valid {
            println!("Invalid argument: {arg}.");
            let is_accepted = prompt("Do you want to continue without this argument".to_string(), true);

            if !is_accepted {
                process::exit(1);
            }
        }
    }
}

/*
 * if there is already a symlink of the file, skips the file
 * if there is already a symlink of a directory, delete the symlink, create a real directory,
 * stow everything old symlink has inside directory, and stow new things inside directory
 * if there is already a directory, tries to stow things inside the folder
 * if there is already a file, asks the user if user wants to remove existing one and stow or cancel
 * otherwise creates symlink
 */
fn stow(original: &PathBuf, destination: &PathBuf) {
    let fname = get_name(destination);

    if destination.is_symlink() {
        if destination.is_dir() {
            let real_dest = fs::canonicalize(destination);
            if real_dest.is_ok(){
                _ = fs::remove_file(destination);
                _ = fs::create_dir(destination);

                stow_all_inside_dir(&real_dest.unwrap(), destination);
                stow_all_inside_dir(original, destination);
            } else {
                let is_accepted = prompt(
                    format!("There is an invalid symlink on {fname}. Would you like to delete it and replace with new symlink"),
                    false);

                if is_accepted {
                    _ = fs::remove_file(&destination);
                    _ = unix::fs::symlink(original, destination);
                }
            }
        } else {
            println!("{fname} is already stowed. Skipping...");
        }
    } else if destination.exists() {
        if destination.is_dir() {
            stow_all_inside_dir(original, destination);
        } else {
            let is_accepted = prompt(
                format!("{fname} already exists, would you like to delete it and replace with symlink"),
                false);

            if is_accepted {
                _ = fs::remove_file(&destination);
                _ = unix::fs::symlink(original, destination);
            }
        }
    } else {
        _ = unix::fs::symlink(original, destination);
    }
}

/*
 * iterates over everything inside a directory and stows it
 */
fn stow_all_inside_dir(original: &PathBuf, destination: &PathBuf) {
    let fname = get_name(destination);

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        println!("{fname} couldn't be read. Skipping...");
        return;
    }

    let mut write_location = destination.clone();
    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            write_location.push(element.file_name());
            stow(&element.path(), &write_location);
            write_location.pop();
        });
}

/*
 * if there is symlink, removes it
 * if there is a directory, try to unstow things inside the folder
 * if thete is a file, prompts error and skips
 */
fn unstow(original: &PathBuf, target: &PathBuf) {
    let fname = get_name(target);

    if !target.exists() {
        println!("{fname} does not exists. Nothing to unstow.");
        return;
    }

    if target.is_symlink() {
        _ = fs::remove_file(target);
    } else {
        if target.is_dir() && original.is_dir() {
            unstow_all_inside_dir(original, target);
        } else {
            println!("{fname} exists but it is not a symlink. Skipping...");
        }
    }
}

/*
 * iterates over everything inside a directory and unstows it
 * if the directory becomes empty, deletes the directory
 */
fn unstow_all_inside_dir(original: &PathBuf, target: &PathBuf) {
    let fname = get_name(target);

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        println!("{fname} couldn't be read. Skipping...");
        return;
    }

    let mut write_location = target.clone();
    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            write_location.push(element.file_name());
            unstow(&element.path(), &write_location);
            write_location.pop();
        });

    if let Ok(mut dir_iterator) = fs::read_dir(target) {
        if dir_iterator.next().is_none() {
            _ = fs::remove_dir(target);
        }
    }
}

/*
 * Prints the help message
 */
fn print_help() {
    let help_str = concat!(
        "rustow version 0.1", "\n",
        "\n",
        "Usage:", "\n",
        "    rustow [OPTION ...] [-S|-D|-R|-A] PACKAGE ... [-S|-D|-R|-A] PACKAGE ...", "\n",
        "\n",
        "Actions:", "\n",
        "    -S  Stow the package.", "\n",
        "        Creates symlinks of files in the package to target directory", "\n",
        "    -D  Unstow the package.", "\n",
        "        Removes existing symlinks of files in the package in target directory", "\n",
        "    -R  Restow the package.", "\n",
        "        Same as unstowing and stowing a package", "\n",
        "    -A  Adopt the package.", "\n",
        "        Imports existing files in target directory to stow package. USE WITH CAUTION!", "\n",
        "\n",
        "Options:", "\n",
        "    -h      --help            Prints this help message", "\n",
        "    -d DIR  --stow-dir DIR    Set stow dir to DIR (default is current dir)", "\n",
        "    -t DIR  --target-dir DIR  Set target dir to DIR (default is parent of stow dir)", "\n",
        );

    println!("{help_str}");
}

/*
 * Returns file/directory name as str
 * if path is "/", it returns "filesystem root"
 */
fn get_name(path: &PathBuf) -> Cow<'_, str> {
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
fn prompt(message: String, is_yes_default: bool) -> bool {
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

enum PushMode {
    Stow,
    Unstow,
    Restow,
    Adopt,
}

