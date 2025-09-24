fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.compile().expect("embed icon failed");
    }

    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
}
