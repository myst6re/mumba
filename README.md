# The Yellow Mumba

Modern and fast FFNx configurator

Features:
- Auto install and update FFNx
- Configuration interface for FFNx
- The game can be started without the launcher graphical interface (useful for Big Picture mode)
- Game installation is not modified, except the game launcher on Steam
- Compatible with the original PC release (2000) and the Steam rerelease (2013)
- Windows and Linux support

## How to compile

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

## Create installer

### Windows

Download Wix **3** from here: https://github.com/wixtoolset/wix3/releases

```sh
cargo install cargo-wix
cd gui
cargo wix --nocapture --no-build -p mumba
```

The MSI file is produced in `target/wix/`.

### Debian-like linuxes

```sh
apt-get install liblzma-dev dpkg-dev
cargo install cargo-deb
cargo deb -p mumba
```

The DEB file is produced in `target/debian/`.

## Thanks

- Tokyoship: Initial author of [gamepad_layout.svg](https://commons.wikimedia.org/w/index.php?title=File:Dualshock_4_Layout.svg&oldid=769091332)
