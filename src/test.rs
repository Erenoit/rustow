use super::*;

/*
 * Unit tests module
 * TEST_NO is used for creating unique directory for each test
 * This allows us to run tusts in parallel
 * Every test name briefly describes what it tests
 */

fn ready_test(test_no: u8) {
    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{test_no}"));

    match fs::create_dir(&working_dir) {
        Ok(_) => {}
        Err(e) => {
            match e.kind() {
                io::ErrorKind::AlreadyExists => {
                    clean_test(test_no);
                    fs::create_dir(&working_dir).expect("Couldn't create test directory.");
                }
                _ => {panic!("Couldn't create test directory. {e}")}
            }
        }
    }

    working_dir.push("stow_dir");
    fs::create_dir(&working_dir).expect("Couldn't create stow directory.");

    working_dir.pop();
    let inside = fs::read_dir(working_dir).expect("Couldn't read test directory.")
        .filter(|e| { e.is_ok() })
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();

    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0].file_name(), "stow_dir");
}

fn clean_test(test_no: u8) {
    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{test_no}"));

    match fs::remove_dir_all(working_dir) {
        Ok(_) => {}
        Err(e) => {
            match e.kind() {
                io::ErrorKind::NotFound => {}
                _ => {panic!("Couldn't remove test directory. {e}")}
            }
        }
    }
}

fn create_environment(directory: &PathBuf) {
    let mut working_dir = directory.clone();

    working_dir.push("existing_directory");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.pop();

    working_dir.push("stow_dir");

    working_dir.push("stow1");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("existing_directory");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("file_in_existing_directory");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.pop();
    working_dir.push("file_in_target_root");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.pop();

    working_dir.push("stow2");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("existing_directory");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("another_file_in_existing_directory");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.pop();
    working_dir.push("another_directory");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("file_in_another_directory1");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.push("file_in_another_directory2");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.pop();
    working_dir.pop();

    working_dir.push("stow3");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("another_directory");
    fs::create_dir(&working_dir).expect("Test environment creation failed");
    working_dir.push("file_in_another_directory3");
    fs::File::create(&working_dir).expect("Test environment creation failed");
    working_dir.pop();
    working_dir.pop();
    working_dir.pop();
}

#[test]
fn basic_stow_file() {
    const TEST_NO: u8 = 0;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);


    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow1");
    file_path.push("file_in_target_root");

    working_dir.push("file_in_target_root");

    stow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 3) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!( directories_after[1].path().is_symlink());
    assert!(!directories_after[2].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn basic_stow_folder() {
    const TEST_NO: u8 = 1;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);


    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow2");
    file_path.push("another_directory");

    working_dir.push("another_directory");

    stow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 3) ;
    assert!( directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());
    assert!(!directories_after[2].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn basic_stow_w_parent_directory_exits() {
    const TEST_NO: u8 = 2;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);


    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow1");
    file_path.push("existing_directory");

    working_dir.push("existing_directory");

    stow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 2) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());

    working_dir.push("existing_directory");
    let inside_existing_dir = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();

    assert_eq!(inside_existing_dir.len(), 1) ;
    assert!(inside_existing_dir[0].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn full_stow() {
    const TEST_NO: u8 = 3;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);


    working_dir.push("stow_dir");
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
            !e.file_name().to_string_lossy().starts_with(".")
        })
        .collect::<Vec<_>>();

    working_dir.pop();
    for dir in directories {
        stow_all_inside_dir(&dir.path(), &working_dir);
    }


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 4) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());
    assert!( directories_after[2].path().is_symlink());
    assert!(!directories_after[3].path().is_symlink());

    working_dir.push("existing_directory");
    let inside_existing_dir = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();

    assert_eq!(inside_existing_dir.len(), 2);
    assert!(inside_existing_dir[0].path().is_symlink());
    assert!(inside_existing_dir[1].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn basic_unstow_file() {
    const TEST_NO: u8 = 4;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);

    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow1");
    file_path.push("file_in_target_root");

    working_dir.push("file_in_target_root");

    stow(&file_path, &working_dir);

    working_dir.pop();

    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 3) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!( directories_after[1].path().is_symlink());
    assert!(!directories_after[2].path().is_symlink());


    working_dir.push("file_in_target_root");

    unstow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 2) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn basic_unstow_folder() {
    const TEST_NO: u8 = 5;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);

    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow2");
    file_path.push("another_directory");

    working_dir.push("another_directory");

    stow(&file_path, &working_dir);

    working_dir.pop();

    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 3) ;
    assert!( directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());
    assert!(!directories_after[2].path().is_symlink());


    working_dir.push("another_directory");

    unstow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 2) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn basic_unstow_w_parent_directory_exits() {
    const TEST_NO: u8 = 6;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);

    let mut file_path = working_dir.clone();
    file_path.push("stow_dir");
    file_path.push("stow1");
    file_path.push("existing_directory");

    working_dir.push("existing_directory");

    stow(&file_path, &working_dir);

    working_dir.pop();

    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 2) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());

    working_dir.push("existing_directory");
    let inside_existing_dir = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();

    assert_eq!(inside_existing_dir.len(), 1) ;
    assert!(inside_existing_dir[0].path().is_symlink());

    working_dir.pop();


    working_dir.push("existing_directory");

    unstow(&file_path, &working_dir);

    working_dir.pop();


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 1) ;
    assert!(!directories_after[0].path().is_symlink());


    clean_test(TEST_NO);
}

#[test]
fn full_unstow() {
    const TEST_NO: u8 = 7;

    ready_test(TEST_NO);

    let mut working_dir = env::current_dir().expect("Working directory couldn't found.");
    working_dir.push(format!("test_temp_file_{TEST_NO}"));

    create_environment(&working_dir);

    working_dir.push("stow_dir");
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
            !e.file_name().to_string_lossy().starts_with(".")
        })
        .collect::<Vec<_>>();

    working_dir.pop();
    for dir in directories {
        stow_all_inside_dir(&dir.path(), &working_dir);
    }

    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 4) ;
    assert!(!directories_after[0].path().is_symlink());
    assert!(!directories_after[1].path().is_symlink());
    assert!( directories_after[2].path().is_symlink());
    assert!(!directories_after[3].path().is_symlink());

    working_dir.push("existing_directory");
    let inside_existing_dir = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();

    assert_eq!(inside_existing_dir.len(), 2);
    assert!(inside_existing_dir[0].path().is_symlink());
    assert!(inside_existing_dir[1].path().is_symlink());

    working_dir.pop();


    working_dir.push("stow_dir");
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
            !e.file_name().to_string_lossy().starts_with(".")
        })
        .collect::<Vec<_>>();

    working_dir.pop();
    for dir in directories {
        unstow_all_inside_dir(&dir.path(), &working_dir);
    }


    let mut directories_after = fs::read_dir(&working_dir)
        .expect("Couldn't read working directory.")
        .filter(|e| { e.is_ok() })        // Remove errors
        .map(|e| { e.unwrap() })
        .collect::<Vec<_>>();
    directories_after.sort_by(|a, b| { a.file_name().cmp(&b.file_name()) });

    assert_eq!(directories_after.len(), 1) ;
    assert!(!directories_after[0].path().is_symlink());


    clean_test(TEST_NO);
}

