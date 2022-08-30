use std::{env, fs::{self, DirEntry}, io::{self, Write}, path::PathBuf, process, os::unix};

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
        })
        .collect::<Vec<_>>();

    let mut write_dir_main = working_dir.clone();
    write_dir_main.pop();


    let (stow_l, unstow_l, restow_l, adopt_l) = handle_cmd_arguments(&directories);

    if unstow_l.len() > 0 {
        println!("Unstow functionality is not implemented yet. Skipping...");
    }

    if restow_l.len() > 0 {
        println!("Restow functionality is not implemented yet. Skipping...");
    }

    if stow_l.len() > 0 {
        for directory in stow_l {
            stow_all_inside_dir(directory.path(), write_dir_main.clone());
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
fn handle_cmd_arguments(directories: &Vec<DirEntry>) -> (Vec<&DirEntry>, Vec<&DirEntry>, Vec<&DirEntry>, Vec<&DirEntry>) {
    let mut stow   = Vec::<&DirEntry>::new();
    let mut unstow = Vec::<&DirEntry>::new();
    let mut restow = Vec::<&DirEntry>::new();
    let mut adopt  = Vec::<&DirEntry>::new();

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
                _ => {
                    println!("Unknown argument: {argument}.");
                    println!("Use `rustow -h` for available arguments.");
                    process::exit(1);
                }
            }
        } else {
            let mut is_valid = false;
            for e in directories {
                if e.file_name().to_string_lossy() == argument {
                    is_valid = true;
                    match push_mode {
                        PushMode::Stow   => stow.push(e),
                        PushMode::Unstow => unstow.push(e),
                        PushMode::Restow => restow.push(e),
                        PushMode::Adopt  => adopt.push(e),
                    }
                }
            }
            if !is_valid {
                println!("Invalid argument: {argument}.");
                print!("Do you want to continue without this argument (Y/n): ");
                io::stdout().flush().expect("Failed to print.");
                let mut buffer = String::new();
                let stdin = io::stdin();
                _ = stdin.read_line(&mut buffer);
                let res = buffer.trim().to_lowercase();

                if res != "y" && res != "yes" && res != "" {
                    process::exit(1);
                }
            }
        }
    }

    return (stow, unstow, restow, adopt)  ;
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
            print!("{} already exists, would you like to delete it and replace with symlink (y/N): ", fname);
            io::stdout().flush().expect("Failed to print.");
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

enum PushMode {
    Stow,
    Unstow,
    Restow,
    Adopt,
}

