# BootUEFI

This is a tool for running and testing Rust UEFI project.

BootUEFI is modified from [bootimage](https://github.com/rust-osdev/bootimage)

## Install

```shell
cargo install bootuefi
```

## Usage

First you should install `cargo-xbuild`.

Then set `bootuefi` as a custom runner in `.cargo/config`:

```toml
[build]
target = "x86_64-unknown-uefi"

[target.x86_64-unknown-uefi]
runner = "bootuefi"
```

You can run your rust UEFI application through `cargo xrun` or test it throught `cargo xtest`.

## Configuration

Configuration is done through a through a `[package.metadata.bootuefi]` table in the `Cargo.toml` of your project. The following options are available:

```toml
[package.metadata.bootuefi]

# The command to run qemu.
# Set this to an absolute path if your qemu is not in your PATH
qemu = "qemu-system-x86_64"

# The Path to UEFI firmware
bios = "OVMF.fd"

# Additional arguments passed to qemu for non-test executables
run-args = []

# Additional arguments passed to qemu for test executables
test-args = []

# Don't use default arguments for qemu
default-args = true

# An exit code that should be considered as success for test executables
test-success-exit-code = 0

# The timeout for running a test
test-timeout = 300
```

Default arguments for qemu:

```rust
// Disable default devices. QEMU by defaults enables a ton of devices which slow down boot.
    "-nodefaults",
// Use a modern machine, with acceleration if possible.
    "-machine", "q35,accel=kvm:tcg",
// A standard VGA card with Bochs VBE extensions.
    "-vga", "std",
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
