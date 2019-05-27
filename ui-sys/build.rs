extern crate bindgen;
use bindgen::Builder as BindgenBuilder;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(unix)]
use std::os::unix;
#[cfg(windows)]
use std::os::windows;

fn main() {
    // Fetch the submodule if needed
    if cfg!(feature = "fetch") {
        Command::new("git")
            .args(&["version"])
            .status()
            .expect("Git does not appear to be installed. Error");
        // Init or update the submodule with libui if needed
        if !Path::new("libui/.git").exists() {
            Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status()
                .expect("Unable to init libui submodule. Error");
        } else {
            Command::new("git")
                .args(&["submodule", "update", "--recursive"])
                .status()
                .expect("Unable to update libui submodule. Error");
        }
    }

    // Generate libui bindings on the fly
    let bindings = BindgenBuilder::default()
        .header("wrapper.h")
        .opaque_type("max_align_t") // For some reason this ends up too large
        //.rustified_enum(".*")
        .generate()
        .expect("Unable to generate bindings. Error");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings. Error");

    // Deterimine build platform
    let target = env::var("TARGET").unwrap();
    let msvc = target.contains("msvc");
    let apple = target.contains("apple");

    let configured_out_dir = env::var("OUT_DIR").expect("Could not get OUT_DIR. Error");
    // Build libui if needed. Otherwise, assume it's in lib/
    let mut dst = Path::new(&configured_out_dir).join("build");
    let build_destination: String = dst
            .to_str()
            .expect("Could not stringify build destination.")
            .to_owned().clone();
    if cfg!(feature = "build") {
        // Verify that both meson and ninja are available
        Command::new("meson")
            .args(&["--version"])
            .status()
            .expect("Could not run meson. Error");
        Command::new("ninja")
            .args(&["--version"])
            .status()
            .expect("Could not run ninja. Error");

        // The directory where libui source is held.
        let source_root_dir = "libui";
        // The arguments to pass to meson
        let mut args = vec![];

        // If the build directory doesn't exist, create it
        if !dst.exists() {
            args.push("setup");
        } else {
            args.push("configure");
        }
        // Push the build directory
        args.push(&build_destination);

        // Choose the type of library to build
        if cfg!(feature = "static") {
            args.push("--default-library=static");
        } else {
            args.push("--default-library=shared");
        }

        // Choose which optimization level to build
        if cfg!(debug_assertions) {
            args.push("--buildtype=debug");
        } else {
            args.push("--buildtype=release")
        }

        Command::new("meson")
            .current_dir(source_root_dir)
            .args(&args)
            .status()
            .expect("Could not configure build. Error");

        Command::new("ninja")
            .current_dir(&dst)
            .status()
            .expect("Could not build libui. Error");

        dst = dst.join("meson-out");

        // Dynamic libraries we built on UNIX need to be symlinked.
        if !cfg!(feature = "static") && !msvc {
            let mut actual_so_location = dst.clone();
            let mut link_so_location = dst.clone();
            actual_so_location.push("libui.so.1");

            // ... but not if there's already a symlink.
            if (!link_so_location.exists()) {
                link_so_location.push("libui.so");
                unix::fs::symlink(&actual_so_location, &link_so_location)
                    .expect("Could not symlink .so.1 to .so. Error");
            }
        }

        if msvc {
            dst = dst.join("Release");
        }
    } else {
        dst = PathBuf::new().join("lib");
    }

    let libname;
    if msvc {
        libname = "libui";
    } else {
        libname = "ui";
    }

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=dylib={}", libname);
}
