use std::{env, fs, os::unix};

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
        let subdirs = fs::read_dir(file.path());
        if subdirs.is_err() { continue; }

        subdirs.unwrap()
            .filter(|e| { e.is_ok() })
            .map(|e| { e.unwrap() })
            .for_each(|element| {
                let mut write_location = write_dir_main.clone();
                write_location.push(element.file_name());
                _ = unix::fs::symlink(element.path(), write_location);
            });
    }
}
