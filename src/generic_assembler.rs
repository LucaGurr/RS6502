use std::collections::{HashMap, HashSet};

use crate::assembler::AssemblyImage;
use crate::macro_assembler::{Mode, opcode};

#[derive(Clone)]
struct SourceLine {
    number: usize,
    text: String,
    indented: bool,
}

#[derive(Clone)]
struct Statement {
    line: SourceLine,
    operation: String,
    operand: Option<String>,
    pc: u16,
}

pub fn assemble(source: &str) -> Result<AssemblyImage, String> {
    let lines = source_lines(source);
    let mut previous_symbols = HashMap::new();
    let mut statements = Vec::new();
    let mut entrypoint = None;
    for _ in 0..8 {
        let (symbols, next_statements, next_entry) = layout(&lines, &previous_symbols)?;
        let stable = symbols == previous_symbols;
        previous_symbols = symbols;
        statements = next_statements;
        entrypoint = next_entry.or(entrypoint);
        if stable {
            break;
        }
    }

    let mut segments = Vec::new();
    let mut high_address = 0usize;
    let mut low_address = usize::MAX;
    for statement in &statements {
        let bytes = encode(statement, &previous_symbols)?;
        if bytes.is_empty() {
            continue;
        }
        let start = statement.pc as usize;
        let end = start
            .checked_add(bytes.len())
            .filter(|end| *end <= 65536)
            .ok_or_else(|| {
                format!(
                    "line {}: emitted data extends past address $FFFF",
                    statement.line.number
                )
            })?;
        low_address = low_address.min(start);
        high_address = high_address.max(end);
        segments.push((start, bytes));
    }
    if segments.is_empty() {
        return Err("source contains no emitted instructions or data".into());
    }

    let mut image = vec![0; high_address - low_address];
    let mut occupied = HashSet::new();
    for (start, bytes) in segments {
        for (offset, byte) in bytes.into_iter().enumerate() {
            let address = start + offset;
            if !occupied.insert(address) {
                return Err(format!("multiple statements emit data at ${address:04X}"));
            }
            image[address - low_address] = byte;
        }
    }

    let load_address = low_address as u16;
    Ok(AssemblyImage {
        load_address,
        entrypoint: entrypoint.unwrap_or(load_address),
        bytes: image,
        read_only: None,
    })
}

fn source_lines(source: &str) -> Vec<SourceLine> {
    let mut result = Vec::new();
    let mut pending = String::new();
    let mut pending_line = 0;
    let mut pending_indented = false;
    for (index, raw) in source.lines().enumerate() {
        let without_comment = strip_comment(raw);
        if without_comment.trim().is_empty() && pending.is_empty() {
            continue;
        }
        if pending.is_empty() {
            pending_line = index + 1;
            pending_indented = raw.starts_with(char::is_whitespace);
            pending.push_str(without_comment.trim());
        } else {
            pending.push(' ');
            pending.push_str(without_comment.trim());
        }
        if quote_is_closed(&pending) {
            result.push(SourceLine {
                number: pending_line,
                text: std::mem::take(&mut pending),
                indented: pending_indented,
            });
        }
    }
    if !pending.is_empty() {
        result.push(SourceLine {
            number: pending_line,
            text: pending,
            indented: pending_indented,
        });
    }
    result
}

fn strip_comment(text: &str) -> String {
    let mut quoted = false;
    for (index, character) in text.char_indices() {
        if character == '"' {
            quoted = !quoted;
        } else if character == ';' && !quoted {
            return text[..index].to_string();
        }
    }
    text.to_string()
}

fn quote_is_closed(text: &str) -> bool {
    let mut quoted = false;
    for character in text.chars() {
        if character == '"' {
            quoted = !quoted;
        }
    }
    !quoted
}

