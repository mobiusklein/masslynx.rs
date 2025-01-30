use std::{io, env};

fn main() -> io::Result<()> {
    let current_dir = env::current_dir()?;

    println!("cargo:rustc-link-lib=static=MassLynxRaw");
    println!("cargo:rustc-link-search=native={}/lib", current_dir.display());
    Ok(())
}
