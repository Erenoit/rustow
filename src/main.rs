use std::{env, fs, io, path::PathBuf, os::unix};

fn main() {
    let working_dir = env::current_dir().expect("Working directory couldn't found.");

    let directories = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .filter(|e| {                     // Only take directories
            let ftype = e.file_type();
            if ftype.is_err() { return false; }

            ftype.unwrap().is_dir()
        })
        .filter(|e| {                     // Remove files starts with dot
            let fname = e.file_name().into_string();
            if fname.is_err() { return false; }

            !fname.unwrap().starts_with(".")
        });

    let mut write_dir_main = working_dir.clone();
    write_dir_main.pop();
    for file in directories {
        stow_all_inside_dir(file.path(), write_dir_main.clone());
    }
}

/*
 * if there is already a folder try stow things inside the folder
 * if there is already a file ask if user wants to remove existing one and stow or cancel
 * otherwise creates symlink
 */
fn stow(original: PathBuf, destination: PathBuf) {
    let fname = destination
        .file_name()
        .expect("There should always be a file name.")
        .to_string_lossy();

    if destination.is_symlink() {
        println!("{} is already stowed. Skipping...", fname);
    } else if destination.exists() {
        if destination.is_dir() {
            stow_all_inside_dir(original, destination);
        } else {
            println!("{} already exists, would you like to delete it and replace with symlink (y/N): ", fname);
            let mut buffer = String::new();
            let stdin = io::stdin();
            _ = stdin.read_line(&mut buffer);
            let res = buffer.trim().to_lowercase();

            if res == "y" || res == "yes" {
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
fn stow_all_inside_dir(original: PathBuf, destination: PathBuf) {
    let fname = destination
        .file_name()
        .expect("There should always be a file name.")
        .to_string_lossy();

    let subdirs = fs::read_dir(original);
    if subdirs.is_err() { 
        println!("{} couldn't be read. Skipping...", fname);
        return;
    }

    subdirs.unwrap()
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .for_each(|element| {
            let mut write_location = destination.clone();
            write_location.push(element.file_name());
            stow(element.path(), write_location);
        });
}

