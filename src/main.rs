mod cmd;
mod stower;

use clap::Parser;

use crate::{cmd::Args, stower::Stower};

// TODO: add tests

fn main() { Stower::new(Args::parse()).unwrap().run(); }
