#![cfg(feature = "test-fixtures")]

use std::fs;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const BINBOX: &str = env!("CARGO_BIN_EXE_binbox");

struct TestFixture {
    binbox: PathBuf,
    _temp_dir: tempfile::TempDir,
}

static FIXTURE: OnceLock<TestFixture> = OnceLock::new();

fn fixture() -> &'static TestFixture {
    FIXTURE.get_or_init(|| {
        let temp_dir = tempfile::tempdir().unwrap();
        let binbox = temp_dir.path().join("binbox");

        fs::copy(BINBOX, &binbox).unwrap();

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&binbox)
            .expect("Failed to open binbox for writing");
        file.seek(SeekFrom::End(0)).unwrap();
        // Write in place so ZIP offsets include the executable prefix.
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::<()>::default();
        zip.start_file("test-tool", options).unwrap();
        let mut tool_file = fs::File::open(env!("CARGO_BIN_EXE_test-tool")).unwrap();
        std::io::copy(&mut tool_file, &mut zip).expect("Failed to copy test tool into ZIP");
        zip.finish().unwrap();

        TestFixture {
            binbox,
            _temp_dir: temp_dir,
        }
    })
}

#[test]
fn test_list_tools() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = fixture();

    let result = Command::new(&fixture.binbox).arg("--list").output()?;

    assert!(
        result.status.success(),
        "Bundled tool failed: {}\nstdout: {}\nstderr: {}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("test-tool"));
    Ok(())
}

#[test]
fn test_execute_bundled_tool() -> Result<(), Box<dyn std::error::Error>> {
    let result = Command::new(&fixture().binbox)
        .arg("test-tool")
        .args(["hello", "world"])
        .output()?;
    assert!(
        result.status.success(),
        "Bundled tool failed: {}\nstdout: {}\nstderr: {}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    );
    assert_eq!(result.stdout, b"hello world\n");
    assert!(result.stderr.is_empty());

    Ok(())
}