fn layout(
    lines: &[SourceLine],
    previous_symbols: &HashMap<String, i64>,
) -> Result<(HashMap<String, i64>, Vec<Statement>, Option<u16>), String> {
    let mut symbols = previous_symbols.clone();
    let mut labels_in_pass = HashSet::new();
    let mut statements = Vec::new();
    let mut pc = crate::assembler::LOAD_ADDRESS as u32;
    let mut entrypoint = None;

    for line in lines {
        let Some((label, operation, operand)) = parse_statement(line)? else {
            continue;
        };
        if operation == "=" {
            let name = label.ok_or_else(|| error(line, "invalid assignment"))?;
            if let Some(value) =
                eval(operand.as_deref().unwrap_or_default(), &symbols, pc as u16).ok()
            {
                symbols.insert(name, value);
            }
            continue;
        }
        if let Some(label) = &label {
            if !labels_in_pass.insert(label.clone()) {
                return Err(error(line, &format!("duplicate label {label}")));
            }
            symbols.insert(label.clone(), pc as i64);
        }

        match operation.as_str() {
            "*" | ".ORG" | "ORG" => {
                let address = eval(required(line, &operand)?, &symbols, pc as u16)
                    .map_err(|message| error(line, &message))?;
                pc = checked_address(line, address)?;
                continue;
            }
            "END" => {
                if let Some(expression) = operand.as_deref() {
                    if let Ok(value) = eval(expression, &symbols, pc as u16) {
                        entrypoint = Some(checked_address(line, value)? as u16);
                    }
                }
                continue;
            }
            _ => {}
        }

        let current_pc = checked_address(line, pc as i64)? as u16;
        let statement = Statement {
            line: line.clone(),
            operation: operation.clone(),
            operand: operand.clone(),
            pc: current_pc,
        };
        let size = statement_size(&statement, &symbols)?;
        if pc as usize + size > 65536 {
            return Err(error(line, "statement extends past address $FFFF"));
        }
        if is_instruction(&operation) && entrypoint.is_none() {
            entrypoint = Some(current_pc);
        }
        pc += size as u32;
        statements.push(statement);
    }
    Ok((symbols, statements, entrypoint))
}

fn parse_statement(
    line: &SourceLine,
) -> Result<Option<(Option<String>, String, Option<String>)>, String> {
    let text = line.text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if let Some((name, expression)) = text.split_once('=').filter(|(left, _)| {
        !left.trim().is_empty()
            && left.trim().chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '_' || character == '*'
            })
    }) {
        let name = name.trim().to_ascii_uppercase();
        if name == "*" {
            return Ok(Some((
                None,
                "*".into(),
                Some(expression.trim().to_string()),
            )));
        }
        return Ok(Some((
            Some(name),
            "=".into(),
            Some(expression.trim().to_string()),
        )));
    }

    if let Some((label, rest)) = text.split_once(':') {
        if !label.trim().is_empty() {
            let (operation, operand) = split_operation(rest);
            return Ok(Some((
                Some(label.trim().to_ascii_uppercase()),
                operation,
                operand,
            )));
        }
    }

    let (first, rest) = split_operation(text);
    let end_is_label = first == "END"
        && rest
            .as_deref()
            .map(split_operation)
            .is_some_and(|(operation, _)| is_instruction(&operation));
    if line.indented || (is_operation(&first) && !end_is_label) {
        return Ok(Some((None, first, rest)));
    }
    if rest.is_none() {
        return Err(error(line, &format!("cannot parse statement {text:?}")));
    }
    let (operation, operand) = split_operation(rest.as_deref().unwrap_or_default());
    if operation.is_empty() {
        return Err(error(line, "label has no statement"));
    }
    Ok(Some((
        Some(first.trim_end_matches(':').to_ascii_uppercase()),
        operation,
        operand,
    )))
}

fn split_operation(text: &str) -> (String, Option<String>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    let operand = trimmed[end..]
        .trim()
        .strip_prefix(',')
        .unwrap_or(trimmed[end..].trim())
        .trim();
    (
        trimmed[..end].to_ascii_uppercase(),
        (!operand.is_empty()).then(|| operand.to_string()),
    )
}

