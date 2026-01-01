#[cfg(feature = "debug")]
mod util;

pub mod bus;
pub mod cpu;
pub mod instructions;

fn main() {
    util::debug::debug(
        "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA",
    );
}
