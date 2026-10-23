# binbox

**BusyBox-inspired, bring-your-own-binaries toolbox.**

binbox is a minimal Rust executable that runs tools bundled in a ZIP archive appended to itself.

No built-in tools, no configuration, no dependencies at runtime. Just one binary containing exactly the tools you need.

## Usage

```bash
binbox --list
binbox kubectl get pods
binbox helm version
```

## How it works

binbox uses a simple trick: ZIP archives can be appended to an executable without preventing it from running.

At runtime, binbox:
- Opens its own executable and reads the appended ZIP archive.
- Locates the requested tool within the archive.
- Extracts it into a private temporary directory.
- Executes it, forwarding arguments and standard I/O.
- Removes the temporary directory when execution finishes.

The tools are extracted on demand, not all at once.

## Building

Requires a Rust toolchain.

```shell
cargo build --release
```

The executable is available at target/release/binbox.

## Bundling tools
Create a ZIP archive containing the executables you want, then append it to binbox.

For example, using kubectl and helm:

```shell
zip tools.zip kubectl helm
cat target/release/binbox tools.zip > mybox
zip -A mybox
chmod +x mybox
```

That's it! `mybox` is now a self-contained executable containing the tools you bundled.

The `zip -A` step adjusts ZIP offsets to account for the executable prefix.
Without it, the ZIP reader can mistake signature bytes inside the executable for archive metadata.

**Note:** The bundled executables must be compatible with the target operating system and architecture. Any external shared-library dependencies must also be available at runtime.

## Testing

```shell
cargo test --features test-fixtures
```

The `test-fixtures` feature builds the echo-like test tool and enables the bundled-tool integration tests.

## Status

🚧 Early development. A project for learning Rust.

## Known Limitations

Only native executables compatible with the host platform are supported.

Only the requested executable is extracted; associated files or libraries are not bundled automatically.

Temporary files may remain if binbox is terminated abruptly by a signal.

Some macOS system executables cannot be executed after relocation due to Apple's code-signing restrictions.

## License

Licensed under either of Apache License, Version 2.0 or MIT license,
at your option. See the LICENSE-APACHE and LICENSE-MIT files for details.
