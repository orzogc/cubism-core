use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(feature = "doc_only") {
        return;
    }

    let cubism_dir = env::var("LIVE2D_CUBISM").expect(
        "The environment variable `LIVE2D_CUBISM` is not set properly. \
        `LIVE2D_CUBISM` should be set to the Live2D Cubism directory.",
    );
    println!("cargo:rerun-if-env-changed=LIVE2D_CUBISM");

    let mut lib_dir = PathBuf::from(cubism_dir);
    if !lib_dir.exists() {
        panic!("{} didn't exist", lib_dir.display());
    }
    lib_dir.push("Core");
    let mut header = lib_dir.clone();
    header.push("include");
    header.push("Live2DCubismCore.h");
    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.display().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .blocklist_type("csmMoc")
        .blocklist_type("csmModel")
        .generate()
        .expect("failed to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("cubism_core.rs"))
        .expect("failed to write bindings");

    let profile = env::var("PROFILE").unwrap();
    let target = env::var("TARGET").unwrap();
    let (arch, vendor, sys, abi) = {
        let mut split = target.split('-');
        (
            split.next().unwrap_or(""),
            split.next().unwrap_or(""),
            split.next().unwrap_or(""),
            split.next().unwrap_or(""),
        )
    };

    let link = if cfg!(feature = "static") && !cfg!(feature = "dynamic") {
        "static"
    } else {
        "dylib"
    };
    if link == "static" {
        lib_dir.push("lib");
    } else {
        lib_dir.push("dll");
    }

    let mut runtime = String::new();

    match (vendor, sys) {
        ("apple", "darwin") => {
            if arch != "x86_64" {
                panic!("only support x86_64 for macOS");
            }
            lib_dir.push("macos");
        }
        ("apple", "ios") => {
            if link != "static" {
                panic!("no dynamic lib support for iOS");
            }
            lib_dir.push("ios");
            let mut ios_dir = String::new();
            if profile == "release" {
                ios_dir.push_str("Release-");
            } else {
                ios_dir.push_str("Debug-");
            }
            let ios = env::var("IOS_BUILD").unwrap_or("device".to_string());
            match ios.as_str() {
                "device" => {
                    if arch != "aarch64" {
                        panic!("only support aarch64 for iOS device");
                    }
                    ios_dir.push_str("iphoneos");
                }
                "simulator" => {
                    if arch != "x86_64" {
                        panic!("only support x86_64 for iOS simulator");
                    }
                    ios_dir.push_str("iphonesimulator");
                }
                _ => panic!("unsupported iOS build: {}", ios),
            }
            lib_dir.push(ios_dir);
        }
        ("linux", "android") | ("linux", "androideabi") => {
            lib_dir.push("android");
            lib_dir.push(match arch {
                "i686" => "x86",
                "armv7" => "armeabi-v7a",
                "aarch64" => "arm64-v8a",
                _ => panic!("only support i686, armv7 and aarch64 for Android"),
            });
        }
        ("pc", "windows") => {
            lib_dir.push("windows");
            lib_dir.push(match arch {
                "i586" | "i686" => "x86",
                "x86_64" => "x86_64",
                _ => panic!("only support i586/i686 and x86_64 for Windows"),
            });
            if link == "static" {
                if abi != "msvc" {
                    panic!("need msvc ABI to link Live2D Cubism Core's Windows static lib");
                }
                let msvc = env::var("VISUAL_STUDIO_VERSION").unwrap_or("140".to_string());
                match msvc.as_str() {
                    "120" | "140" | "141" | "142" => lib_dir.push(msvc),
                    _ => panic!("unsupported Visual Studio version: {}", msvc),
                }
                let runtime_lib = env::var("RUNTIME_LIB").unwrap_or("MT".to_string());
                match runtime_lib.as_str() {
                    "MD" => runtime.push_str("Live2DCubismCore_MD"),
                    "MT" => runtime.push_str("Live2DCubismCore_MT"),
                    _ => panic!("unsupported run-time library: {}", runtime_lib),
                }
            }
        }
        ("unknown", "linux") => match arch {
            "x86_64" => {
                lib_dir.push("linux");
                lib_dir.push("x86_64");
            }
            "arm" | "armv7" => {
                lib_dir.push("experimental");
                lib_dir.push("rpi");
            }
            _ => {
                panic!("unsupported Linux architecture: {}", arch)
            }
        },
        ("uwp", "windows") => {
            if link == "static" {
                panic!("no static lib support for UWP")
            }
            lib_dir.push("experimental");
            lib_dir.push("uwp");
            lib_dir.push(match arch {
                "thumbv7a" => "arm",
                "aarch64" => "arm64",
                "i686" => "x86",
                "x86_64" => "x64",
                _ => {
                    panic!("unsupported UWP architecture: {}", arch)
                }
            })
        }
        _ => panic!("unsupported target: {}", target),
    }

    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rustc-link-search=all={}", lib_dir.display());

    match (link, vendor, sys, profile.as_str()) {
        ("static", "pc", "windows", "debug") => {
            println!("cargo:rustc-link-lib={}={}d", link, runtime)
        }
        ("static", "pc", "windows", "release") => {
            println!("cargo:rustc-link-lib={}={}", link, runtime)
        }
        _ => println!("cargo:rustc-link-lib={}=Live2DCubismCore", link),
    }
}
