mod cmd;
mod stower;

use clap::Parser;

use crate::{cmd::Args, stower::Stower};

// TODO: make --simulate keep trck of changes so it will generate more real
// outcome

// TODO: add tests

fn main() { Stower::new(Args::parse()).unwrap().run(); }