fn statement_size(statement: &Statement, symbols: &HashMap<String, i64>) -> Result<usize, String> {
    match statement.operation.as_str() {
        ".BYTE" | "BYTE" | ".DB" | "DB" => {
            let items = split_arguments(required(&statement.line, &statement.operand)?);
            items.iter().try_fold(0usize, |size, item| {
                Ok(size
                    + if parse_string(item).is_some() {
                        parse_string(item).unwrap().len()
                    } else {
                        1
                    })
            })
        }
        ".WORD" | "WORD" | ".DW" | "DW" => {
            Ok(2 * split_arguments(required(&statement.line, &statement.operand)?).len())
        }
        ".TEXT" | ".ASCII" | "TEXT" | "ASCII" => {
            parse_string(required(&statement.line, &statement.operand)?)
                .map(|text| text.len())
                .ok_or_else(|| error(&statement.line, "expected a quoted string"))
        }
        ".ASCIIZ" | "ASCIIZ" => parse_string(required(&statement.line, &statement.operand)?)
            .map(|text| text.len() + 1)
            .ok_or_else(|| error(&statement.line, "expected a quoted string")),
        ".RES" | "RES" | ".DS" | "DS" => {
            let count = eval(
                required(&statement.line, &statement.operand)?,
                symbols,
                statement.pc,
            )
            .map_err(|message| error(&statement.line, &message))?;
            usize::try_from(count).map_err(|_| error(&statement.line, "invalid reserve size"))
        }
        "END" => Ok(0),
        _ => {
            let (_, mode, _) = instruction(statement, symbols)?;
            Ok(match mode {
                Mode::Implied | Mode::Accumulator => 1,
                Mode::Immediate
                | Mode::ZeroPage
                | Mode::ZeroPageX
                | Mode::ZeroPageY
                | Mode::IndexedIndirect
                | Mode::IndirectIndexed
                | Mode::Relative => 2,
                Mode::Absolute | Mode::AbsoluteX | Mode::AbsoluteY | Mode::Indirect => 3,
            })
        }
    }
}

