use std::collections::HashMap;

use crate::assembler::AssemblyImage;
use crate::execute::{SIM_CHAR_INPUT_HOOK, SIM_INPUT_HOOK, SIM_OUTPUT_HOOK};

#[derive(Clone)]
struct Macro {
    parameters: Vec<String>,
    body: String,
}

#[derive(Clone)]
struct SourceLine {
    text: String,
    line: usize,
    radix: u32,
}

#[derive(Clone)]
struct EncodedLine {
    pc: u16,
    line: SourceLine,
    operation: String,
    operand: Option<String>,
}

pub fn assemble(source: &str) -> Result<AssemblyImage, String> {
    let lines = clean_source(source);
    let mut labels = HashMap::<String, u16>::new();
    let mut layout = Vec::new();
    let mut entrypoint = None;
    let mut constants = HashMap::<String, i64>::new();
    for _ in 0..8 {
        constants = labels
            .iter()
            .map(|(name, value)| (name.clone(), *value as i64))
            .collect();
        constants.insert(
            "CZGETL".into(),
            constants
                .get("CZGETL")
                .copied()
                .unwrap_or(SIM_CHAR_INPUT_HOOK as i64),
        );
        let mut macros = HashMap::new();
        let expanded = preprocess(&lines, &mut constants, &mut macros, 0)?;
        let (mut next_labels, next_layout, next_entry) =
            layout_pass(&expanded, &constants, &labels)?;
        if let Some(address) = next_labels.get("RESTOR").copied() {
            next_labels.entry("RESTORE".into()).or_insert(address);
        }
        let stable = next_labels == labels;
        labels = next_labels;
        layout = next_layout;
        entrypoint = next_entry.or(entrypoint);
        if stable {
            break;
        }
    }

    let entrypoint = entrypoint.ok_or("source does not declare an END entrypoint")?;
    let mut image = vec![0; 65536];
    let mut high_water = 0usize;
    for item in &layout {
        let start = item.pc as usize;
        let bytes = encode_line(item, &constants, &labels)?;
        let end = start
            .checked_add(bytes.len())
            .filter(|end| *end <= image.len())
            .ok_or_else(|| {
                format!(
                    "line {}: output exceeds the 6502 address space",
                    item.line.line
                )
            })?;
        image[start..end].copy_from_slice(&bytes);
        high_water = high_water.max(end);
    }

    let read_only = constants
        .get("ROMSW")
        .copied()
        .filter(|value| *value != 0)
        .map(|_| {
            let start = constants.get("ROMLOC").copied().unwrap_or(0).max(0) as usize;
            (start, high_water)
        });
    Ok(AssemblyImage {
        load_address: 0,
        entrypoint,
        bytes: image[..high_water].to_vec(),
        read_only,
    })
}

fn clean_source(source: &str) -> Vec<SourceLine> {
    let mut comment_end = None;
    let mut radix = 10;
    let mut result = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let trimmed = raw.trim();
        if let Some(end) = &comment_end {
            if trimmed == end {
                comment_end = None;
            }
            continue;
        }
        let cleaned = strip_comment(raw);
        let cleaned_trimmed = cleaned.trim();
        if cleaned_trimmed.to_ascii_uppercase().starts_with("COMMENT ") {
            comment_end = cleaned_trimmed
                .split_whitespace()
                .nth(1)
                .map(str::to_string);
            continue;
        }
        let (first, rest) = first_word(cleaned_trimmed);
        if first.eq_ignore_ascii_case("RADIX") {
            if let Ok(value) = rest.trim().parse::<u32>() {
                radix = value;
            }
            continue;
        }
        if cleaned_trimmed.is_empty() {
            continue;
        }
        result.push(SourceLine {
            text: cleaned.trim_end().to_string(),
            line: index + 1,
            radix,
        });
    }
    result
}

fn strip_comment(line: &str) -> String {
    let mut quoted = false;
    let mut chars = line.char_indices();
    while let Some((index, ch)) = chars.next() {
        if ch == '"' {
            quoted = !quoted;
        } else if ch == ';' && !quoted {
            return line[..index].to_string();
        }
    }
    line.to_string()
}

