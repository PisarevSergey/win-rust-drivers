use std::env;
use std::path::PathBuf;

fn main() {
    let mut inc_dirs = vec![get_km_inc_dir(), get_shared_inc_dir(), get_km_crt_inc_dir()];
    inc_dirs
        .iter_mut()
        .for_each(|s| s.insert_str(0, "-isystem"));

    let mut clang_args = inc_dirs;

    clang_args.append(&mut Vec::from([
        String::from("-D_KERNEL_MODE=1"),
        String::from("-DWINNT=1"),
        String::from("-DWINVER=0x0A00"),
        String::from("-D_WIN32_WINNT=0x0A00"),
        String::from("-DNTDDI_VERSION=0xA000012"),
    ]));

    if "debug" == env::var("PROFILE").unwrap().as_str() {
        clang_args.append(&mut Vec::from([
            String::from("-DDBG=1"),
            String::from("-DMSC_NOOPT"),
            String::from("-DDEPRECATE_DDK_FUNCTIONS=1"),
        ]));
    }

    let target = env::var("TARGET").unwrap();
    match target.as_str() {
        "aarch64-pc-windows-msvc" => {
            let mut defs = [
                "_ARM64_".to_string(),
                "ARM64".to_string(),
                "_USE_DECLSPECS_FOR_SAL=1".to_string(),
                "STD_CALL".to_string(),
                "_ARM64_WINAPI_PARTITION_DESKTOP_SDK_AVAILABLE=1".to_string(),
            ];
            defs.iter_mut().for_each(|s| s.insert_str(0, "-D"));
            clang_args.append(&mut Vec::from(defs));
        }
        "x86_64-pc-windows-msvc" => {
            let mut defs = [
                "_WIN64".to_string(),
                "_AMD64_".to_string(),
                "AMD64".to_string(),
            ];
            defs.iter_mut().for_each(|s| s.insert_str(0, "-D"));
            clang_args.append(&mut Vec::from(defs));
        }
        _ => panic!("unsupported target {target}"),
    }

    // clang_args.push(String::from("-fpack-struct=8"));

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("flt.h")
        .clang_args(clang_args)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .layout_tests(false)
        .use_core()
        .wrap_unsafe_ops(true)
        .derive_default(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn get_wdk_root_inc_dir() -> String {
    let mut wdk_inc_root = env::var("WDKContentRoot").unwrap();

    wdk_inc_root.push_str(r"Include\");

    let wdk_version = env::var("WindowsTargetPlatformVersion").unwrap();

    wdk_inc_root.push_str(&wdk_version);

    wdk_inc_root
}

fn get_km_inc_dir() -> String {
    let mut km_dir = get_wdk_root_inc_dir();
    km_dir.push_str(r"\km");

    km_dir
}

fn get_shared_inc_dir() -> String {
    let mut sh_dir = get_wdk_root_inc_dir();
    sh_dir.push_str(r"\shared");

    sh_dir
}

fn get_km_crt_inc_dir() -> String {
    let mut crt_inc = get_km_inc_dir();
    crt_inc.push_str(r"\crt");

    crt_inc
}
