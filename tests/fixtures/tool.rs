use std::{
    io::{self, Write},
    os::unix::ffi::OsStrExt,
};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    for (index, arg) in std::env::args_os().skip(1).enumerate() {
        if index > 0 {
            stdout.write_all(b" ")?;
        }
        stdout.write_all(arg.as_bytes())?;
    }
    stdout.write_all(b"\n")
}
