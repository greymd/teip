fn main() {
    match pkg_config::Config::new().probe("onig") {
        Ok(library) => {
            for include_path in library.include_paths {
                println!("cargo:include={}", include_path.to_string_lossy());
            }
        }
        Err(_) => {}
    }
}