fn preprocess(
    lines: &[SourceLine],
    constants: &mut HashMap<String, i64>,
    macros: &mut HashMap<String, Macro>,
    depth: usize,
) -> Result<Vec<SourceLine>, String> {
    if depth > 64 {
        return Err("macro expansion nesting is too deep".into());
    }
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let current = &lines[index];
        constants.insert("__RADIX".into(), current.radix as i64);
        let text = current.text.trim();
        let (conditional_label, directive_text) = split_leading_label(text)
            .map(|(label, statement)| (Some(label), statement))
            .unwrap_or((None, text));
        let (first, rest) = first_word(directive_text);
        let first_upper = first.to_ascii_uppercase();

        if first_upper == "DEFINE" {
            let (body, _, consumed) = take_angle_block(lines, index)?;
            define_macro(rest, body, macros, current.line)?;
            index += consumed;
            continue;
        }

        if matches!(
            first_upper.as_str(),
            "IFE" | "IFN" | "IF1" | "IF2" | "IFNDEF"
        ) {
            let (body, tail, consumed) = take_angle_block(lines, index)?;
            if let Some(label) = conditional_label {
                output.push(SourceLine {
                    text: format!("{label}:"),
                    line: current.line,
                    radix: current.radix,
                });
            }
            let condition_text = rest
                .split_once(',')
                .map(|(condition, _)| condition.trim())
                .unwrap_or(rest.trim());
            let condition = match first_upper.as_str() {
                "IFE" => evaluate(condition_text, constants, &HashMap::new(), 0)
                    .map(|value| value == 0)
                    .or_else(|error| is_forward_reference(&error).then_some(false).ok_or(error)),
                "IFN" => evaluate(condition_text, constants, &HashMap::new(), 0)
                    .map(|value| value != 0)
                    .or_else(|error| is_forward_reference(&error).then_some(true).ok_or(error)),
                "IF1" | "IF2" => Ok(false),
                "IFNDEF" => Ok(!constants.contains_key(&condition_text.to_ascii_uppercase())),
                _ => unreachable!(),
            }
            .map_err(|error| format!("line {}: {error}", current.line))?;
            if condition {
                let nested = body
                    .lines()
                    .map(|line| SourceLine {
                        text: line.to_string(),
                        line: current.line,
                        radix: current.radix,
                    })
                    .collect::<Vec<_>>();
                output.extend(preprocess(&nested, constants, macros, depth + 1)?);
            }
            if !tail.trim().is_empty() {
                let trailing = tail
                    .lines()
                    .map(|line| SourceLine {
                        text: line.to_string(),
                        line: current.line,
                        radix: current.radix,
                    })
                    .collect::<Vec<_>>();
                output.extend(preprocess(&trailing, constants, macros, depth + 1)?);
            }
            index += consumed;
            continue;
        }

        if first_upper == "REPEAT" {
            let (body, tail, consumed) = take_angle_block(lines, index)?;
            let count_text = rest.split_once(',').map(|(n, _)| n.trim()).unwrap_or(rest);
            let count = evaluate(count_text, constants, &HashMap::new(), 0)
                .map_err(|error| format!("line {}: {error}", current.line))?;
            if !(0..=10000).contains(&count) {
                return Err(format!(
                    "line {}: invalid REPEAT count {count}",
                    current.line
                ));
            }
            if let Some(label) = conditional_label {
                output.push(SourceLine {
                    text: format!("{label}:"),
                    line: current.line,
                    radix: current.radix,
                });
            }
            let repeated = body
                .lines()
                .map(|line| SourceLine {
                    text: line.to_string(),
                    line: current.line,
                    radix: current.radix,
                })
                .collect::<Vec<_>>();
            for _ in 0..count {
                output.extend(preprocess(&repeated, constants, macros, depth + 1)?);
            }
            if !tail.trim().is_empty() {
                let trailing = tail
                    .lines()
                    .map(|line| SourceLine {
                        text: line.to_string(),
                        line: current.line,
                        radix: current.radix,
                    })
                    .collect::<Vec<_>>();
                output.extend(preprocess(&trailing, constants, macros, depth + 1)?);
            }
            index += consumed;
            continue;
        }

        if matches!(
            first_upper.as_str(),
            "TITLE"
                | "SEARCH"
                | "SALL"
                | "SUBTTL"
                | "PAGE"
                | "RADIX"
                | "PRINTX"
                | "PURGE"
                | "XLIST"
                | ".CREF"
                | ".XCREF"
        ) {
            index += 1;
            continue;
        }

        if conditional_label.is_none()
            && (first_upper.starts_with("DCI") || first_upper.starts_with("DCE"))
        {
            let is_dci = first_upper.starts_with("DCI");
            let directive_len = if is_dci { 3 } else { 3 };
            let text_arg = rest_of_directive(text, directive_len);
            let string = parse_string(text_arg).ok_or_else(|| {
                format!(
                    "line {}: expected a string after {first_upper}",
                    current.line
                )
            })?;
            if is_dci {
                let q = constants.entry("Q".to_string()).or_insert(0);
                *q += 1;
            } else {
                let q = constants.entry("Q".to_string()).or_insert(0);
                *q += 2;
            }
            output.push(SourceLine {
                text: format!("@STRING {}", quote_string(&string)),
                line: current.line,
                radix: current.radix,
            });
            index += 1;
            continue;
        }

        if conditional_label.is_none()
            && (first_upper.starts_with("DT") || first_upper.starts_with("DC\""))
        {
            let directive_len = if first_upper.starts_with("DT") { 2 } else { 2 };
            let string = parse_string(rest_of_directive(text, directive_len)).ok_or_else(|| {
                format!(
                    "line {}: expected a string after {first_upper}",
                    current.line
                )
            })?;
            output.push(SourceLine {
                text: format!(
                    "{} {}",
                    if first_upper.starts_with("DT") {
                        "@TEXT"
                    } else {
                        "@STRING"
                    },
                    quote_string(&string)
                ),
                line: current.line,
                radix: current.radix,
            });
            index += 1;
            continue;
        }

        if let Some((name, value)) = parse_assignment(text) {
            if let Ok(value) = evaluate(value, constants, &HashMap::new(), 0) {
                constants.insert(name, value);
                index += 1;
                continue;
            }
        }

        if let Some((label, instruction)) = split_leading_label(text) {
            if !instruction.is_empty() {
                let (mnemonic, args) = first_word(instruction);
                let upper_mnemonic = mnemonic.to_ascii_uppercase();
                if upper_mnemonic.starts_with("DCI")
                    || upper_mnemonic.starts_with("DCE")
                    || upper_mnemonic.starts_with("DT")
                    || upper_mnemonic.starts_with("DC\"")
                {
                    let prefix_len =
                        if upper_mnemonic.starts_with("DCI") || upper_mnemonic.starts_with("DCE") {
                            3
                        } else if upper_mnemonic.starts_with("DT") {
                            2
                        } else {
                            2
                        };
                    let string = parse_string(rest_of_directive(instruction, prefix_len))
                        .ok_or_else(|| {
                            format!("line {}: expected a string after {mnemonic}", current.line)
                        })?;
                    if upper_mnemonic.starts_with("DCI") {
                        *constants.entry("Q".to_string()).or_insert(0) += 1;
                    } else if upper_mnemonic.starts_with("DCE") {
                        *constants.entry("Q".to_string()).or_insert(0) += 2;
                    }
                    let data_directive = if upper_mnemonic.starts_with("DT") {
                        "@TEXT"
                    } else {
                        "@STRING"
                    };
                    output.push(SourceLine {
                        text: format!("{label}: {data_directive} {}", quote_string(&string)),
                        line: current.line,
                        radix: current.radix,
                    });
                    index += 1;
                    continue;
                }
                if let Some(definition) = macros.get(&mnemonic.to_ascii_uppercase()).cloned() {
                    let expanded = expand_macro(&definition, args, current.line)?;
                    let expanded_lines = expanded
                        .lines()
                        .enumerate()
                        .map(|(index, line)| SourceLine {
                            text: if index == 0 {
                                format!("{label}: {line}")
                            } else {
                                line.to_string()
                            },
                            line: current.line,
                            radix: current.radix,
                        })
                        .collect::<Vec<_>>();
                    output.extend(preprocess(&expanded_lines, constants, macros, depth + 1)?);
                    index += 1;
                    continue;
                }
            }
        }

        if let Some(definition) = macros.get(&first_upper).cloned() {
            let expanded = expand_macro(&definition, rest, current.line)?;
            let expanded_lines = expanded
                .lines()
                .map(|line| SourceLine {
                    text: line.to_string(),
                    line: current.line,
                    radix: current.radix,
                })
                .collect::<Vec<_>>();
            output.extend(preprocess(&expanded_lines, constants, macros, depth + 1)?);
            index += 1;
            continue;
        }

        output.push(current.clone());
        index += 1;
    }
    Ok(output)
}