fn encode(statement: &Statement, symbols: &HashMap<String, i64>) -> Result<Vec<u8>, String> {
    let operand = statement.operand.as_deref();
    match statement.operation.as_str() {
        ".BYTE" | "BYTE" | ".DB" | "DB" => {
            split_arguments(required(&statement.line, &statement.operand)?)
                .iter()
                .map(|item| {
                    if let Some(text) = parse_string(item) {
                        Ok(text.bytes().collect::<Vec<_>>())
                    } else {
                        let value = eval(item, symbols, statement.pc)
                            .map_err(|message| error(&statement.line, &message))?;
                        byte_value(&statement.line, value).map(|byte| vec![byte])
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|parts| parts.into_iter().flatten().collect())
        }
        ".WORD" | "WORD" | ".DW" | "DW" => {
            split_arguments(required(&statement.line, &statement.operand)?)
                .iter()
                .map(|item| {
                    let value = eval(item, symbols, statement.pc)
                        .map_err(|message| error(&statement.line, &message))?;
                    if !(-32768..=65535).contains(&value) {
                        return Err(error(
                            &statement.line,
                            "word value is outside -32768..65535",
                        ));
                    }
                    let value = value as u16;
                    Ok(vec![value as u8, (value >> 8) as u8])
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|parts| parts.into_iter().flatten().collect())
        }
        ".TEXT" | ".ASCII" | "TEXT" | "ASCII" => {
            Ok(parse_string(required(&statement.line, &statement.operand)?)
                .ok_or_else(|| error(&statement.line, "expected a quoted string"))?
                .bytes()
                .collect())
        }
        ".ASCIIZ" | "ASCIIZ" => {
            let mut bytes = parse_string(required(&statement.line, &statement.operand)?)
                .ok_or_else(|| error(&statement.line, "expected a quoted string"))?
                .bytes()
                .collect::<Vec<_>>();
            bytes.push(0);
            Ok(bytes)
        }
        ".RES" | "RES" | ".DS" | "DS" => {
            let count = eval(
                required(&statement.line, &statement.operand)?,
                symbols,
                statement.pc,
            )
            .map_err(|message| error(&statement.line, &message))?;
            let fill = split_arguments(operand.unwrap_or_default())
                .get(1)
                .map(|value| eval(value, symbols, statement.pc))
                .transpose()
                .map_err(|message| error(&statement.line, &message))?
                .unwrap_or(0);
            Ok(vec![byte_value(&statement.line, fill)?; count as usize])
        }
        "END" => Ok(Vec::new()),
        _ => encode_instruction(statement, symbols),
    }
}

fn instruction<'a>(
    statement: &'a Statement,
    symbols: &HashMap<String, i64>,
) -> Result<(&'a str, Mode, Option<i64>), String> {
    let mnemonic = statement.operation.as_str();
    let operand = statement.operand.as_deref().map(str::trim);
    let err = |message: &str| error(&statement.line, message);
    if let Some(operand) = operand {
        if operand.starts_with('#') {
            let value = eval_optional(&operand[1..], symbols, statement.pc)
                .map_err(|message| err(&message))?;
            return Ok((mnemonic, Mode::Immediate, value));
        }
        if operand.eq_ignore_ascii_case("A") {
            return Ok((mnemonic, Mode::Accumulator, None));
        }
        if operand.starts_with('(') {
            let upper = operand.to_ascii_uppercase();
            if upper.ends_with(",X)") {
                let expression = &operand[1..operand.len() - 3];
                let value = eval_optional(expression, symbols, statement.pc)
                    .map_err(|message| err(&message))?;
                return Ok((mnemonic, Mode::IndexedIndirect, value));
            }
            if upper.ends_with("),Y") {
                let expression = &operand[1..operand.len() - 3];
                let value = eval_optional(expression, symbols, statement.pc)
                    .map_err(|message| err(&message))?;
                return Ok((mnemonic, Mode::IndirectIndexed, value));
            }
            let expression = operand
                .strip_prefix('(')
                .and_then(|value| value.strip_suffix(')'))
                .ok_or_else(|| err("malformed indirect operand"))?;
            let value = eval_optional(expression, symbols, statement.pc)
                .map_err(|message| err(&message))?;
            return Ok((mnemonic, Mode::Indirect, value));
        }
    }

    let branch = matches!(
        mnemonic,
        "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BVC" | "BVS"
    );
    if branch {
        let value = eval_optional(
            operand.ok_or_else(|| err("branch requires a target"))?,
            symbols,
            statement.pc,
        )
        .map_err(|message| err(&message))?;
        return Ok((mnemonic, Mode::Relative, value));
    }
    let Some(operand) = operand else {
        let mode = if matches!(mnemonic, "ASL" | "LSR" | "ROL" | "ROR") {
            Mode::Accumulator
        } else {
            Mode::Implied
        };
        if opcode(mnemonic, mode).is_none() {
            return Err(if is_instruction(mnemonic) {
                err("instruction does not support implied addressing")
            } else {
                err(&format!("unsupported instruction {mnemonic}"))
            });
        }
        return Ok((mnemonic, mode, None));
    };

    let upper = operand.to_ascii_uppercase();
    let (expression, indexed_mode, zero_page_mode) = if upper.ends_with(",X") {
        (
            &operand[..operand.len() - 2],
            Mode::AbsoluteX,
            Mode::ZeroPageX,
        )
    } else if upper.ends_with(",Y") {
        (
            &operand[..operand.len() - 2],
            Mode::AbsoluteY,
            Mode::ZeroPageY,
        )
    } else {
        (operand, Mode::Absolute, Mode::ZeroPage)
    };
    let value =
        eval_optional(expression.trim(), symbols, statement.pc).map_err(|message| err(&message))?;
    let mode = if value.is_some_and(|value| (0..=255).contains(&value))
        && opcode(mnemonic, zero_page_mode).is_some()
    {
        zero_page_mode
    } else {
        indexed_mode
    };
    if opcode(mnemonic, mode).is_none() {
        return Err(err("instruction does not support this addressing mode"));
    }
    Ok((mnemonic, mode, value))
}

fn encode_instruction(
    statement: &Statement,
    symbols: &HashMap<String, i64>,
) -> Result<Vec<u8>, String> {
    let (mnemonic, mode, value) = instruction(statement, symbols)?;
    let code = opcode(mnemonic, mode)
        .ok_or_else(|| error(&statement.line, "unsupported instruction/addressing mode"))?;
    let operand_size = match mode {
        Mode::Implied | Mode::Accumulator => return Ok(vec![code]),
        Mode::Immediate
        | Mode::ZeroPage
        | Mode::ZeroPageX
        | Mode::ZeroPageY
        | Mode::IndexedIndirect
        | Mode::IndirectIndexed
        | Mode::Relative => 1,
        Mode::Absolute | Mode::AbsoluteX | Mode::AbsoluteY | Mode::Indirect => 2,
    };
    let value = value.ok_or_else(|| error(&statement.line, "unresolved operand"))?;
    if operand_size == 1 {
        if mode == Mode::Relative {
            let displacement = value - (statement.pc as i64 + 2);
            if !(-128..=127).contains(&displacement) {
                return Err(error(&statement.line, "branch target is out of range"));
            }
            Ok(vec![code, displacement as i8 as u8])
        } else {
            Ok(vec![code, byte_value(&statement.line, value)?])
        }
    } else {
        if !(0..=65535).contains(&value) {
            return Err(error(&statement.line, "address is outside 0..65535"));
        }
        Ok(vec![code, value as u8, (value >> 8) as u8])
    }
}

fn is_instruction(operation: &str) -> bool {
    [
        "ADC", "AND", "ASL", "BCC", "BCS", "BEQ", "BIT", "BMI", "BNE", "BPL", "BRK", "BVC", "BVS",
        "CLC", "CLD", "CLI", "CLV", "CMP", "CPX", "CPY", "DEC", "DEX", "DEY", "EOR", "INC", "INX",
        "INY", "JMP", "JSR", "LDA", "LDX", "LDY", "LSR", "NOP", "ORA", "PHA", "PHP", "PLA", "PLP",
        "ROL", "ROR", "RTI", "RTS", "SBC", "SEC", "SED", "SEI", "STA", "STX", "STY", "TAX", "TAY",
        "TSX", "TXA", "TXS", "TYA",
    ]
    .contains(&operation)
}

fn is_operation(operation: &str) -> bool {
    is_instruction(operation)
        || matches!(
            operation,
            ".BYTE"
                | "BYTE"
                | ".DB"
                | "DB"
                | ".WORD"
                | "WORD"
                | ".DW"
                | "DW"
                | ".TEXT"
                | ".ASCII"
                | "TEXT"
                | "ASCII"
                | ".ASCIIZ"
                | "ASCIIZ"
                | ".RES"
                | "RES"
                | ".DS"
                | "DS"
                | ".ORG"
                | "ORG"
                | "END"
                | "*"
        )
}

fn split_arguments(text: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            arguments.push(text[start..index].trim().to_string());
            start = index + 1;
        }
    }
    arguments.push(text[start..].trim().to_string());
    arguments
}

