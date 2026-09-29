fn main() {
    println!("cargo:rerun-if-changed=native.c");
    let mut build = cc::Build::new();
    build
        .file("native.c")
        .include(std::env::var_os("DEP_DOBBY_INCLUDE").unwrap());
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        build.compiler("clang-cl");
    }
    build.compile("dobby_sys_e2e_fixture");
}