fn take_angle_block(lines: &[SourceLine], start: usize) -> Result<(String, String, usize), String> {
    let first = &lines[start].text;
    let comma = first
        .find(',')
        .ok_or_else(|| format!("line {}: expected a comma before <...>", lines[start].line))?;
    let open = first[comma + 1..]
        .find('<')
        .map(|offset| comma + 1 + offset)
        .ok_or_else(|| {
            format!(
                "line {}: expected an angle-bracket block",
                lines[start].line
            )
        })?;
    let joined = lines[start..]
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let (close, body) = matching_angle(&joined, open).ok_or_else(|| {
        format!(
            "line {}: unterminated angle-bracket block",
            lines[start].line
        )
    })?;
    let consumed = joined[..=close]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let tail = joined[close + 1..]
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(&joined[close + 1..])
        .to_string();
    Ok((body, tail, consumed))
}

fn matching_angle(text: &str, open: usize) -> Option<(usize, String)> {
    let mut depth = 0usize;
    for (index, ch) in text[open..].char_indices() {
        let absolute = open + index;
        match ch {
            '<' => depth += 1,
            '>' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((absolute, text[open + 1..absolute].to_string()));
                }
            }
            _ => {}
        }
    }
    None
}

fn define_macro(
    declaration: &str,
    body: String,
    macros: &mut HashMap<String, Macro>,
    line: usize,
) -> Result<(), String> {
    let declaration = declaration.trim();
    let (name, parameters) = if let Some(open) = declaration.find('(') {
        let close = declaration[open + 1..]
            .find(')')
            .map(|offset| open + 1 + offset)
            .ok_or_else(|| format!("line {line}: malformed macro parameter list"))?;
        let parameters = declaration[open + 1..close]
            .split(',')
            .map(|parameter| parameter.trim().to_ascii_uppercase())
            .filter(|parameter| !parameter.is_empty())
            .collect();
        (&declaration[..open], parameters)
    } else {
        (
            declaration.split(',').next().unwrap_or(declaration),
            Vec::new(),
        )
    };
    let name = name.trim().to_ascii_uppercase();
    if name.is_empty() {
        return Err(format!("line {line}: macro has no name"));
    }
    macros.insert(name, Macro { parameters, body });
    Ok(())
}

fn expand_macro(definition: &Macro, arguments: &str, line: usize) -> Result<String, String> {
    let values = split_arguments(arguments);
    if values.len() != definition.parameters.len() {
        return Err(format!(
            "line {line}: macro expects {} arguments, got {}",
            definition.parameters.len(),
            values.len()
        ));
    }
    let mut body = definition.body.clone();
    for (parameter, value) in definition.parameters.iter().zip(values) {
        body = replace_word(&body, parameter, value.trim());
    }
    Ok(body)
}

fn replace_word(text: &str, word: &str, replacement: &str) -> String {
    let upper = text.to_ascii_uppercase();
    let upper_word = word.to_ascii_uppercase();
    let mut result = String::new();
    let mut last = 0;
    for (index, _) in upper.match_indices(&upper_word) {
        let before = index == 0
            || !upper.as_bytes()[index - 1].is_ascii_alphanumeric()
                && upper.as_bytes()[index - 1] != b'_';
        let after = index + upper_word.len() == upper.len()
            || !upper.as_bytes()[index + upper_word.len()].is_ascii_alphanumeric()
                && upper.as_bytes()[index + upper_word.len()] != b'_';
        if before && after {
            result.push_str(&text[last..index]);
            result.push_str(replacement);
            last = index + upper_word.len();
        }
    }
    result.push_str(&text[last..]);
    result
}

