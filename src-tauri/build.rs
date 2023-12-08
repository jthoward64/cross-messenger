use cc;

fn main() {
    // Get the python include path
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg("import sysconfig; print(sysconfig.get_paths()['include'])")
        .output()
        .expect("failed to execute process");

    dbg!(&output);

    cc::Build::new()
        .include(std::str::from_utf8(&output.stdout).unwrap().trim())
        .file("src/emulated/bindings.c")
        .compile("emulated");
    tauri_build::build()
}
