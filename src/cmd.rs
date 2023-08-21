use std::path::PathBuf;

use clap::{arg, command, ArgAction, Parser, ValueHint};

// TODO: add ability to add custom special keywords
// TODO: include dotfiles
// TODO: strict mode: fail if couldn't stow/unstow/adopt/restow any package

#[derive(Parser)]
#[command(author, version)]
pub struct Args {
    /// The directory containing the packages to be stowed.
    #[arg(
        short = 'd',
        long,
        value_name = "DIR",
        value_hint = ValueHint::DirPath,
        default_value = ".",
    )]
    pub stow_dir: PathBuf,

    /// The directory in which the packages will be stowed.
    #[arg(
        short = 't',
        long,
        value_name = "DIR",
        value_hint = ValueHint::DirPath,
        default_value = "..",
    )]
    pub target_dir: PathBuf,

    /// Enable verbose output.
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,

    /// Enable simulation mode.
    #[arg(short, long, default_value = "false")]
    pub simulate: bool,

    /// Disable the special paths feature.
    #[arg(long, default_value = "false")]
    pub no_special_paths: bool,

    /// Disable the security checks.
    #[arg(
        long,
        default_value = "false",
        default_value_if("no-special-keywords", "true", "true")
    )]
    pub no_security_check: bool,

    /// Stow the package.
    /// Creates symlinks of files in the package to target directory
    #[arg(
        short = 'S',
        long,
        value_name = "PACKAGE",
        num_args = 1..,
        action = ArgAction::Append,
        value_hint = ValueHint::FilePath,
        required_unless_present_any = ["unstow", "restow", "adopt"],
        next_line_help = true,
    )]
    pub stow: Vec<PathBuf>,

    /// Unstow the package.
    /// Removes existing symlinks of files in the package in target directory
    #[arg(
        short = 'D',
        long,
        value_name = "PACKAGE",
        num_args = 1..,
        action = ArgAction::Append,
        value_hint = ValueHint::FilePath,
        required_unless_present_any = ["stow", "restow", "adopt"],
        next_line_help = true,
    )]
    pub unstow: Vec<PathBuf>,

    /// Restow the package.
    /// Same as unstowing and stowing a package
    #[arg(
        short = 'R',
        long,
        value_name = "PACKAGE",
        num_args = 1..,
        action = ArgAction::Append,
        value_hint = ValueHint::FilePath,
        required_unless_present_any = ["stow", "unstow", "adopt"],
        next_line_help = true,
    )]
    pub restow: Vec<PathBuf>,

    /// Adopt the package.
    /// Imports existing files in target directory to stow package. USE WITH
    /// CAUTION!
    #[arg(
        short = 'A',
        long,
        value_name = "PACKAGE",
        num_args = 1..,
        action = ArgAction::Append,
        value_hint = ValueHint::FilePath,
        required_unless_present_any = ["stow", "unstow", "restow"],
        next_line_help = true,
    )]
    pub adopt: Vec<PathBuf>,
}
