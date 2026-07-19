#![forbid(unsafe_code)]

mod feature_map;
mod simdoc;

fn main() {
    if let Err(err) = simdoc::run(std::env::args().collect()) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