fn parse_string(text: &str) -> Option<String> {
    let text = text.trim();
    text.strip_prefix('"')?
        .strip_suffix('"')
        .map(str::to_string)
}

fn required<'a>(line: &SourceLine, operand: &'a Option<String>) -> Result<&'a str, String> {
    operand
        .as_deref()
        .ok_or_else(|| error(line, "expected an operand"))
}

fn eval_optional(
    expression: &str,
    symbols: &HashMap<String, i64>,
    pc: u16,
) -> Result<Option<i64>, String> {
    match eval(expression, symbols, pc) {
        Ok(value) => Ok(Some(value)),
        Err(message) if message.starts_with("unknown symbol ") => Ok(None),
        Err(message) => Err(message),
    }
}

fn eval(expression: &str, symbols: &HashMap<String, i64>, pc: u16) -> Result<i64, String> {
    let expression = expression.trim();
    if let Some(rest) = expression.strip_prefix('<') {
        return eval(rest, symbols, pc).map(|value| value & 0xFF);
    }
    if let Some(rest) = expression.strip_prefix('>') {
        return eval(rest, symbols, pc).map(|value| (value >> 8) & 0xFF);
    }
    if let Some(character) = expression
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .filter(|text| text.len() == 1)
    {
        return Ok(character.as_bytes()[0] as i64);
    }
    if let Some(character) = expression
        .strip_prefix('\'')
        .and_then(|text| text.strip_suffix('\''))
        .filter(|text| text.len() == 1)
    {
        return Ok(character.as_bytes()[0] as i64);
    }
    if let Some(rest) = expression.strip_prefix('-') {
        return eval(rest, symbols, pc).map(|value| -value);
    }
    if let Some(rest) = expression.strip_prefix('+') {
        return eval(rest, symbols, pc);
    }
    let mut depth = 0i32;
    for (index, character) in expression.char_indices().rev() {
        match character {
            ')' => depth += 1,
            '(' => depth -= 1,
            '+' | '-' if depth == 0 && index > 0 => {
                let left = eval(&expression[..index], symbols, pc)?;
                let right = eval(&expression[index + 1..], symbols, pc)?;
                return Ok(if character == '+' {
                    left + right
                } else {
                    left - right
                });
            }
            _ => {}
        }
    }
    let expression = expression.trim();
    if let Some(rest) = expression
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
    {
        return eval(rest, symbols, pc);
    }
    if expression == "*" {
        return Ok(pc as i64);
    }
    if let Some(hex) = expression.strip_prefix('$') {
        return i64::from_str_radix(hex, 16).map_err(|_| format!("invalid number {expression}"));
    }
    if let Some(binary) = expression.strip_prefix('%') {
        return i64::from_str_radix(binary, 2).map_err(|_| format!("invalid number {expression}"));
    }
    if let Some(hex) = expression
        .strip_prefix("0x")
        .or_else(|| expression.strip_prefix("0X"))
    {
        return i64::from_str_radix(hex, 16).map_err(|_| format!("invalid number {expression}"));
    }
    if let Ok(value) = expression.parse::<i64>() {
        return Ok(value);
    }
    symbols
        .get(&expression.to_ascii_uppercase())
        .copied()
        .ok_or_else(|| format!("unknown symbol {}", expression.to_ascii_uppercase()))
}

