use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn copy_dir_contents(src_dir: &Path, dst_dir: &Path) -> io::Result<()> {
    if !src_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst_dir)?;

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst_dir.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn print_rerun_if_changed_for_dir(dir: &Path) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            print_rerun_if_changed_for_dir(&path)?;
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(())
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let manifest_path = Path::new(&manifest_dir);
        println!("cargo:rustc-link-search=native={manifest_dir}");
        println!("cargo:rerun-if-changed=SDL2.lib");
        println!("cargo:rerun-if-changed=SDL2_mixer.lib");
        println!("cargo:rerun-if-changed=SDL2.dll");
        println!("cargo:rerun-if-changed=SDL2_mixer.dll");
        println!("cargo:rerun-if-changed=assets");
        print_rerun_if_changed_for_dir(&manifest_path.join("assets"))
            .expect("failed to track assets changes");

        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let profile_dir = Path::new(&out_dir)
            .ancestors()
            .nth(3)
            .expect("failed to resolve target profile directory");

        for dll_name in ["SDL2.dll", "SDL2_mixer.dll"] {
            let src = manifest_path.join(dll_name);
            let dst = profile_dir.join(dll_name);
            if src.exists() {
                fs::copy(&src, &dst).unwrap_or_else(|err| {
                    panic!(
                        "failed to copy {} to {}: {err}",
                        src.display(),
                        dst.display()
                    )
                });
            }
        }

        let assets_src = manifest_path.join("assets");
        let assets_dst = profile_dir;
        copy_dir_contents(&assets_src, &assets_dst).unwrap_or_else(|err| {
            panic!(
                "failed to copy assets from {} to {}: {err}",
                assets_src.display(),
                assets_dst.display()
            )
        });
    }
}
