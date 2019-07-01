use failure::{format_err, Error};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml::Value;

#[derive(Default)]
pub struct Config {
    pub qemu: Option<String>,
    pub bios: Option<String>,
    pub run_args: Option<Vec<String>>,
    pub test_args: Option<Vec<String>>,
    pub default_args: Option<bool>,
    pub test_success_exit_code: Option<i32>,
    pub test_timeout: Option<u32>,
}

pub struct Profile {
    pub qemu: String,
    pub args: Vec<String>,
    pub test_success_exit_code: i32,
    pub test_timeout: u32,
}

impl Config {
    pub fn read() -> Result<Config, Error> {
        let manifest_path = locate_cargo_manifest::locate_manifest()
            .map_err(|e| format_err!("Can't find Cargo.toml: {:?}", e))?;
        let mut mainfest_context = String::new();
        File::open(manifest_path)
            .map_err(|e| format_err!("Failed to open Cargo.toml: {}", e))?
            .read_to_string(&mut mainfest_context)
            .map_err(|e| format_err!("Failed to read Cargo.toml: {}", e))?;
        let cargo_toml = mainfest_context
            .parse::<Value>()
            .map_err(|e| format_err!("Failed to parse Cargo.toml: {}", e))?;
        let metadata = match cargo_toml
            .get("package")
            .and_then(|table| table.get("metadata"))
            .and_then(|table| table.get("bootuefi"))
        {
            None => return Ok(Default::default()),
            Some(meta) => meta
                .as_table()
                .ok_or(format_err!("package.metadata.bootuefi is invalid"))?,
        };
        let mut config: Config = Default::default();
        for (key, value) in metadata {
            match (key.as_str(), value.clone()) {
                ("qemu", Value::String(s)) => config.qemu = Some(s),
                ("bios", Value::String(s)) => config.bios = Some(s),
                ("default-args", Value::Boolean(b)) => config.default_args = Some(b),
                ("test-timeout", Value::Integer(i)) => {
                    if i < 0 {
                        return Err(format_err!("test-timeout must not be negative"));
                    } else {
                        config.test_timeout = Some(i as u32);
                    }
                }
                ("test-success-exit-code", Value::Integer(i)) => {
                    config.test_success_exit_code = Some(i as i32);
                }
                ("run-args", Value::Array(a)) => {
                    let mut args = Vec::new();
                    for v in a {
                        match v {
                            Value::String(s) => args.push(s),
                            _ => return Err(format_err!("run-args has non string element: {}", v)),
                        }
                    }
                    config.run_args = Some(args);
                }
                ("test-args", Value::Array(a)) => {
                    let mut args = Vec::new();
                    for v in a {
                        match v {
                            Value::String(s) => args.push(s),
                            _ => {
                                return Err(format_err!("test-args has non string element: {}", v))
                            }
                        }
                    }
                    config.test_args = Some(args);
                }
                (key, value) => {
                    return Err(format_err!(
                        "unexpect key `{}` with value `{}` in `package.metadata.bootuefi`",
                        key,
                        value
                    ))
                }
            }
        }
        Ok(config)
    }

    pub fn build_profile(self, is_test: bool, esp: &Path) -> Result<Profile, Error> {
        let qemu = self.qemu.unwrap_or("qemu-system-x86_64".into());
        let bios = self.bios.unwrap_or("OVMF.fd".into());
        let mut args = if is_test {
            self.test_args.unwrap_or(Vec::new())
        } else {
            self.run_args.unwrap_or(Vec::new())
        };
        if self.default_args.unwrap_or(true) {
            args.extend(
                vec![
                    // Disable default devices.
                    // QEMU by defaults enables a ton of devices which slow down boot.
                    "-nodefaults",
                    // Use a modern machine, with acceleration if possible.
                    "-machine",
                    "q35,accel=kvm:tcg",
                    // A standard VGA card with Bochs VBE extensions.
                    "-vga",
                    "std",
                    // Connect the serial port to the host. OVMF is kind enough to connect
                    // the UEFI stdout and stdin to that port too.
                    "-serial",
                    "stdio",
                ]
                .into_iter()
                .map(|x| x.to_owned()),
            );
        }
        args.push("-bios".into());
        args.push(bios);
        args.push("-drive".into());
        args.push(format!("format=raw,file=fat:rw:{}", esp.display()));
        Ok(Profile {
            qemu,
            args,
            test_success_exit_code: self.test_success_exit_code.unwrap_or(0),
            test_timeout: self.test_timeout.unwrap_or(300),
        })
    }
}