fn checked_address(line: &SourceLine, address: i64) -> Result<u32, String> {
    u32::try_from(address)
        .ok()
        .filter(|address| *address <= 65535)
        .ok_or_else(|| error(line, &format!("address {address} is outside 0..65535")))
}

fn byte_value(line: &SourceLine, value: i64) -> Result<u8, String> {
    if !(-128..=255).contains(&value) {
        return Err(error(
            line,
            &format!("byte value {value} is outside -128..255"),
        ));
    }
    Ok(value as u8)
}

fn error(line: &SourceLine, message: &str) -> String {
    format!("line {}: {message}", line.number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_common_origins_labels_and_data_directives() {
        let source = r#"* = $1000
START       LDX #$00
            LDA TABLE,X
            STA $0200
            LDY #$01
            LDA ($20),Y
            BNE START
            JSR SUB
            JMP (VECTOR)
SUB         RTS
TABLE       .byte $11, $22, "OK"
VECTOR      .word START
* = $1100
OTHER       .asciiz "6502"
        "#;
        let image = assemble(source).unwrap();
        assert_eq!(image.load_address, 0x1000);
        assert_eq!(image.entrypoint, 0x1000);
        assert_eq!(&image.bytes[..2], &[0xA2, 0x00]);
        assert_eq!(&image.bytes[2..5], &[0xBD, 0x15, 0x10]);
        assert_eq!(&image.bytes[5..8], &[0x8D, 0x00, 0x02]);
        assert_eq!(&image.bytes[10..12], &[0xB1, 0x20]);
        assert_eq!(&image.bytes[17..20], &[0x6C, 0x19, 0x10]);
        assert_eq!(&image.bytes[21..27], &[0x11, 0x22, b'O', b'K', 0x00, 0x10]);
        assert_eq!(&image.bytes[0x100..0x105], b"6502\0");
    }

    #[test]
    fn reports_unsupported_non_nmos_instructions() {
        let error = match assemble("        PHY\n") {
            Ok(_) => panic!("PHY is not an NMOS 6502 instruction"),
            Err(error) => error,
        };
        assert!(error.contains("unsupported instruction PHY"), "{error}");
    }

    #[test]
    fn assembles_mchess_using_only_official_nmos_opcodes() {
        let image =
            crate::assembler::assemble_image(include_str!("../assembly/mchess.asm")).unwrap();
        assert_eq!(image.load_address, 0x1000);
        assert_eq!(image.entrypoint, 0x1000);
        assert!(image.bytes.len() > 0x580);
    }

    #[test]
    fn mchess_plays_a_bot_move_and_quits_through_the_local_terminal() {
        let image =
            crate::assembler::assemble_image(include_str!("../assembly/mchess.asm")).unwrap();
        let (mut cpu, mut mem) = crate::define::init();
        let start = image.load_address as usize;
        mem.data[start..start + image.bytes.len()].copy_from_slice(&image.bytes);
        cpu.pc = image.entrypoint as u32;
        let mut output = Vec::new();
        crate::execute::run_with_io(
            &mut cpu,
            &mut mem,
            std::io::Cursor::new(b"1020\nP\nQ\n"),
            &mut output,
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(cpu.halted);
        assert!(output.contains("MicroChess"), "{output:?}");
        assert!(output.contains("00 01 02 03 04 05 06 07"), "{output:?}");
        assert!(output.contains("|WC|WR|WB|WK|WQ|WB|WR|WC|00"), "{output:?}");
        assert!(output.contains("|WP|  |**|"), "{output:?}");
        assert!(output.ends_with('?'), "{output:?}");
    }
}
