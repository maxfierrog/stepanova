//! Exports the device binary's symbols in its dynamic symbol table, so the
//! vendor e-paper engine binds to the interposed Qt constructor (see
//! `vendor.rs`).

fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target == "linux" {
        println!("cargo:rustc-link-arg-bins=-Wl,--export-dynamic");
    }
}
