# The Great Moomba

Modern and fast FFNx configurator

```sh
# Build everything and run GUI
cargo run --release
# Build and run CLI only
cargo run --release --bin mmb
```

## Compile ff8_launcher on Linux

On Linux we need to cross-compile `ff8_launcher` for Windows target

```sh
# See https://github.com/BenjaminRi/winresource?tab=readme-ov-file#cross-compiling-on-a-non-windows-os
sudo apt-get install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --bin ff8_launcher --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/ff8_launcher.exe target/release/
```
