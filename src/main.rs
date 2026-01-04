#[cfg(feature = "debug")]
mod util;

// use std::env;
// use std::fs;
// use std::path::Path;

mod nes;

fn main() -> std::io::Result<()> {
    // util::debug::debug(
    //     "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA",
    // );

    let path = std::env::args().nth(1).expect("No ROM path given");
    let mut nes = nes::Nes::new(path);
    nes.reset();
    nes.run();
    Ok(())
}
