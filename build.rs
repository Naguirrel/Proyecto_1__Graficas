fn main() {
    if cfg!(target_os = "linux") {
        for libudev_path in [
            "/usr/lib64/libudev.so.1",
            "/lib64/libudev.so.1",
            "/usr/lib/x86_64-linux-gnu/libudev.so.1",
            "/lib/x86_64-linux-gnu/libudev.so.1",
        ] {
            if std::path::Path::new(libudev_path).exists() {
                if let Some(parent) = std::path::Path::new(libudev_path).parent() {
                    println!("cargo:rustc-link-search=native={}", parent.display());
                }

                println!("cargo:rustc-link-arg={libudev_path}");
                break;
            }
        }
    }
}
