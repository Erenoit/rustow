mod extras;
mod options;
#[cfg(test)]
mod test;

use crate::extras::*;
use crate::options::Options;
use std::{env, fs::{self, DirEntry}, path::PathBuf, process};

// TODO: make --simulate keeo trck of changes so it will generate more real outcome

fn main() {
    let mut options = Options::default();
    let (stow_l, unstow_l, restow_l, adopt_l) = handle_cmd_arguments(&mut options);

    if unstow_l.len() > 0 {
        for directory in unstow_l {
            unstow_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
        }
    }

    if restow_l.len() > 0 {
        for directory in restow_l {
            unstow_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
            stow_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
        }
    }

    if stow_l.len() > 0 {
        for directory in stow_l {
            stow_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
        }
    }

    if adopt_l.len() > 0 {
        for directory in adopt_l {
            adopt_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
            stow_all_inside_dir(&directory, &options.target_dir, options.special_keywords, &options);
        }
    }
}

/*
 * Takes command line arguments and creates Vec for each possible action (stow, unstow, restow,
 * adopt).
 * if there is an invalid flag, prompts error and exits
 * if there is an invalid argument (one doesn't match with a directory name) prompts error
 * and asks if user wants to comtinue without that argument
 */
fn handle_cmd_arguments(options: &mut Options) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
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
                "-V" | "--version" => {
                    print_version();
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

                    options.stow_dir = new_dir;
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

                    options.target_dir = new_dir;
                }
                "-v" | "--verbose" => {
                    options.verbose = true;
                }
                "-s" | "--simulate" => {
                    options.simulate = true;
                    /* If it does not print anythink, there is no purpose for simulating */
                    options.verbose = true;
                }
                "--no-special-keywords" => {
                    options.special_keywords = false;
                }
                "--no-security-check" => {
                    options.security_check = false;
                    println!("--no-security-check flag is not implemented yet. It will have no effect.");
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

    let directories = fs::read_dir(&options.stow_dir)
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
fn stow(original: &PathBuf, destination: &PathBuf, use_special_paths: bool, options: &Options) {
    let new_dest = if use_special_paths { handle_special_path(original, destination, options) }
                   else { destination.clone() };

    let fname = get_name(&new_dest);

    if new_dest.is_symlink() {
        if new_dest.is_dir() {
            if let Ok(real_dest) = fs::canonicalize(&new_dest) {
                if &real_dest == original {
                    if options.verbose {
                        println!("{fname} is already stowed. Skipping...");
                    }
                    return;
                }
                remove_symlink(&new_dest, options);
                create_dir(&new_dest, options);

                stow_all_inside_dir(&real_dest, &new_dest, false, options);
                stow_all_inside_dir(original, &new_dest, false, options);
            } else {
                let is_accepted = prompt(
                    format!("There is an invalid symlink on {fname}. Would you like to delete it and replace with new symlink"),
                    false);

                if is_accepted {
                    remove_symlink(&new_dest, options);
                    create_symlink(original, &new_dest, options);
                } else if options.verbose {
                    println!("{fname} is already stowed. Skipping...");
                }
            }
        } else if options.verbose {
            println!("{fname} is already stowed. Skipping...");
        }
    } else if new_dest.exists() {
        if new_dest.is_dir() {
            stow_all_inside_dir(original, &new_dest, false, options);
        } else {
            let is_accepted = prompt(
                format!("{fname} already exists, would you like to delete it and replace with symlink"),
                false);

            if is_accepted {
                remove_file(&new_dest, options);
                create_symlink(original, &new_dest, options);
            }
        }
    } else {
        create_symlink(original, &new_dest, options);
    }
}

/*
 * iterates over everything inside a directory and stows it
 */
fn stow_all_inside_dir(original: &PathBuf, destination: &PathBuf, use_special_paths: bool, options: &Options) {
    let fname = get_name(destination);

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        if options.verbose {
            println!("{fname} couldn't be read. Skipping...");
        }
        return;
    }

    let mut write_location = destination.clone();
    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            write_location.push(element.file_name());
            stow(&element.path(), &write_location, use_special_paths, options);
            write_location.pop();
        });
}

/*
 * if there is symlink, removes it
 * if there is a directory, try to unstow things inside the folder
 * if thete is a file, prompts error and skips
 */
fn unstow(original: &PathBuf, target: &PathBuf, use_special_paths: bool, options: &Options) {
    let new_target = if use_special_paths { handle_special_path(original, target, options) }
                   else { target.clone() };

    let fname = get_name(&new_target);

    if !new_target.exists() {
        if options.verbose {
            println!("{fname} does not exists. Nothing to unstow.");
        }
        return;
    }

    if new_target.is_symlink() {
        remove_symlink(&new_target, options);
    } else {
        if new_target.is_dir() && original.is_dir() {
            unstow_all_inside_dir(original, &new_target, false, options);
        } else if options.verbose {
            println!("{fname} exists but it is not a symlink. Skipping...");
        }
    }
}

