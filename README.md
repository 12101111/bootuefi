# BootUEFI

This is a tool for running and testing Rust UEFI project.

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
// Connect the serial port to the host. OVMF is kind enough to connect the UEFI stdout and stdin to that port too.
    "-serial", "stdio",
```
