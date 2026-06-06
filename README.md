<p align="center">
  <img src="scripts/icon/Bat%20to%20EXE%20icon%20round%20corner.png" alt="ExeFoundry logo" width="120">
</p>

# ExeFoundry

ExeFoundry converts a Windows Batch file (`.bat`) into a single portable Windows executable (`.exe`). The new implementation is written in Rust, so the converter itself can be released as one standalone binary for Windows, macOS, and Linux.

The generated file is still a Windows `.exe`, because `.bat` scripts are Windows scripts. Running the converter on macOS or Linux is useful for packaging Windows utilities from those systems.

## Website and Playground

The repository ships with a GitHub Pages website in `docs/`.

- Website: <https://the-sudipta.github.io/ExeFoundry/>
- Playground: <https://the-sudipta.github.io/ExeFoundry/#playground>
- Releases: <https://github.com/the-sudipta/ExeFoundry/releases/latest>

The playground helps users pick their platform binary, input BAT, output EXE, icon, and console/GUI mode, then generates the exact command to run.

## What It Does

- Embeds the entire `.bat` payload inside the generated `.exe`.
- Embeds ExeFoundry's built-in icon by default, so the generated EXE does not need a separate icon file.
- Optionally embeds your own `.ico`, `.png`, or other supported image instead.
- Preserves argument forwarding to the original batch script.
- Supports console output by default, or GUI/no-console mode with `--gui`.
- Does not require PowerShell, `csc.exe`, .NET Framework, or admin rights for normal use.

The old PowerShell/C# scripts are still in `scripts/` for reference, but the Rust binary is now the primary tool.

## Quick Start

```powershell
# Windows
.\exefoundry-windows-x64.exe --input ".\scripts\tool.bat" --output ".\Tool.exe"

# With a custom icon embedded into the output EXE instead of the bundled ExeFoundry icon
.\exefoundry-windows-x64.exe --input ".\scripts\tool.bat" --output ".\Tool.exe" --icon ".\scripts\icon\bat_to_exe.ico"

# GUI app, no console window
.\exefoundry-windows-x64.exe --input ".\scripts\tool.bat" --output ".\Tool.exe" --icon ".\scripts\icon\bat_to_exe.ico" --gui
```

On macOS or Linux, use the matching `exefoundry-macos` or `exefoundry-linux-x64` release binary with the same flags. The output remains a Windows `.exe`.

## Interactive Mode

Run ExeFoundry without arguments, or pass `--interactive`, and it will ask for the BAT path, output path, icon path, and console mode.

```powershell
.\exefoundry-windows-x64.exe --interactive
```

## Command-Line Options

| Option | Description |
| --- | --- |
| `-i, --input <BAT>` | Source `.bat` file to package. |
| `-o, --output <EXE>` | Output `.exe` path. Defaults to the input name with `.exe`. |
| `--icon <ICO_OR_IMAGE>` | Optional custom icon/image to embed into the output EXE. If omitted, ExeFoundry embeds its bundled icon. |
| `--gui`, `--win-exe` | Build the output as a GUI app with no console window. |
| `--console` | Build the output as a console app. This is the default. |
| `--interactive` | Prompt for missing values. |
| `--template <EXE>` | Development escape hatch for using an external Windows runner template. Release binaries already embed this. |

## Build From Source

For a local development build, build the Windows runner template first, then build the converter with the template embedded:

```powershell
cargo build --release --bin exefoundry-runner
$env:EXEFOUNDRY_RUNNER_TEMPLATE = (Resolve-Path ".\target\release\exefoundry-runner.exe").Path
cargo build --release --bin exefoundry
```

You can also run a development converter without embedding the template:

```powershell
cargo run --bin exefoundry -- --input ".\scripts\tool.bat" --output ".\Tool.exe" --template ".\target\release\exefoundry-runner.exe"
```

## Releases and Versioning

The GitHub workflow builds:

- `exefoundry-windows-x64.exe`
- `exefoundry-linux-x64`
- `exefoundry-macos`
- SHA-256 checksum files for each binary

Every push to `main` compiles and uploads build artifacts for Windows, Linux, and macOS. Push a tag like `v1.1.0` to create a GitHub Release automatically. The crate version in `Cargo.toml` should be updated to the same version before tagging.

GitHub Pages is deployed from `docs/` on every push to `main`.

## Documentation

- PDF note: `docs/ExeFoundry Portable BAT to EXE Converter for Shareable Windows Utilities.pdf`
- Citation metadata: `CITATION.cff`

## License

MIT
