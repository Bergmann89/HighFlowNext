#![allow(missing_docs)]

use std::io::stdin;
use std::mem::take;
use std::num::ParseIntError;

use anyhow::{Context, Result};

use crc::{Crc, CRC_16_USB};
use hidapi::HidApi;

const VID: u16 = 0x0C70;
const PID: u16 = 0xF012;

fn main() -> Result<()> {
    let api = HidApi::new()?;
    let dev = api
        .open(VID, PID)
        .with_context(|| format!("Open VID={:#06x} PID={:#06x}", VID, PID))?;

    let crc = Crc::<u16>::new(&CRC_16_USB);

    let stdin = stdin();
    let mut line = String::new();
    let mut data_old = Vec::new();
    let mut offset = 0;
    let mut length = usize::MAX;
    while stdin.read_line(&mut line)? > 0 {
        let s = line.trim().to_owned();
        line.clear();
        let report_id = if s == "e" || s == "exit" {
            break;
        } else if s == "r" || s == "reset" {
            offset = 0;
            length = usize::MAX;
            continue;
        } else if let Some(s) = s.strip_prefix("o") {
            match parse_usize(s) {
                Ok(x) => {
                    offset = x;
                    println!("Offset set to {offset}");
                }
                Err(error) => println!("Invalid offset {s}: {error}"),
            }
            continue;
        } else if let Some(s) = s.strip_prefix("l") {
            match parse_usize(s) {
                Ok(x) => {
                    length = x;
                    println!("Length set to {length}");
                }
                Err(error) => println!("Invalid length {s}: {error}"),
            }
            continue;
        } else if s.is_empty() || s == "c" {
            0x03
        } else {
            println!("Unknown command: {s}");

            continue;
        };

        println!("Send Request");
        let mut buffer = [0u8; 0x1000];
        buffer[0] = report_id;
        let bytes_recv = dev
            .get_feature_report(&mut buffer[..])
            .context("Unable to get feature report")?;

        let o = bytes_recv.min(offset);
        let l = (bytes_recv).min(o.saturating_add(length));

        let data_new = &buffer[o..l];
        println!("{:04X}", crc.checksum(&data_new[1..data_new.len() - 2]));

        print_diff(0, &data_old, data_new);

        data_new.clone_into(&mut data_old);
    }

    Ok(())
}

fn parse_usize(s: &str) -> Result<usize, ParseIntError> {
    let s = s.trim();
    let (radix, digits) = if let Some(x) = s.strip_prefix("0x").or(s.strip_prefix("0X")) {
        (16, x)
    } else if let Some(x) = s.strip_prefix("0b").or(s.strip_prefix("0B")) {
        (2, x)
    } else if let Some(x) = s.strip_prefix("0o").or(s.strip_prefix("0O")) {
        (8, x)
    } else {
        (10, s)
    };
    let clean = digits.replace('_', "");

    usize::from_str_radix(&clean, radix)
}

#[allow(clippy::too_many_lines)]
fn print_diff(offset: usize, old: &[u8], new: &[u8]) {
    use std::fmt::Write;

    const BYTES_PER_LINE: usize = 16;
    const BYTES_PER_GROUP: usize = 8;

    const HIGHLIGHT_SET: &str = "\x1b[4m";
    const HIGHLIGHT_RESET: &str = "\x1b[24m";

    const COLOR_OLD: &str = "\x1b[91m";
    const COLOR_NEW: &str = "\x1b[92m";
    const RESET_ALL: &str = "\x1b[0m";

    let mut i = 0;
    let mut line = 0;
    let mut eof = false;
    let mut highlight = false;

    let mut old = old.iter().fuse();
    let mut new = new.iter().fuse();

    let mut hex_old = String::new();
    let mut hex_new = String::new();

    let mut ascii_old = String::new();
    let mut ascii_new = String::new();

    println!("            00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F");

    while !eof {
        let old = old.next();
        let new = new.next();

        let diff = matches!((old, new), (Some(old), Some(new)) if old != new);

        let (fmt_before, fmt_after) = match (highlight, diff) {
            (false, false) | (true, true) => ("", ""),
            (false, true) => {
                highlight = true;

                (HIGHLIGHT_SET, "")
            }
            (true, false) => {
                highlight = false;

                ("", HIGHLIGHT_RESET)
            }
        };

        if let Some(old) = old {
            let _ = write!(hex_old, "{fmt_after} {fmt_before}{old:02X}");
            let _ = write!(
                ascii_old,
                "{fmt_after}{fmt_before}{}",
                if old.is_ascii_graphic() {
                    *old as char
                } else {
                    '.'
                }
            );
        } else {
            let _ = write!(hex_old, "{fmt_after}");
            let _ = write!(ascii_old, "{fmt_after}");
        }

        if let Some(new) = new {
            let _ = write!(hex_new, "{fmt_after} {fmt_before}{new:02X}");
            let _ = write!(
                ascii_new,
                "{fmt_after}{fmt_before}{}",
                if new.is_ascii_graphic() {
                    *new as char
                } else {
                    '.'
                }
            );
        } else {
            let _ = write!(hex_new, "{fmt_after}");
            let _ = write!(ascii_new, "{fmt_after}");
        }

        i += 1;

        eof = old.is_none() && new.is_none();

        let eol = i % BYTES_PER_LINE == 0;
        let eog = i % BYTES_PER_GROUP == 0;

        if eof || eol {
            let addr = offset + line;

            let fill = if eol {
                0
            } else {
                line + BYTES_PER_LINE - i + 1
            };
            let fill = "   ".repeat(fill);

            if take(&mut highlight) {
                let _ = write!(hex_old, "{HIGHLIGHT_RESET}");
                let _ = write!(ascii_old, "{HIGHLIGHT_RESET}");

                let _ = write!(hex_new, "{HIGHLIGHT_RESET}");
                let _ = write!(ascii_new, "{HIGHLIGHT_RESET}");
            }

            if hex_old == hex_new {
                println!(" {addr:08X}: {hex_old}{fill}  {ascii_old}");
            } else {
                if !hex_old.trim().is_empty() {
                    println!("{COLOR_OLD}-{addr:08X}: {hex_old}{fill}  {ascii_old}{RESET_ALL}");
                }
                if !hex_new.trim().is_empty() {
                    println!("{COLOR_NEW}+{addr:08X}: {hex_new}{fill}  {ascii_new}{RESET_ALL}");
                }
            }

            line = i;

            hex_old.clear();
            ascii_old.clear();

            hex_new.clear();
            ascii_new.clear();
        } else if eog {
            hex_old.push(' ');
            ascii_old.push(' ');

            hex_new.push(' ');
            ascii_new.push(' ');
        }
    }
}
