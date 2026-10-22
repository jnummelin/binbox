use std::{
    ffi::OsString,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    process::ExitCode,
};

const USAGE: &str = "Usage: binbox [--list | <command> [args...]]";
const LIST_FLAG: &str = "--list";
const HELP_FLAG: &str = "--help";

fn main() -> ExitCode {
    let args = std::env::args_os().collect::<Vec<_>>();

    let flag: Option<&OsString> = args.get(1);

    match flag {
        None => {
            eprintln!("{}", USAGE);
            ExitCode::FAILURE
        }
        Some(flag) if flag == HELP_FLAG => {
            println!("{}", USAGE);
            ExitCode::SUCCESS
        }
        Some(flag) if flag == LIST_FLAG => {
            // List available tools
            if let Err(err) = list_tools() {
                eprintln!("Failed to list tools: {}", err);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(tool_name) => match execute_tool(tool_name, &args[2..]) {
            Ok(status) => match status.code() {
                Some(code) => ExitCode::from(code as u8),
                None => ExitCode::FAILURE,
            },
            Err(err) => {
                eprintln!("Failed to execute tool: {}", err);
                ExitCode::FAILURE
            }
        },
    }
}

fn open_self() -> std::io::Result<std::fs::File> {
    let self_path = std::env::current_exe()?;
    std::fs::File::open(self_path)
}

fn _self_size() -> std::io::Result<u64> {
    let file = open_self()?;
    let meta = file.metadata()?;
    Ok(meta.len())
}

fn list_tools() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(open_self()?)?;
    println!("Available tools:");
    // Iterate through the entries in the zip archive and list them
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.is_dir() {
            continue;
        }
        let name = file.name()?;
        println!("{}", name);
    }
    Ok(())
}

fn extract_tool(
    tool_name: &OsString,
    tmp_dir: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(open_self()?)?;
    let name = tool_name.to_str().ok_or("Tool name must be valid UTF-8")?;

    let mut file = archive.by_name(name)?;
    if file.is_dir() {
        return Err("Tool name must not be a directory".into());
    }

    let tmp_path = tmp_dir.join(tool_name);
    // If we use like utils/jq, create intermediate directories
    if let Some(parent) = tmp_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = std::fs::File::create(&tmp_path)?;
    std::io::copy(&mut file, &mut out)?;
    drop(out);
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o700))?;
    Ok(tmp_path)
}

fn execute_tool(
    tool_name: &OsString,
    args: &[OsString],
) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
    validate_tool_name(tool_name)?;
    let tmp_dir = tempfile::Builder::new().prefix("binbox-").tempdir()?;

    let tool_path = extract_tool(tool_name, tmp_dir.path())?;

    Ok(std::process::Command::new(tool_path).args(args).status()?)
}

fn validate_tool_name(tool_name: &OsString) -> Result<(), Box<dyn std::error::Error>> {
    if tool_name.is_empty() {
        return Err("Tool name must not be empty".into());
    }

    let bytes = tool_name.as_bytes();
    if bytes
        .split(|&b| b == b'/')
        .any(|part| part.is_empty() || part == b"." || part == b".." || part.contains(&b'\\'))
    {
        return Err("Tool name must not contain empty or special path components".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tool_name() {
        let cases = [
            ("kubectl", true),
            ("kubernetes/kubectl", true),
            ("utils/jq", true),
            ("../kubectl", false),
            ("/tmp/evil", false),
            ("foo/../../bar", false),
            ("", false),
            ("./foo", false),
            ("foo//bar", false),
            ("foo/", false),
            (".", false),
            ("..", false),
            ("foo/bar/baz", true),
            ("my-tool_v1.2", true),
            ("foo\\bar", false),
        ];

        for (tool, expected) in cases {
            let result = validate_tool_name(&OsString::from(tool));
            assert_eq!(
                result.is_ok(),
                expected,
                "Unexpected validation result for {tool:?}"
            );
        }
    }
}