fn split_arguments(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut start = 0;
    let mut angle = 0i32;
    let mut paren = 0i32;
    let mut quoted = false;
    for (index, ch) in text.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            '<' if !quoted => angle += 1,
            '>' if !quoted => angle -= 1,
            '(' if !quoted => paren += 1,
            ')' if !quoted => paren -= 1,
            ',' if !quoted && angle == 0 && paren == 0 => {
                result.push(text[start..index].trim().to_string());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    result.push(text[start..].trim().to_string());
    result
}

fn first_word(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    match text.find(|ch: char| ch.is_whitespace() || ch == ',') {
        Some(index) => (&text[..index], text[index..].trim()),
        None => (text, ""),
    }
}

fn rest_of_directive(text: &str, prefix_len: usize) -> &str {
    text.get(prefix_len..).unwrap_or("").trim()
}

fn parse_string(text: &str) -> Option<String> {
    let text = text.trim();
    let start = text.find('"')?;
    let end = text[start + 1..].find('"').map(|offset| start + 1 + offset);
    Some(match end {
        Some(end) => text[start + 1..end].to_string(),
        None => text[start + 1..].to_string(),
    })
}

fn quote_string(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn parse_assignment(text: &str) -> Option<(String, &str)> {
    let assignment = text.find("==").map(|index| (index, 2)).or_else(|| {
        text.find('=').and_then(|index| {
            let previous = text.as_bytes().get(index.wrapping_sub(1)).copied();
            let next = text.as_bytes().get(index + 1).copied();
            (previous != Some(b'<') && previous != Some(b'>') && next != Some(b'='))
                .then_some((index, 1))
        })
    })?;
    let (index, width) = assignment;
    let name = text[..index].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
    {
        return None;
    }
    Some((name.to_ascii_uppercase(), text[index + width..].trim()))
}

fn split_leading_label(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    let colon = text.find(':')?;
    let label = text[..colon].trim();
    if label.is_empty()
        || label
            .chars()
            .any(|ch| ch.is_whitespace() || !(ch.is_ascii_alphanumeric() || "_.$%".contains(ch)))
    {
        return None;
    }
    let mut tail = &text[colon + 1..];
    tail = tail.trim_start_matches(':').trim();
    Some((label, tail))
}

fn evaluate(
    expression: &str,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
    pc: u16,
) -> Result<i64, String> {
    let radix = constants.get("__RADIX").copied().unwrap_or(10) as u32;
    let tokens = tokenize(expression, pc, radix)?;
    let mut parser = ExpressionParser {
        tokens,
        position: 0,
        constants,
        labels,
    };
    let value = parser.expression(0)?;
    if parser.position != parser.tokens.len() {
        return Err(format!("unexpected token in expression {expression}"));
    }
    Ok(value)
}

#[derive(Clone, Debug)]
enum ExpressionToken {
    Number(i64),
    Name(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Left,
    Right,
}

fn tokenize(expression: &str, pc: u16, radix: u32) -> Result<Vec<ExpressionToken>, String> {
    let bytes = expression.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let ch = bytes[index] as char;
        if ch.is_ascii_whitespace() || ch == ',' {
            index += 1;
            continue;
        }
        match ch {
            '+' => tokens.push(ExpressionToken::Plus),
            '-' => tokens.push(ExpressionToken::Minus),
            '*' => tokens.push(ExpressionToken::Multiply),
            '/' => tokens.push(ExpressionToken::Divide),
            '&' => tokens.push(ExpressionToken::And),
            '!' => tokens.push(ExpressionToken::Or),
            '(' | '<' => tokens.push(ExpressionToken::Left),
            ')' | '>' => tokens.push(ExpressionToken::Right),
            '"' => {
                let end = expression[index + 1..]
                    .find('"')
                    .map(|offset| index + 1 + offset)
                    .ok_or_else(|| format!("unterminated character constant in {expression}"))?;
                let value = expression[index + 1..end].as_bytes();
                if value.len() != 1 {
                    return Err(format!("expected a one-character constant in {expression}"));
                }
                tokens.push(ExpressionToken::Number(value[0] as i64));
                index = end;
            }
            '^' if bytes
                .get(index + 1)
                .is_some_and(|byte| matches!(*byte as char, 'O' | 'o' | 'D' | 'd')) =>
            {
                let radix = if matches!(bytes[index + 1] as char, 'O' | 'o') {
                    8
                } else {
                    10
                };
                let start = index + 2;
                let end = scan_digits(bytes, start, radix);
                if end == start {
                    return Err(format!("invalid radix constant in {expression}"));
                }
                let value = i64::from_str_radix(&expression[start..end], radix)
                    .map_err(|_| format!("invalid radix constant in {expression}"))?;
                tokens.push(ExpressionToken::Number(value));
                index = end - 1;
            }
            '$' if bytes
                .get(index + 1)
                .is_some_and(|byte| (*byte as char).is_ascii_hexdigit()) =>
            {
                let start = index + 1;
                let end = scan_digits(bytes, start, 16);
                let value = i64::from_str_radix(&expression[start..end], 16)
                    .map_err(|_| format!("invalid hexadecimal constant in {expression}"))?;
                tokens.push(ExpressionToken::Number(value));
                index = end - 1;
            }
            '0'..='9' => {
                let start = index;
                let mut end = start;
                while bytes
                    .get(end)
                    .is_some_and(|byte| (*byte as char).is_ascii_digit())
                {
                    end += 1;
                }
                let digits = &expression[start..end];
                let has_non_octal_digit = digits.bytes().any(|byte| byte > b'7');
                // The original BASIC source uses decimal literals in its radix-8 math package.
                let value = if radix == 8 && has_non_octal_digit {
                    digits
                        .parse::<i64>()
                        .map_err(|_| format!("invalid number in {expression}"))?
                } else {
                    i64::from_str_radix(digits, radix)
                        .map_err(|_| format!("invalid number in {expression}"))?
                };
                tokens.push(ExpressionToken::Number(value));
                index = end - 1;
            }
            '.' => tokens.push(ExpressionToken::Number(pc as i64)),
            _ if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' || ch == '%' => {
                let start = index;
                while index + 1 < bytes.len()
                    && (bytes[index + 1].is_ascii_alphanumeric()
                        || matches!(bytes[index + 1], b'_' | b'$' | b'.' | b'%'))
                {
                    index += 1;
                }
                tokens.push(ExpressionToken::Name(
                    expression[start..=index].to_ascii_uppercase(),
                ));
            }
            _ => {
                return Err(format!(
                    "unsupported character '{ch}' in expression {expression}"
                ));
            }
        }
        index += 1;
    }
    Ok(tokens)
}

fn scan_digits(bytes: &[u8], start: usize, radix: u32) -> usize {
    let mut end = start;
    while let Some(byte) = bytes.get(end) {
        if (*byte as char).to_digit(radix).is_some() {
            end += 1;
        } else {
            break;
        }
    }
    end
}

struct ExpressionParser<'a> {
    tokens: Vec<ExpressionToken>,
    position: usize,
    constants: &'a HashMap<String, i64>,
    labels: &'a HashMap<String, u16>,
}

