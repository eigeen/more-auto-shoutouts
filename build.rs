#[cfg(feature = "use_logger")]
fn use_logger() {
    println!("cargo:rustc-link-search=lib");
    println!("cargo:rustc-link-lib=static=LoaderFFI");
    println!("cargo:rustc-link-lib=static=loader");
}

#[cfg(not(feature = "use_logger"))]
fn use_logger() {
}

fn main() {
    use_logger()
}
