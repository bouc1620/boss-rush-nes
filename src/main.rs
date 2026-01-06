mod nes;
#[cfg(feature = "debug")]
mod util;

#[cfg(feature = "debug")]
fn main() {
    match util::debug::debug_cpu(
        "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA",
    ) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Debug session failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "debug"))]
fn main() {
    let path = std::env::args().nth(1).expect("No ROM path given");
    match nes::Nes::from_rom(path) {
        Ok(mut nes) => {
            nes.reset();
            nes.run();
        }
        Err(e) => {
            eprintln!("Failed to start the NES emulator: {}", e);
            std::process::exit(1);
        }
    }
}
