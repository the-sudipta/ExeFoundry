use std::{
    env, fs,
    path::Path,
    process::{Command, ExitCode},
};

use exefoundry::payload::extract_package;
use tempfile::Builder;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> anyhow::Result<u8> {
    let current_exe = env::current_exe()?;
    let data = fs::read(&current_exe)?;
    let payload = extract_package(&data)?;

    let temp = Builder::new().prefix("exefoundry_").tempdir()?;
    let bat = temp.path().join("payload.bat");
    fs::write(&bat, payload.bat)?;

    let command_line = build_cmd_line(&bat, env::args().skip(1));
    let mut command = Command::new("cmd.exe");
    command.arg("/C");
    append_cmd_line(&mut command, &command_line);
    let status = command.current_dir(temp.path()).status()?;

    Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
}

#[cfg(windows)]
fn append_cmd_line(command: &mut Command, command_line: &str) {
    command.raw_arg(command_line);
}

#[cfg(not(windows))]
fn append_cmd_line(command: &mut Command, command_line: &str) {
    command.arg(command_line);
}

fn build_cmd_line<I, S>(bat: &Path, args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut line = format!("call {}", quote_for_cmd(&bat.display().to_string()));
    for arg in args {
        line.push(' ');
        line.push_str(&quote_for_cmd(arg.as_ref()));
    }
    line
}

fn quote_for_cmd(value: &str) -> String {
    let escaped = value.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_command_line_with_quoted_args() {
        let line = build_cmd_line(
            Path::new(r"C:\Temp Folder\payload.bat"),
            ["one", "two words"],
        );
        assert_eq!(
            line,
            r#"call "C:\Temp Folder\payload.bat" "one" "two words""#
        );
    }
}
