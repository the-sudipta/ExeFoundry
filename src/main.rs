use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use editpe::{Image, ResourceDirectory};
use exefoundry::payload::{append_package, build_package};

include!(concat!(env!("OUT_DIR"), "/runner_template.rs"));

const IMAGE_SUBSYSTEM_WINDOWS_GUI: u16 = 2;
const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3;
const DEFAULT_ICON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/scripts/icon/bat_to_exe.ico"
));

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Convert a Windows .bat file into a single portable .exe"
)]
struct Cli {
    #[arg(
        short = 'i',
        long = "input",
        alias = "input-bat",
        value_name = "BAT",
        help = "Source .bat file to package"
    )]
    input_bat: Option<PathBuf>,

    #[arg(
        short = 'o',
        long = "output",
        alias = "output-exe",
        value_name = "EXE",
        help = "Output .exe path"
    )]
    output_exe: Option<PathBuf>,

    #[arg(
        long,
        value_name = "ICO_OR_IMAGE",
        help = "Optional icon/image to embed into the output EXE"
    )]
    icon: Option<PathBuf>,

    #[arg(
        long,
        alias = "win-exe",
        help = "Build the output EXE as a GUI app with no console window"
    )]
    gui: bool,

    #[arg(
        long,
        conflicts_with = "gui",
        help = "Build the output EXE as a console app"
    )]
    console: bool,

    #[arg(
        long,
        value_name = "EXE",
        help = "Use a Windows runner template instead of the embedded template"
    )]
    template: Option<PathBuf>,

    #[arg(long, help = "Prompt for missing input/output/icon values")]
    interactive: bool,
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    if should_prompt(&cli) {
        prompt_missing(&mut cli)?;
    }

    let input_bat = cli
        .input_bat
        .as_deref()
        .context("missing input BAT path; pass --input <file.bat> or use --interactive")?;
    let output_exe = cli
        .output_exe
        .clone()
        .unwrap_or_else(|| default_output_path(input_bat));

    convert(ConvertOptions {
        input_bat,
        output_exe: &ensure_exe_extension(output_exe),
        icon: cli.icon.as_deref(),
        gui: cli.gui,
        template: cli.template.as_deref(),
    })
}

struct ConvertOptions<'a> {
    input_bat: &'a Path,
    output_exe: &'a Path,
    icon: Option<&'a Path>,
    gui: bool,
    template: Option<&'a Path>,
}

fn convert(options: ConvertOptions<'_>) -> Result<()> {
    if options.input_bat.extension().and_then(|s| s.to_str()) != Some("bat") {
        bail!(
            "input file must have a .bat extension: {}",
            options.input_bat.display()
        );
    }

    let bat = fs::read(options.input_bat)
        .with_context(|| format!("failed to read BAT file {}", options.input_bat.display()))?;
    let mut image_bytes = load_template(options.template)?;

    {
        let mut image = Image::parse(image_bytes)
            .context("runner template is not a valid Windows PE executable")?;
        image.set_subsystem(if options.gui {
            IMAGE_SUBSYSTEM_WINDOWS_GUI
        } else {
            IMAGE_SUBSYSTEM_WINDOWS_CUI
        });

        apply_icon(&mut image, options.icon)?;

        image_bytes = image.data().to_vec();
    }

    let package = build_package(&bat, 0);
    append_package(&mut image_bytes, &package);

    if let Some(parent) = options
        .output_exe
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    fs::write(options.output_exe, image_bytes)
        .with_context(|| format!("failed to write {}", options.output_exe.display()))?;

    println!("OK: Built {}", options.output_exe.display());
    Ok(())
}

fn load_template(template: Option<&Path>) -> Result<Vec<u8>> {
    if let Some(path) = template {
        return fs::read(path)
            .with_context(|| format!("failed to read runner template {}", path.display()));
    }

    if let Some(bytes) = RUNNER_TEMPLATE {
        return Ok(bytes.to_vec());
    }

    bail!(
        "this development build does not contain an embedded Windows runner template; pass --template <exefoundry-runner.exe>"
    );
}

fn apply_icon(image: &mut Image<'_>, icon_path: Option<&Path>) -> Result<()> {
    let mut resources = image.resource_directory().cloned().unwrap_or_default();
    resources.remove_main_icon().ok();

    let Some(icon_path) = icon_path else {
        resources
            .set_main_icon(DEFAULT_ICON)
            .context("failed to parse bundled ExeFoundry icon")?;
        image
            .set_resource_directory(resources)
            .context("failed to write bundled icon resource into output EXE")?;
        return Ok(());
    };

    if !icon_path.exists() {
        bail!("icon not found: {}", icon_path.display());
    }

    let is_ico = icon_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("ico"))
        .unwrap_or(false);

    if is_ico {
        let icon = fs::read(icon_path)
            .with_context(|| format!("failed to read icon {}", icon_path.display()))?;
        resources
            .set_main_icon(icon.as_slice())
            .with_context(|| format!("failed to parse ICO {}", icon_path.display()))?;
    } else {
        resources
            .set_main_icon_file(&icon_path.to_string_lossy())
            .with_context(|| format!("failed to load icon image {}", icon_path.display()))?;
    }

    image
        .set_resource_directory(resources)
        .context("failed to write icon resource into output EXE")?;
    Ok(())
}

fn should_prompt(cli: &Cli) -> bool {
    cli.interactive || (cli.input_bat.is_none() && std::env::args_os().len() == 1)
}

fn prompt_missing(cli: &mut Cli) -> Result<()> {
    if cli.input_bat.is_none() {
        cli.input_bat = Some(prompt_path("Input BAT path")?);
    }

    if cli.output_exe.is_none() {
        let default = cli
            .input_bat
            .as_deref()
            .map(default_output_path)
            .context("input BAT path is required before choosing output")?;
        cli.output_exe = Some(
            prompt_optional_path(&format!("Output EXE path [{}]", default.display()))?
                .unwrap_or(default),
        );
    }

    if cli.icon.is_none() {
        cli.icon = prompt_optional_path("Icon path (.ico/.png optional, Enter to skip)")?;
    }

    if !cli.gui && !cli.console {
        cli.gui = prompt_yes_no("Hide console window? [y/N]")?;
    }

    Ok(())
}

fn prompt_path(label: &str) -> Result<PathBuf> {
    loop {
        let mut line = String::new();
        print!("{label}: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        let value = line.trim().trim_matches('"');
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }
}

fn prompt_optional_path(label: &str) -> Result<Option<PathBuf>> {
    let mut line = String::new();
    print!("{label}: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut line)?;
    let value = line.trim().trim_matches('"');
    Ok((!value.is_empty()).then(|| PathBuf::from(value)))
}

fn prompt_yes_no(label: &str) -> Result<bool> {
    let mut line = String::new();
    print!("{label}: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut line)?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "YES" | "Yes"))
}

fn default_output_path(input: &Path) -> PathBuf {
    let mut out = input.to_path_buf();
    out.set_extension("exe");
    out
}

fn ensure_exe_extension(mut path: PathBuf) -> PathBuf {
    if path.extension().and_then(|s| s.to_str()) != Some("exe") {
        path.set_extension("exe");
    }
    path
}

#[allow(dead_code)]
fn _keep_resource_directory_type(_: ResourceDirectory) {}
