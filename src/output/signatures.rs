use std::fmt::{self, Write};
use heck::{AsPascalCase, AsSnakeCase};
use super::{CodeWriter, Formatter, SignatureMap, slugify, zig_ident};

impl CodeWriter for SignatureMap {
    fn write_cs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("namespace CS2Dumper.Signatures", false, |fmt| {
            for (module_name, sigs) in self {
                writeln!(fmt, "// Module: {}", module_name)?;
                fmt.block(
                    &format!("public static class {}", AsPascalCase(slugify(module_name))),
                    false,
                    |fmt| {
                        for (name, sig) in sigs {
                            writeln!(fmt, "public const string {} = \"{}\";", name, sig)?;
                        }
                        Ok(())
                    },
                )?;
            }
            Ok(())
        })
    }

    fn write_hpp(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#pragma once\n")?;
        writeln!(fmt, "#include <cstddef>")?;
        writeln!(fmt, "#include <cstdint>\n")?;

        fmt.block("namespace cs2_dumper", false, |fmt| {
            fmt.block("namespace signatures", false, |fmt| {
                for (module_name, sigs) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    fmt.block(
                        &format!("namespace {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, sig) in sigs {
                                writeln!(fmt, "constexpr const char* {} = \"{}\";", name, sig)?;
                            }
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            })
        })
    }

    fn write_json(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str(&serde_json::to_string_pretty(self).unwrap())
    }

    fn write_rs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#![allow(non_upper_case_globals, unused)]\n")?;

        fmt.block("pub mod cs2_dumper", false, |fmt| {
            fmt.block("pub mod signatures", false, |fmt| {
                for (module_name, sigs) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    fmt.block(
                        &format!("pub mod {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, sig) in sigs {
                                writeln!(fmt, "pub const {}: &str = \"{}\";", name, sig)?;
                            }
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            })
        })
    }

    fn write_zig(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("pub const cs2_dumper = struct", true, |fmt| {
            fmt.block("pub const signatures = struct", true, |fmt| {
                for (module_name, sigs) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    let module_name = zig_ident(&AsSnakeCase(slugify(module_name)).to_string());
                    fmt.block(
                        &format!("pub const {} = struct", module_name),
                        true,
                        |fmt| {
                            for (name, sig) in sigs {
                                writeln!(
                                    fmt,
                                    "pub const {}: [:0]const u8 = \"{}\";",
                                    zig_ident(name),
                                    sig
                                )?;
                            }
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            })
        })
    }
                               }
