use std::collections::HashMap;

pub const LOAD_ADDRESS: u16 = 0x0600;

pub struct AssemblyImage {
    pub load_address: u16,
    pub entrypoint: u16,
    pub bytes: Vec<u8>,
    pub read_only: Option<(usize, usize)>,
}

pub fn assemble_image(source: &str) -> Result<AssemblyImage, String> {
    if source.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("TITLE") || line.starts_with("SEARCH")
    }) {
        return crate::macro_assembler::assemble(source);
    }

    let (load_address, bytes) = assemble(source)?;
    Ok(AssemblyImage {
        load_address,
        entrypoint: load_address,
        bytes,
        read_only: None,
    })
}

#[derive(Debug, Clone)]
struct Line {
    number: usize,
    pc: u16,
    mnemonic: String,
    operand: Option<String>,
}

pub fn assemble(source: &str) -> Result<(u16, Vec<u8>), String> {
    let mut labels = HashMap::<String, u16>::new();
    let mut lines = Vec::new();
    let mut pc = LOAD_ADDRESS;

    for (index, raw) in source.lines().enumerate() {
        let clean = raw.split(';').next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }
        let mut rest = clean;
        if let Some((label, tail)) = clean.split_once(':').filter(|(label, _)| {
            !label.trim().is_empty()
                && !label.chars().any(char::is_whitespace)
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }) {
            let label = label.trim().to_ascii_uppercase();
            if labels.insert(label.clone(), pc).is_some() {
                return Err(format!("line {}: duplicate label {label}", index + 1));
            }
            rest = tail.trim();
            if rest.is_empty() {
                continue;
            }
        }
        let mut parts = rest.splitn(2, char::is_whitespace);
        let mnemonic = parts.next().unwrap().to_ascii_uppercase();
        let operand = parts
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let size = instruction_size(&mnemonic, operand.as_deref())
            .map_err(|e| format!("line {}: {e}", index + 1))?;
        lines.push(Line {
            number: index + 1,
            pc,
            mnemonic,
            operand,
        });
        pc = pc.wrapping_add(size as u16);
    }

    let mut bytes = Vec::new();
    for line in lines {
        let encoded = encode(&line, &labels).map_err(|e| format!("line {}: {e}", line.number))?;
        bytes.extend(encoded);
    }
    Ok((LOAD_ADDRESS, bytes))
}

fn instruction_size(mnemonic: &str, operand: Option<&str>) -> Result<usize, String> {
    use Mnemonic::*;
    match parse_mnemonic(mnemonic)? {
        Brk | Inx | Dex | Nop | Clc => no_operand(operand),
        Bne | Beq => required(operand).map(|_| 2),
        Jmp => required(operand).map(|_| 3),
        Lda | Sta => {
            let op = required(operand)?;
            if op.starts_with('#') {
                return if mnemonic == "LDA" {
                    Ok(2)
                } else {
                    Err("STA has no immediate mode".into())
                };
            }
            if op.to_ascii_uppercase().ends_with(",X") {
                return if mnemonic == "LDA" {
                    Ok(2)
                } else {
                    Err("indexed STA is not implemented".into())
                };
            }
            let width = parse_number_or_label(op)
                .map(|(_, width)| width)
                .unwrap_or(4);
            Ok(if width > 2 { 3 } else { 2 })
        }
        Ldx | Ldy | Adc | Cmp => {
            let op = required(operand)?;
            if !op.starts_with('#') {
                return Err(format!(
                    "{mnemonic} currently requires immediate mode (#value)"
                ));
            }
            Ok(2)
        }
        Stx => {
            required(operand)?;
            Ok(2)
        }
    }
}

#[derive(Clone, Copy)]
enum Mnemonic {
    Lda,
    Ldx,
    Ldy,
    Sta,
    Stx,
    Adc,
    Cmp,
    Bne,
    Beq,
    Jmp,
    Brk,
    Inx,
    Dex,
    Nop,
    Clc,
}

fn parse_mnemonic(name: &str) -> Result<Mnemonic, String> {
    match name {
        "LDA" => Ok(Mnemonic::Lda),
        "LDX" => Ok(Mnemonic::Ldx),
        "LDY" => Ok(Mnemonic::Ldy),
        "STA" => Ok(Mnemonic::Sta),
        "STX" => Ok(Mnemonic::Stx),
        "ADC" => Ok(Mnemonic::Adc),
        "CMP" => Ok(Mnemonic::Cmp),
        "BNE" => Ok(Mnemonic::Bne),
        "BEQ" => Ok(Mnemonic::Beq),
        "JMP" => Ok(Mnemonic::Jmp),
        "BRK" => Ok(Mnemonic::Brk),
        "INX" => Ok(Mnemonic::Inx),
        "DEX" => Ok(Mnemonic::Dex),
        "NOP" => Ok(Mnemonic::Nop),
        "CLC" => Ok(Mnemonic::Clc),
        _ => Err(format!("unsupported instruction {name}")),
    }
}

