use std::env;

mod util;
mod structure;

mod class_leader;
mod javap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <class file path>", args[0]);
        return;
    }
    let path = &args[1];
    let class_file = class_leader::read_file(path);

    match class_file {
        Ok(cf) => javap::javap_viewer(cf),
        Err(e) => eprintln!("Error reading class file: {}", e),
    }
}
