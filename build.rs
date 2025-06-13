fn main() {
    println!("cargo:rustc-link-arg-examples=-Tlinkall.x");
    #[cfg(feature = "espidf")]
    embuild::espidf::sysenv::output();
}
