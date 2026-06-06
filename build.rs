use std::{env, fs, path::PathBuf};

const DEFAULT_WINDOWS_ICON: &str = "scripts/icon/bat_to_exe.ico";

fn main() {
    println!("cargo:rerun-if-env-changed=EXEFOUNDRY_RUNNER_TEMPLATE");
    println!("cargo:rerun-if-changed={DEFAULT_WINDOWS_ICON}");
    embed_windows_metadata();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let generated = out_dir.join("runner_template.rs");

    if let Ok(template) = env::var("EXEFOUNDRY_RUNNER_TEMPLATE") {
        println!("cargo:rerun-if-changed={template}");
        let bytes = fs::read(&template).expect("failed to read EXEFOUNDRY_RUNNER_TEMPLATE");
        fs::write(
            generated,
            format!(
                "pub const RUNNER_TEMPLATE: Option<&[u8]> = Some(&{:?});",
                bytes
            ),
        )
        .expect("failed to write embedded runner template");
    } else {
        fs::write(
            generated,
            "pub const RUNNER_TEMPLATE: Option<&[u8]> = None;",
        )
        .expect("failed to write empty runner template marker");
    }
}

#[cfg(windows)]
fn embed_windows_metadata() {
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(DEFAULT_WINDOWS_ICON)
        .set("FileDescription", "ExeFoundry BAT to EXE Converter")
        .set("ProductName", "ExeFoundry");

    resource
        .compile()
        .expect("failed to embed ExeFoundry Windows icon/resource metadata");
}

#[cfg(not(windows))]
fn embed_windows_metadata() {}