impl ExpressionParser<'_> {
    fn expression(&mut self, minimum_precedence: u8) -> Result<i64, String> {
        let mut left = self.primary()?;
        loop {
            let Some(token) = self.tokens.get(self.position) else {
                break;
            };
            let (precedence, operator) = match token {
                ExpressionToken::Or => (1, '|'),
                ExpressionToken::And => (2, '&'),
                ExpressionToken::Plus => (3, '+'),
                ExpressionToken::Minus => (3, '-'),
                ExpressionToken::Multiply => (4, '*'),
                ExpressionToken::Divide => (4, '/'),
                _ => break,
            };
            if precedence < minimum_precedence {
                break;
            }
            self.position += 1;
            let right = self.expression(precedence + 1)?;
            left = match operator {
                '|' => left | right,
                '&' => left & right,
                '+' => left + right,
                '-' => left - right,
                '*' => left * right,
                '/' if right != 0 => left / right,
                '/' => return Err("division by zero in expression".into()),
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<i64, String> {
        let Some(token) = self.tokens.get(self.position).cloned() else {
            return Err("expected an expression".into());
        };
        self.position += 1;
        match token {
            ExpressionToken::Number(value) => Ok(value),
            ExpressionToken::Minus => Ok(-self.primary()?),
            ExpressionToken::Plus => self.primary(),
            ExpressionToken::Left => {
                let value = self.expression(0)?;
                if !matches!(self.tokens.get(self.position), Some(ExpressionToken::Right)) {
                    return Err("unclosed parenthesis in expression".into());
                }
                self.position += 1;
                Ok(value)
            }
            ExpressionToken::Name(name) => {
                if name == "ADR"
                    && matches!(self.tokens.get(self.position), Some(ExpressionToken::Left))
                {
                    self.position += 1;
                    let value = self.expression(0)?;
                    if !matches!(self.tokens.get(self.position), Some(ExpressionToken::Right)) {
                        return Err("unclosed ADR() expression".into());
                    }
                    self.position += 1;
                    return Ok(value);
                }
                self.labels
                    .get(&name)
                    .map(|value| *value as i64)
                    .or_else(|| self.constants.get(&name).copied())
                    .ok_or_else(|| format!("unknown symbol {name}"))
            }
            _ => Err("expected a value in expression".into()),
        }
    }
}

fn layout_pass(
    lines: &[SourceLine],
    constants: &HashMap<String, i64>,
    previous_labels: &HashMap<String, u16>,
) -> Result<(HashMap<String, u16>, Vec<EncodedLine>, Option<u16>), String> {
    let mut symbols = constants.clone();
    symbols.extend(
        previous_labels
            .iter()
            .map(|(name, value)| (name.clone(), *value as i64)),
    );
    let mut labels = HashMap::<String, u16>::new();
    let mut layout = Vec::new();
    let mut pc = 0u16;
    let mut entrypoint = None;

    for source_line in lines {
        symbols.insert("__RADIX".into(), source_line.radix as i64);
        let text = source_line.text.trim();
        if text.is_empty() {
            continue;
        }
        let (label, statement) = split_leading_label(text)
            .map(|(label, statement)| (Some(label.to_ascii_uppercase()), statement))
            .unwrap_or((None, text));
        if let Some(label) = label {
            if labels.insert(label.clone(), pc).is_some() {
                return Err(format!(
                    "line {}: duplicate label {label}",
                    source_line.line
                ));
            }
            symbols.insert(label, pc as i64);
        }
        if statement.is_empty() {
            continue;
        }

        if let Some((name, expression)) = parse_assignment(statement) {
            if let Ok(value) = evaluate(expression, &symbols, &labels, pc) {
                symbols.insert(name, value);
            }
            continue;
        }

        let (operation, rest) = first_word(statement);
        let mut operation = operation.to_ascii_uppercase();
        let mut operand = (!rest.trim().is_empty()).then(|| rest.trim().to_string());
        if operation.starts_with("ADR(") && operation.ends_with(')') {
            operand = Some(operation[4..operation.len() - 1].to_string());
            operation = "ADR".into();
        }
        match operation.as_str() {
            "ORG" => {
                let value = evaluate(
                    required_operand(&operand, source_line.line)?,
                    &symbols,
                    &labels,
                    pc,
                )
                .map_err(|error| format!("line {}: {error}", source_line.line))?;
                pc = checked_address(value, source_line.line)?;
            }
            "END" => {
                let expression = required_operand(&operand, source_line.line)?;
                let value = evaluate(expression, &symbols, &labels, pc)
                    .map_err(|error| format!("line {}: {error}", source_line.line))?;
                entrypoint = Some(checked_address(value, source_line.line)?);
            }
            "BLOCK" => {
                let count = evaluate(
                    required_operand(&operand, source_line.line)?,
                    &symbols,
                    &labels,
                    pc,
                )
                .map_err(|error| format!("line {}: {error}", source_line.line))?;
                if !(0..=65536).contains(&count) {
                    return Err(format!(
                        "line {}: invalid BLOCK size {count}",
                        source_line.line
                    ));
                }
                layout.push(EncodedLine {
                    pc,
                    line: source_line.clone(),
                    operation,
                    operand,
                });
                pc = pc.wrapping_add(count as u16);
            }
            _ => {
                let size = line_size(&operation, operand.as_deref(), pc, &symbols, &labels)
                    .map_err(|error| format!("line {}: {error}", source_line.line))?;
                layout.push(EncodedLine {
                    pc,
                    line: source_line.clone(),
                    operation,
                    operand,
                });
                let end = pc as usize + size;
                if end > 65536 {
                    return Err(format!(
                        "line {}: output exceeds the 6502 address space",
                        source_line.line
                    ));
                }
                pc = end as u16;
            }
        }
    }
    Ok((labels, layout, entrypoint))
}

fn line_size(
    operation: &str,
    operand: Option<&str>,
    pc: u16,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
) -> Result<usize, String> {
    match operation {
        "@STRING" | "@TEXT" => Ok(parse_string(operand.unwrap_or_default())
            .ok_or("expected string data")?
            .len()),
        "EXP" => Ok(split_arguments(operand.unwrap_or_default()).len()),
        "XWD" => Ok(1),
        "ADR" => Ok(2),
        "HRLI" => Ok(1),
        "HRRZ" | "JRST" => Ok(3),
        "BLOCK" => Ok(evaluate(
            operand.ok_or("BLOCK requires a size")?,
            constants,
            labels,
            pc,
        )? as usize),
        "DC" | "DT" => Ok(parse_string(operand.unwrap_or_default())
            .ok_or("expected string data")?
            .len()),
        _ => {
            if evaluate(operation, constants, labels, pc).is_ok() {
                return Ok(1);
            }
            let (_, mode, _) = instruction(operation, operand, constants, labels, pc)?;
            Ok(if matches!(mode, Mode::Implied | Mode::Accumulator) {
                1
            } else if matches!(
                mode,
                Mode::ZeroPage
                    | Mode::ZeroPageX
                    | Mode::ZeroPageY
                    | Mode::Immediate
                    | Mode::Relative
                    | Mode::IndexedIndirect
                    | Mode::IndirectIndexed
            ) {
                2
            } else {
                3
            })
        }
    }
}

fn required_operand(operand: &Option<String>, line: usize) -> Result<&str, String> {
    operand
        .as_deref()
        .ok_or_else(|| format!("line {line}: expected an operand"))
}

fn checked_address(value: i64, line: usize) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("line {line}: address {value} is outside 0..65535"))
}

fn is_forward_reference(error: &str) -> bool {
    error.starts_with("unknown symbol ")
}

