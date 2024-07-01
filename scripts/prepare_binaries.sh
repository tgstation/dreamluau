#!/bin/bash
set -euo pipefail

echo '==== Make sure cross is installed ===='
cargo install cross

echo '==== Linux build ====' # ------------------------------------------------
rustup target add i686-unknown-linux-gnu
env PKG_CONFIG_ALLOW_CROSS=1 \
	cargo build --release --target i686-unknown-linux-gnu

echo '==== Windows build ====' # ----------------------------------------------
cross build --release --target i686-pc-windows-gnu
# https://github.com/rust-lang/rust/issues/12859#issuecomment-62255275
# Most distros ship 32-bit toolchains with SJLJ unwinding, but for 32-bit Rust
# can only cross-compile targeting DWARF. All 64-bit toolchains use SEH, where
# there is no problem. One of two workarounds is required:

# Disable unwinding with with "-C panic=abort" instead. Without `catch_unwind`
# use in the Rust code or luck in the DreamDaemon runtime, panics already bring
# down the host process anyways.

# Use wine to run rust-mingw component distributed by Rust for pc-windows-gnu:
# wget https://static.rust-lang.org/dist/rust-mingw-nightly-i686-pc-windows-gnu.tar.gz
# tar xf rust-mingw-nightly-i686-pc-windows-gnu.tar.gz
# ./rust-mingw-nightly-i686-pc-windows-gnu/install.sh --prefix=$(rustc --print sysroot)

# Luckily we are taking the third workaround, which is using the cross projects i686
# windows toolchain docker which comes with an updated version of mingw that supports
# the DWARF based unwinding, freeing us from such earthly concerns

echo '==== Organize files ====' # ---------------------------------------------
DEST=target/publish/
rm -rf "$DEST"
mkdir -p "$DEST"
cp \
    target/i686-unknown-linux-gnu/release/libdreamluau.so \
    target/i686-pc-windows-gnu/release/dreamluau.dll \
    "$DEST"
echo "$DEST :"
ls -lh --color=auto "$DEST"