fn required(operand: Option<&str>) -> Result<&str, String> {
    operand.ok_or_else(|| "expected an operand".into())
}

fn no_operand(operand: Option<&str>) -> Result<usize, String> {
    if operand.is_some() {
        Err("instruction takes no operand".into())
    } else {
        Ok(1)
    }
}

fn encode(line: &Line, labels: &HashMap<String, u16>) -> Result<Vec<u8>, String> {
    let m = parse_mnemonic(&line.mnemonic)?;
    let op = line.operand.as_deref();
    let bare = op.map(|s| s.trim_start_matches('#').trim());
    let one = |code, value: u16| -> Result<Vec<u8>, String> {
        if value > 0xFF {
            Err("operand must fit in one byte".into())
        } else {
            Ok(vec![code, value as u8])
        }
    };
    let word = |code, value: u16| vec![code, value as u8, (value >> 8) as u8];
    Ok(match m {
        Mnemonic::Lda => {
            let value_str = required(op)?;
            if value_str.starts_with('#') {
                one(0xA9, resolve(value_str.trim_start_matches('#'), labels)?)?
            } else if value_str.to_ascii_uppercase().ends_with(",X") {
                one(
                    0xB5,
                    resolve(value_str[..value_str.len() - 2].trim(), labels)?,
                )?
            } else {
                let value = resolve(value_str, labels)?;
                let width = parse_number_or_label(value_str)
                    .map(|(_, w)| w)
                    .unwrap_or(4);
                if width > 2 {
                    word(0xAD, value)
                } else {
                    one(0xA5, value)?
                }
            }
        }
        Mnemonic::Ldx => one(0xA2, resolve(required(bare)?, labels)?)?,
        Mnemonic::Ldy => one(0xA0, resolve(required(bare)?, labels)?)?,
        Mnemonic::Sta => {
            let text = required(op)?;
            let value = resolve(text, labels)?;
            let width = parse_number_or_label(text).map(|(_, w)| w).unwrap_or(4);
            if width > 2 {
                word(0x8D, value)
            } else {
                one(0x85, value)?
            }
        }
        Mnemonic::Stx => one(0x86, resolve(required(op)?, labels)?)?,
        Mnemonic::Adc => one(0x69, resolve(required(bare)?, labels)?)?,
        Mnemonic::Cmp => one(0xC9, resolve(required(bare)?, labels)?)?,
        Mnemonic::Jmp => word(0x4C, resolve(required(op)?, labels)?),
        Mnemonic::Bne | Mnemonic::Beq => {
            let target = resolve(required(op)?, labels)?;
            let delta = target as i32 - (line.pc as i32 + 2);
            if !(-128..=127).contains(&delta) {
                return Err("branch target is out of range".into());
            }
            vec![
                if matches!(m, Mnemonic::Bne) {
                    0xD0
                } else {
                    0xF0
                },
                delta as i8 as u8,
            ]
        }
        Mnemonic::Brk => vec![0x00],
        Mnemonic::Inx => vec![0xE8],
        Mnemonic::Dex => vec![0xCA],
        Mnemonic::Nop => vec![0xEA],
        Mnemonic::Clc => vec![0x18],
    })
}

fn resolve(text: &str, labels: &HashMap<String, u16>) -> Result<u16, String> {
    if let Ok((value, _)) = parse_number_or_label(text) {
        return Ok(value);
    }
    labels
        .get(&text.trim().to_ascii_uppercase())
        .copied()
        .ok_or_else(|| format!("unknown label or invalid value {text}"))
}

fn parse_number_or_label(text: &str) -> Result<(u16, usize), String> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix('$') {
        let width = hex.len();
        return u16::from_str_radix(hex, 16)
            .map(|v| (v, width))
            .map_err(|_| format!("invalid hex value {text}"));
    }
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return u16::from_str_radix(hex, 16)
            .map(|v| (v, hex.len()))
            .map_err(|_| format!("invalid value {text}"));
    }
    if text.starts_with('\'') && text.ends_with('\'') && text.len() == 3 {
        return Ok((text.as_bytes()[1] as u16, 2));
    }
    text.parse::<u16>()
        .map(|v| (v, 2))
        .map_err(|_| format!("invalid number {text}"))
}