fn encode_line(
    line: &EncodedLine,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
) -> Result<Vec<u8>, String> {
    let mut line_constants = constants.clone();
    line_constants.insert("__RADIX".into(), line.line.radix as i64);
    let constants = &line_constants;
    let operand = line.operand.as_deref();
    let error = |message: String| format!("line {}: {message}", line.line.line);
    match line.operation.as_str() {
        "EXP" => split_arguments(required_operand(&line.operand, line.line.line)?)
            .iter()
            .map(|expression| {
                evaluate(expression, constants, labels, line.pc)
                    .map(|value| value as u8)
                    .map_err(&error)
            })
            .collect(),
        "XWD" => {
            let values = split_arguments(required_operand(&line.operand, line.line.line)?);
            let value = values
                .get(1)
                .ok_or_else(|| error("XWD expects two values".into()))?;
            Ok(vec![
                evaluate(value, constants, labels, line.pc).map_err(error)? as u8,
            ])
        }
        "ADR" => {
            let value = evaluate(
                required_operand(&line.operand, line.line.line)?,
                constants,
                labels,
                line.pc,
            )
            .map_err(error)?;
            if !(0..=65535).contains(&value) {
                return Err(error(format!("address {value} is outside 0..65535")));
            }
            Ok(vec![value as u8, (value >> 8) as u8])
        }
        "@STRING" | "DC" => {
            let value = parse_string(operand.unwrap_or_default())
                .ok_or_else(|| error("expected string data".into()))?;
            let mut bytes = value.bytes().collect::<Vec<_>>();
            if let Some(last) = bytes.last_mut() {
                *last |= 0x80;
            }
            Ok(bytes)
        }
        "@TEXT" | "DT" => Ok(parse_string(operand.unwrap_or_default())
            .ok_or_else(|| error("expected text data".into()))?
            .bytes()
            .collect()),
        "BLOCK" => {
            let count = evaluate(
                required_operand(&line.operand, line.line.line)?,
                constants,
                labels,
                line.pc,
            )
            .map_err(error)?;
            Ok(vec![0; count as usize])
        }
        "HRLI" => Ok(vec![0xEA]),
        "HRRZ" => Ok(vec![0x00, 0xEA, 0xEA]),
        "JRST" => Ok(vec![0xEA, 0xEA, 0xEA]),
        _ => {
            if let Ok(value) = evaluate(&line.operation, constants, labels, line.pc) {
                return Ok(vec![value as u8]);
            }
            encode_instruction(&line.operation, operand, line.pc, constants, labels).map_err(error)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
    Indirect,
    Relative,
}

fn instruction(
    mnemonic: &str,
    operand: Option<&str>,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
    pc: u16,
) -> Result<(String, Mode, Option<i64>), String> {
    let mnemonic = mnemonic.to_ascii_uppercase();
    let (base, forced_mode) = match mnemonic.as_str() {
        "LDAI" => ("LDA", Some(Mode::Immediate)),
        "LDXI" => ("LDX", Some(Mode::Immediate)),
        "LDYI" => ("LDY", Some(Mode::Immediate)),
        "ADCI" => ("ADC", Some(Mode::Immediate)),
        "SBCI" => ("SBC", Some(Mode::Immediate)),
        "ANDI" => ("AND", Some(Mode::Immediate)),
        "EORI" => ("EOR", Some(Mode::Immediate)),
        "ORAI" => ("ORA", Some(Mode::Immediate)),
        "CMPI" => ("CMP", Some(Mode::Immediate)),
        "CPXI" => ("CPX", Some(Mode::Immediate)),
        "CPYI" => ("CPY", Some(Mode::Immediate)),
        "LDADY" => ("LDA", Some(Mode::IndirectIndexed)),
        "STADY" => ("STA", Some(Mode::IndirectIndexed)),
        "ADCDY" => ("ADC", Some(Mode::IndirectIndexed)),
        "SBCDY" => ("SBC", Some(Mode::IndirectIndexed)),
        "CMPDY" => ("CMP", Some(Mode::IndirectIndexed)),
        "JMPD" => ("JMP", Some(Mode::Indirect)),
        "TJSR" => ("JSR", Some(Mode::Absolute)),
        _ => (mnemonic.as_str(), None),
    };

    let Some(operand) = operand.map(str::trim).filter(|text| !text.is_empty()) else {
        let mode = forced_mode.unwrap_or_else(|| {
            if matches!(base, "ASL" | "LSR" | "ROL" | "ROR") {
                Mode::Accumulator
            } else {
                Mode::Implied
            }
        });
        if opcode(base, mode).is_none() {
            return Err(format!("{mnemonic} does not accept this operand form"));
        }
        return Ok((base.to_string(), mode, None));
    };
    let operand = operand.trim_end_matches(',').trim();

    if mnemonic == "TJSR" {
        let target = operand.trim_end_matches('>').trim_end_matches('#');
        let address = if target.to_ascii_uppercase().starts_with("INSIM") {
            SIM_INPUT_HOOK as i64
        } else if target.to_ascii_uppercase().starts_with("OUTSIM") {
            SIM_OUTPUT_HOOK as i64
        } else {
            return Err(format!("unknown simulator hook {target}"));
        };
        return Ok((base.to_string(), Mode::Absolute, Some(address)));
    }

    if matches!(
        base,
        "BPL" | "BMI" | "BVC" | "BVS" | "BCC" | "BCS" | "BNE" | "BEQ"
    ) {
        let value = evaluate_operand(operand, constants, labels, pc)?;
        return Ok((base.to_string(), Mode::Relative, value));
    }
    if forced_mode == Some(Mode::Immediate) {
        let expression = operand.trim_start_matches('#').trim();
        let value = evaluate_operand(expression, constants, labels, pc)?;
        return Ok((base.to_string(), Mode::Immediate, value));
    }
    if operand.starts_with('#') {
        let value = evaluate_operand(
            operand.trim_start_matches('#').trim(),
            constants,
            labels,
            pc,
        )?;
        return Ok((base.to_string(), Mode::Immediate, value));
    }
    if operand.eq_ignore_ascii_case("A") {
        return Ok((base.to_string(), Mode::Accumulator, None));
    }
    if forced_mode == Some(Mode::Indirect) || operand.starts_with('(') {
        let upper = operand.to_ascii_uppercase();
        if upper.ends_with(",X)") {
            let expression = &operand[1..operand.len() - 3];
            let value = evaluate_operand(expression, constants, labels, pc)?;
            return Ok((base.to_string(), Mode::IndexedIndirect, value));
        }
        if upper.ends_with("),Y") {
            let expression = &operand[1..operand.len() - 3];
            let value = evaluate_operand(expression, constants, labels, pc)?;
            return Ok((base.to_string(), Mode::IndirectIndexed, value));
        }
        let expression = operand
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .unwrap_or(operand);
        let value = evaluate_operand(expression, constants, labels, pc)?;
        return Ok((base.to_string(), Mode::Indirect, value));
    }

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
    let value = evaluate_operand(expression.trim(), constants, labels, pc)?;
    if let Some(mode) = forced_mode {
        return Ok((base.to_string(), mode, value));
    }
    let mode = if value.is_some_and(|value| (0..=255).contains(&value))
        && opcode(base, zero_page_mode).is_some()
    {
        zero_page_mode
    } else {
        indexed_mode
    };
    if opcode(base, mode).is_none() {
        return Err(format!("{mnemonic} does not support this addressing mode"));
    }
    Ok((base.to_string(), mode, value))
}

fn evaluate_operand(
    expression: &str,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
    pc: u16,
) -> Result<Option<i64>, String> {
    match evaluate(expression, constants, labels, pc) {
        Ok(value) => Ok(Some(value)),
        Err(error) if is_forward_reference(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

fn encode_instruction(
    mnemonic: &str,
    operand: Option<&str>,
    pc: u16,
    constants: &HashMap<String, i64>,
    labels: &HashMap<String, u16>,
) -> Result<Vec<u8>, String> {
    let (base, mode, value) = instruction(mnemonic, operand, constants, labels, pc)?;
    let code = opcode(&base, mode)
        .ok_or_else(|| format!("unsupported instruction or addressing mode {mnemonic}"))?;
    let Some(value) = value else {
        if matches!(mode, Mode::Implied | Mode::Accumulator) {
            return Ok(vec![code]);
        }
        return Err(format!(
            "line operand is unresolved or invalid: {}",
            operand.unwrap_or_default()
        ));
    };
    match mode {
        Mode::Relative => {
            let displacement = value - (pc as i64 + 2);
            if !(-128..=127).contains(&displacement) {
                return Err(format!("branch target is out of range ({displacement})"));
            }
            Ok(vec![code, displacement as i8 as u8])
        }
        Mode::Immediate
        | Mode::ZeroPage
        | Mode::ZeroPageX
        | Mode::ZeroPageY
        | Mode::IndexedIndirect
        | Mode::IndirectIndexed => {
            if !(-128..=255).contains(&value) {
                return Err(format!("operand {value} does not fit in one byte"));
            }
            Ok(vec![code, value as u8])
        }
        Mode::Absolute | Mode::AbsoluteX | Mode::AbsoluteY | Mode::Indirect => {
            if !(0..=65535).contains(&value) {
                return Err(format!("address {value} is outside 0..65535"));
            }
            Ok(vec![code, value as u8, (value >> 8) as u8])
        }
        Mode::Implied | Mode::Accumulator => Ok(vec![code]),
    }
}

fn opcode(mnemonic: &str, mode: Mode) -> Option<u8> {
    use Mode::*;
    Some(match (mnemonic, mode) {
        ("ADC", Immediate) => 0x69,
        ("ADC", ZeroPage) => 0x65,
        ("ADC", ZeroPageX) => 0x75,
        ("ADC", Absolute) => 0x6D,
        ("ADC", AbsoluteX) => 0x7D,
        ("ADC", AbsoluteY) => 0x79,
        ("ADC", IndexedIndirect) => 0x61,
        ("ADC", IndirectIndexed) => 0x71,
        ("AND", Immediate) => 0x29,
        ("AND", ZeroPage) => 0x25,
        ("AND", ZeroPageX) => 0x35,
        ("AND", Absolute) => 0x2D,
        ("AND", AbsoluteX) => 0x3D,
        ("AND", AbsoluteY) => 0x39,
        ("AND", IndexedIndirect) => 0x21,
        ("AND", IndirectIndexed) => 0x31,
        ("ASL", Accumulator) => 0x0A,
        ("ASL", ZeroPage) => 0x06,
        ("ASL", ZeroPageX) => 0x16,
        ("ASL", Absolute) => 0x0E,
        ("ASL", AbsoluteX) => 0x1E,
        ("BCC", Relative) => 0x90,
        ("BCS", Relative) => 0xB0,
        ("BEQ", Relative) => 0xF0,
        ("BIT", ZeroPage) => 0x24,
        ("BIT", Absolute) => 0x2C,
        ("BMI", Relative) => 0x30,
        ("BNE", Relative) => 0xD0,
        ("BPL", Relative) => 0x10,
        ("BRK", Implied) => 0x00,
        ("BVC", Relative) => 0x50,
        ("BVS", Relative) => 0x70,
        ("CLC", Implied) => 0x18,
        ("CLD", Implied) => 0xD8,
        ("CLI", Implied) => 0x58,
        ("CLV", Implied) => 0xB8,
        ("CMP", Immediate) => 0xC9,
        ("CMP", ZeroPage) => 0xC5,
        ("CMP", ZeroPageX) => 0xD5,
        ("CMP", Absolute) => 0xCD,
        ("CMP", AbsoluteX) => 0xDD,
        ("CMP", AbsoluteY) => 0xD9,
        ("CMP", IndexedIndirect) => 0xC1,
        ("CMP", IndirectIndexed) => 0xD1,
        ("CPX", Immediate) => 0xE0,
        ("CPX", ZeroPage) => 0xE4,
        ("CPX", Absolute) => 0xEC,
        ("CPY", Immediate) => 0xC0,
        ("CPY", ZeroPage) => 0xC4,
        ("CPY", Absolute) => 0xCC,
        ("DEC", ZeroPage) => 0xC6,
        ("DEC", ZeroPageX) => 0xD6,
        ("DEC", Absolute) => 0xCE,
        ("DEC", AbsoluteX) => 0xDE,
        ("DEX", Implied) => 0xCA,
        ("DEY", Implied) => 0x88,
        ("EOR", Immediate) => 0x49,
        ("EOR", ZeroPage) => 0x45,
        ("EOR", ZeroPageX) => 0x55,
        ("EOR", Absolute) => 0x4D,
        ("EOR", AbsoluteX) => 0x5D,
        ("EOR", AbsoluteY) => 0x59,
        ("EOR", IndexedIndirect) => 0x41,
        ("EOR", IndirectIndexed) => 0x51,
        ("INC", ZeroPage) => 0xE6,
        ("INC", ZeroPageX) => 0xF6,
        ("INC", Absolute) => 0xEE,
        ("INC", AbsoluteX) => 0xFE,
        ("INX", Implied) => 0xE8,
        ("INY", Implied) => 0xC8,
        ("JMP", Absolute) => 0x4C,
        ("JMP", Indirect) => 0x6C,
        ("JSR", Absolute) => 0x20,
        ("LDA", Immediate) => 0xA9,
        ("LDA", ZeroPage) => 0xA5,
        ("LDA", ZeroPageX) => 0xB5,
        ("LDA", Absolute) => 0xAD,
        ("LDA", AbsoluteX) => 0xBD,
        ("LDA", AbsoluteY) => 0xB9,
        ("LDA", IndexedIndirect) => 0xA1,
        ("LDA", IndirectIndexed) => 0xB1,
        ("LDX", Immediate) => 0xA2,
        ("LDX", ZeroPage) => 0xA6,
        ("LDX", ZeroPageY) => 0xB6,
        ("LDX", Absolute) => 0xAE,
        ("LDX", AbsoluteY) => 0xBE,
        ("LDY", Immediate) => 0xA0,
        ("LDY", ZeroPage) => 0xA4,
        ("LDY", ZeroPageX) => 0xB4,
        ("LDY", Absolute) => 0xAC,
        ("LDY", AbsoluteX) => 0xBC,
        ("LSR", Accumulator) => 0x4A,
        ("LSR", ZeroPage) => 0x46,
        ("LSR", ZeroPageX) => 0x56,
        ("LSR", Absolute) => 0x4E,
        ("LSR", AbsoluteX) => 0x5E,
        ("NOP", Implied) => 0xEA,
        ("ORA", Immediate) => 0x09,
        ("ORA", ZeroPage) => 0x05,
        ("ORA", ZeroPageX) => 0x15,
        ("ORA", Absolute) => 0x0D,
        ("ORA", AbsoluteX) => 0x1D,
        ("ORA", AbsoluteY) => 0x19,
        ("ORA", IndexedIndirect) => 0x01,
        ("ORA", IndirectIndexed) => 0x11,
        ("PHA", Implied) => 0x48,
        ("PHP", Implied) => 0x08,
        ("PLA", Implied) => 0x68,
        ("PLP", Implied) => 0x28,
        ("ROL", Accumulator) => 0x2A,
        ("ROL", ZeroPage) => 0x26,
        ("ROL", ZeroPageX) => 0x36,
        ("ROL", Absolute) => 0x2E,
        ("ROL", AbsoluteX) => 0x3E,
        ("ROR", Accumulator) => 0x6A,
        ("ROR", ZeroPage) => 0x66,
        ("ROR", ZeroPageX) => 0x76,
        ("ROR", Absolute) => 0x6E,
        ("ROR", AbsoluteX) => 0x7E,
        ("RTI", Implied) => 0x40,
        ("RTS", Implied) => 0x60,
        ("SBC", Immediate) => 0xE9,
        ("SBC", ZeroPage) => 0xE5,
        ("SBC", ZeroPageX) => 0xF5,
        ("SBC", Absolute) => 0xED,
        ("SBC", AbsoluteX) => 0xFD,
        ("SBC", AbsoluteY) => 0xF9,
        ("SBC", IndexedIndirect) => 0xE1,
        ("SBC", IndirectIndexed) => 0xF1,
        ("SEC", Implied) => 0x38,
        ("SED", Implied) => 0xF8,
        ("SEI", Implied) => 0x78,
        ("STA", ZeroPage) => 0x85,
        ("STA", ZeroPageX) => 0x95,
        ("STA", Absolute) => 0x8D,
        ("STA", AbsoluteX) => 0x9D,
        ("STA", AbsoluteY) => 0x99,
        ("STA", IndexedIndirect) => 0x81,
        ("STA", IndirectIndexed) => 0x91,
        ("STX", ZeroPage) => 0x86,
        ("STX", ZeroPageY) => 0x96,
        ("STX", Absolute) => 0x8E,
        ("STY", ZeroPage) => 0x84,
        ("STY", ZeroPageX) => 0x94,
        ("STY", Absolute) => 0x8C,
        ("TAX", Implied) => 0xAA,
        ("TAY", Implied) => 0xA8,
        ("TSX", Implied) => 0xBA,
        ("TXA", Implied) => 0x8A,
        ("TXS", Implied) => 0x9A,
        ("TYA", Implied) => 0x98,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radix_eight_math_expressions_accept_decimal_digits() {
        let constants = HashMap::from([("__RADIX".into(), 8), ("ADDPRC".into(), 1)]);
        assert_eq!(
            evaluate("8*ADDPRC+230", &constants, &HashMap::new(), 0),
            Ok(160)
        );
        assert_eq!(evaluate("128", &constants, &HashMap::new(), 0), Ok(128));
        assert_eq!(evaluate("200", &constants, &HashMap::new(), 0), Ok(128));
    }

    #[test]
    fn microsoft_basic_runs_numeric_and_string_commands() {
        let image =
            crate::assembler::assemble_image(include_str!("../assembly/m6502.asm")).unwrap();
        assert_eq!(image.entrypoint, 0);
        assert!(
            image
                .read_only
                .is_some_and(|(start, end)| start == 8192 && end > start)
        );
        let (mut cpu, mut mem) = crate::define::init();
        mem.data[..image.bytes.len()].copy_from_slice(&image.bytes);
        mem.read_only = image.read_only;
        cpu.pc = image.entrypoint as u32;
        let mut output = Vec::new();
        crate::execute::run_with_io(
            &mut cpu,
            &mut mem,
            std::io::Cursor::new(b"8192\n80\nPRINT 2\nPRINT 1+1\nPRINT \"HELLO\"\n"),
            &mut output,
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("7423 BYTES FREE"), "{output:?}");
        assert!(output.matches(" 2 ").count() >= 2, "{output:?}");
        assert!(output.contains("HELLO"), "{output:?}");
    }
}
