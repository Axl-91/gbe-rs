//! Cartridge header parsing, used for the startup banner.

use std::fmt;

const TITLE_START: usize = 0x134;
const TITLE_END: usize = 0x143;
const CARTRIDGE_TYPE: usize = 0x147;
const ROM_SIZE: usize = 0x148;
const RAM_SIZE: usize = 0x149;
const HEADER_END: usize = 0x14F;

const KIB_PER_MIB: usize = 1024;

/// Human-readable summary of the cartridge header.
pub struct CartridgeInfo {
    title: String,
    cartridge_type: String,
    rom_kib: usize,
    ram: Option<String>,
}

impl CartridgeInfo {
    /// Parses the header. Returns `None` if the ROM is too small to have one.
    pub fn parse(rom: &[u8]) -> Option<Self> {
        if rom.len() <= HEADER_END {
            return None;
        }

        // The last title byte can hold the CGB flag (0x80/0xC0), which is not
        // ASCII graphic and gets filtered out here.
        let title: String = rom[TITLE_START..=TITLE_END]
            .iter()
            .take_while(|&&byte| byte != 0)
            .filter(|byte| byte.is_ascii_graphic() || **byte == b' ')
            .map(|&byte| char::from(byte))
            .collect();
        let title = title.trim();
        let title = if title.is_empty() { "UNTITLED" } else { title };

        let type_code = rom[CARTRIDGE_TYPE];

        let rom_kib = match rom[ROM_SIZE] {
            code @ 0..=8 => 32 << code,
            // Unknown code: fall back to the real file size.
            _ => rom.len() / 1024,
        };

        let ram = if matches!(type_code, 0x05 | 0x06) {
            // MBC2 has built-in RAM and the header reports 0 for it.
            Some("512 x 4 bits".to_string())
        } else {
            match rom[RAM_SIZE] {
                0x01 => Some(2),
                0x02 => Some(8),
                0x03 => Some(32),
                0x04 => Some(128),
                0x05 => Some(64),
                _ => None,
            }
            .map(format_kib)
        };

        Some(Self {
            title: title.to_string(),
            cartridge_type: cartridge_type_name(type_code),
            rom_kib,
            ram,
        })
    }
}

impl fmt::Display for CartridgeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NAME: {}\nTYPE: {}\nROM: {}",
            self.title,
            self.cartridge_type,
            format_kib(self.rom_kib)
        )?;

        if let Some(ram) = &self.ram {
            write!(f, "\nRAM: {ram}")?;
        }
        Ok(())
    }
}

fn format_kib(kib: usize) -> String {
    if kib >= KIB_PER_MIB && kib.is_multiple_of(KIB_PER_MIB) {
        format!("{} MiB", kib / KIB_PER_MIB)
    } else {
        format!("{kib} KiB")
    }
}

fn cartridge_type_name(code: u8) -> String {
    let name = match code {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+BATTERY",
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        0x1C => "MBC5+RUMBLE",
        0x1D => "MBC5+RUMBLE+RAM",
        0x1E => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xFC => "POCKET CAMERA",
        0xFD => "BANDAI TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+BATTERY",
        _ => return format!("UNKNOWN ({code:#04X})"),
    };
    name.to_string()
}