/*
 * iterates over everything inside a directory and unstows it
 * if the directory becomes empty, deletes the directory
 */
fn unstow_all_inside_dir(original: &PathBuf, target: &PathBuf, use_special_paths: bool, options: &Options) {
    let fname = get_name(target);

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        if options.verbose {
            println!("{fname} couldn't be read. Skipping...");
        }
        return;
    }

    let mut write_location = target.clone();
    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            write_location.push(element.file_name());
            unstow(&element.path(), &write_location, use_special_paths, options);
            write_location.pop();
        });

    if let Ok(mut dir_iterator) = fs::read_dir(target) {
        if dir_iterator.next().is_none() {
            remove_dir(target, options);
        }
    }
}

/*
 * if path points to a file, adopts it
 * if path points to a file, sends back to adopt_all_inside_dir() to adopt everything insdide the
 * file
 * even though original and target variables should be reversed for logical reasons, it kept same
 * for consistency
 */
fn adopt(original: &PathBuf, target: &PathBuf, use_special_paths: bool, options: &Options) {
    let new_target = if use_special_paths { handle_special_path(original, target, options) }
                   else { target.clone() };

    let fname = get_name(&new_target);

    if !new_target.exists() {
        if options.verbose {
            println!("{fname} does not exists. Nothing to adopt.");
        }
        return;
    }

    if new_target.is_dir() && target.is_dir() {
        adopt_all_inside_dir(original, &new_target, false, options);
    } else if new_target.is_file() && target.is_file() {
        let mut backup_path = original.clone();
        backup_path.pop();
        backup_path.push(format!("{fname}_backup"));

        move_file(original, &backup_path, options);
        let success = move_file(&new_target, original, options);

        if success {
            remove_file(&backup_path, options);
        } else {
            move_file(&backup_path, original, options);
        }
    } else if options.verbose {
        println!("Original and target are not same type (one is file but other is directory). Skipping...");
    }
}

/*
 * iterates over everything inside a directory and sends everything to adopt()
 */
fn adopt_all_inside_dir(original: &PathBuf, target: &PathBuf, use_special_paths: bool, options: &Options) {
    let fname = get_name(target);

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        if options.verbose {
            println!("{fname} couldn't be read. Skipping...");
        }
        return;
    }

    let mut write_location = target.clone();
    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            write_location.push(element.file_name());
            adopt(&element.path(), &write_location, use_special_paths, options);
            write_location.pop();
        });
}

/*
 * Prints version info
 */
#[inline(always)]
fn print_version() {
    println!("rustow version 0.3-beta");
}

/*
 * Prints the help message
 */
#[inline(always)]
fn print_help() {
    print_version();
    println!(r#"
Usage:
    rustow [OPTION ...] [-S|-D|-R|-A] PACKAGE ... [-S|-D|-R|-A] PACKAGE ...

Actions:
    -S  Stow the package.
        Creates symlinks of files in the package to target directory
    -D  Unstow the package.
        Removes existing symlinks of files in the package in target directory
    -R  Restow the package.
        Same as unstowing and stowing a package
    -A  Adopt the package.
        Imports existing files in target directory to stow package. USE WITH CAUTION!

Options:
    -h      --help            Prints this help message
    -V      --version         Prints version info
    -d DIR  --stow-dir DIR    Set stow dir to DIR (default is current dir)
    -t DIR  --target-dir DIR  Set target dir to DIR (default is parent of stow dir)
           "#);
}

/*
 * Takes path and looks its file name
 * If it is special path, returns special path
 * Otherwise returns same path
 */
fn handle_special_path(original: &PathBuf, destination: &PathBuf, options: &Options) -> PathBuf {
    if !options.special_keywords { return destination.clone(); }

    let name = get_name(destination);
    match name.as_ref() {
        "@home" => {
            match env::var("HOME") {
                Ok(path) => {
                    let p = PathBuf::from(path);

                    if p.exists() {
                        return p;
                    } else {
                        println!("Couldn't find HOME variable.");
                        process::exit(1);
                    }
                }
                Err(e) => {
                        println!("Couldn't find HOME variable. {}", e);
                        process::exit(1);
                }
            }
        }
        "@root" => {
            if !options.security_check || is_root_file(original) {
                return PathBuf::from("/");
            } else {
                println!("For security reasons, all the files/folders including and followed by @root file must be owned by root.");
                println!("If you want to stow something inside your home folder, use @home instead.");
                // TODO: add --no-security-check flag
                //println!("Use --no-security-check flag to prevent from this error");
                process::exit(1);
            }
        }
        _ => destination.clone()
    }
}

enum PushMode {
    Stow,
    Unstow,
    Restow,
    Adopt,
}

