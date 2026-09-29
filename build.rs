use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dobby");
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("dobby");
    assert!(
        source.join("CMakeLists.txt").is_file(),
        "Dobby source is missing; run git submodule update --init --recursive"
    );
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target = env::var("TARGET").unwrap();
    assert!(
        matches!(arch.as_str(), "aarch64" | "arm" | "x86" | "x86_64"),
        "unsupported Dobby architecture: {target}"
    );
    assert!(
        matches!(os.as_str(), "android" | "linux" | "macos" | "windows"),
        "unsupported Dobby platform: {target}"
    );
    if os == "windows" {
        assert!(
            target == "x86_64-pc-windows-msvc",
            "Windows support requires x86_64-pc-windows-msvc"
        );
    }

    let mut config = cmake::Config::new(&source);
    config
        .generator("Ninja")
        .profile("Release")
        .no_default_flags(true)
        .define("DOBBY_GENERATE_SHARED", "OFF")
        .define("DOBBY_BUILD_TEST", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("Plugin.SymbolResolver", "ON")
        .define(
            "DOBBY_DEBUG",
            if cfg!(feature = "native-debug") {
                "ON"
            } else {
                "OFF"
            },
        )
        .define(
            "FullFloatingPointRegisterPack",
            if cfg!(feature = "full-floating-point-register-pack") {
                "ON"
            } else {
                "OFF"
            },
        );

    if os == "android" {
        configure_android(&mut config, &arch);
    } else if os == "windows" {
        config
            .define("CMAKE_C_COMPILER", "clang")
            .define("CMAKE_CXX_COMPILER", "clang++")
            .define("CMAKE_ASM_COMPILER", "clang");
        let static_crt = env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .split(',')
            .any(|feature| feature == "crt-static");
        config.define(
            "CMAKE_MSVC_RUNTIME_LIBRARY",
            if static_crt {
                "MultiThreaded"
            } else {
                "MultiThreadedDLL"
            },
        );
    }

    let installed = config.build();
    println!(
        "cargo:rustc-link-search=native={}",
        installed.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=dobby");
    println!("cargo:include={}", source.join("include").display());
    match os.as_str() {
        "android" => {
            println!("cargo:rustc-link-lib=static=c++_static");
            println!("cargo:rustc-link-lib=static=c++abi");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=log");
            println!("cargo:rustc-link-lib=m");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=pthread");
        }
        "macos" => println!("cargo:rustc-link-lib=c++"),
        _ => {}
    }
}

fn tracked_env(name: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={name}");
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn configure_android(config: &mut cmake::Config, arch: &str) {
    let toolchain = tracked_env("CARGO_NDK_CMAKE_TOOLCHAIN_PATH").map(PathBuf::from);
    let ndk_home = tracked_env("ANDROID_NDK_HOME").or(tracked_env("ANDROID_NDK_ROOT"));
    let toolchain = toolchain
        .or_else(|| {
            ndk_home.map(|root| PathBuf::from(root).join("build/cmake/android.toolchain.cmake"))
        })
        .expect("Android requires ANDROID_NDK_HOME or CARGO_NDK_CMAKE_TOOLCHAIN_PATH");
    assert!(
        toolchain.is_file(),
        "Android toolchain not found: {}",
        toolchain.display()
    );
    let runtime_libraries = tracked_env("CARGO_NDK_SYSROOT_LIBS_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let ndk = toolchain
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap();
            let host = match env::consts::OS {
                "windows" => "windows-x86_64",
                "linux" => "linux-x86_64",
                "macos" => "darwin-x86_64",
                other => panic!("unsupported NDK build host: {other}"),
            };
            let triple = match arch {
                "aarch64" => "aarch64-linux-android",
                "arm" => "arm-linux-androideabi",
                "x86" => "i686-linux-android",
                "x86_64" => "x86_64-linux-android",
                _ => unreachable!(),
            };
            ndk.join("toolchains/llvm/prebuilt")
                .join(host)
                .join("sysroot/usr/lib")
                .join(triple)
        });
    assert!(
        runtime_libraries.join("libc++_static.a").is_file(),
        "NDK C++ runtime not found: {}",
        runtime_libraries.display()
    );
    // Do not export the NDK's unversioned sysroot directory: it contains libc.a
    // and would shadow the API-specific libc.so during the final Rust link.
    let runtime_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("runtime");
    fs::create_dir_all(&runtime_dir).unwrap();
    for name in ["libc++_static.a", "libc++abi.a"] {
        let archive = runtime_libraries.join(name);
        println!("cargo:rerun-if-changed={}", archive.display());
        fs::copy(&archive, runtime_dir.join(name)).expect("copy NDK C++ runtime");
    }
    println!("cargo:rustc-link-search=native={}", runtime_dir.display());
    let api = tracked_env("ANDROID_PLATFORM").unwrap_or_else(|| "21".into());
    // rustc uses -nodefaultlibs; Clang's cache-flush builtins must be linked explicitly.
    let prebuilt = runtime_libraries.ancestors().nth(4).unwrap();
    let clang = prebuilt
        .join("bin")
        .join(if cfg!(windows) { "clang.exe" } else { "clang" });
    let triple = match arch {
        "aarch64" => "aarch64-linux-android",
        "arm" => "armv7a-linux-androideabi",
        "x86" => "i686-linux-android",
        "x86_64" => "x86_64-linux-android",
        _ => unreachable!(),
    };
    let level = api.strip_prefix("android-").unwrap_or(&api);
    assert!(
        level.parse::<u32>().is_ok(),
        "invalid Android API level: {api}"
    );
    let output = Command::new(clang)
        .arg(format!("--target={triple}{level}"))
        .arg("--print-libgcc-file-name")
        .output()
        .expect("query NDK compiler runtime");
    assert!(output.status.success(), "NDK compiler runtime query failed");
    let archive = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
    assert!(
        archive.is_file(),
        "NDK builtins not found: {}",
        archive.display()
    );
    let name = archive
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("lib")
        .unwrap()
        .strip_suffix(".a")
        .unwrap();
    println!("cargo:rerun-if-changed={}", archive.display());
    fs::copy(&archive, runtime_dir.join(archive.file_name().unwrap())).expect("copy NDK builtins");
    println!("cargo:rustc-link-lib=static={name}");
    let abi = match arch {
        "aarch64" => "arm64-v8a",
        "arm" => "armeabi-v7a",
        "x86" => "x86",
        "x86_64" => "x86_64",
        _ => unreachable!(),
    };
    config
        .define("CMAKE_TOOLCHAIN_FILE", toolchain)
        .define("CMAKE_SYSTEM_NAME", "Android")
        .define("ANDROID_ABI", abi)
        .define("ANDROID_PLATFORM", api)
        .define("ANDROID_STL", "c++_static");
}
