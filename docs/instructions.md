# MOS 6502 Instruction Implementation Checklist

The emulator implements all 151 official NMOS 6502 opcodes. Unsupported
unofficial opcodes are reported as errors. Addressing and timing behavior follow
the [MOS 6500 Family Hardware Manual](./mcs6500_family_hardware_manual.pdf) and
the instruction definitions in the
[MOS 6500 Family Programming Manual](./mcs6500_family_programming_manual.pdf).

## Addressing mode abbreviations

| Abbreviation | Meaning |
|---|---|
| IMP | Implied |
| ACC | Accumulator |
| IMM | Immediate |
| ZP | Zero page |
| ZPX | Zero page indexed by X |
| ZPY | Zero page indexed by Y |
| ABS | Absolute |
| ABX | Absolute indexed by X |
| ABY | Absolute indexed by Y |
| INX | Indexed indirect: (zp,X) |
| INY | Indirect indexed: (zp),Y |
| IND | Indirect (JMP only) |
| REL | Relative branch |

## Flags

| Flag | Meaning |
|---|---|
| C | Carry / no borrow |
| Z | Zero result |
| I | Interrupt disable |
| D | Decimal mode |
| B | Break marker when pushing status |
| V | Signed overflow |
| N | Negative (bit 7 of result) |
| - | Unchanged |

## Load and store instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | LDA         | IMM  | `A9`   |     2 |      2 | A = value; NZ           |
| - [x] | LDA         | ZP   | `A5`   |     2 |      3 | A = M; NZ               |
| - [x] | LDA         | ZPX  | `B5`   |     2 |      4 | A = M; NZ               |
| - [x] | LDA         | ABS  | `AD`   |     3 |      4 | A = M; NZ               |
| - [x] | LDA         | ABX  | `BD`   |     3 |     4+ | A = M; NZ               |
| - [x] | LDA         | ABY  | `B9`   |     3 |     4+ | A = M; NZ               |
| - [x] | LDA         | INX  | `A1`   |     2 |      6 | A = M; NZ               |
| - [x] | LDA         | INY  | `B1`   |     2 |     5+ | A = M; NZ               |
| - [x] | LDX         | IMM  | `A2`   |     2 |      2 | X = value; NZ           |
| - [x] | LDX         | ZP   | `A6`   |     2 |      3 | X = M; NZ               |
| - [x] | LDX         | ZPY  | `B6`   |     2 |      4 | X = M; NZ               |
| - [x] | LDX         | ABS  | `AE`   |     3 |      4 | X = M; NZ               |
| - [x] | LDX         | ABY  | `BE`   |     3 |     4+ | X = M; NZ               |
| - [x] | LDY         | IMM  | `A0`   |     2 |      2 | Y = value; NZ           |
| - [x] | LDY         | ZP   | `A4`   |     2 |      3 | Y = M; NZ               |
| - [x] | LDY         | ZPX  | `B4`   |     2 |      4 | Y = M; NZ               |
| - [x] | LDY         | ABS  | `AC`   |     3 |      4 | Y = M; NZ               |
| - [x] | LDY         | ABX  | `BC`   |     3 |     4+ | Y = M; NZ               |
| - [x] | STA         | ZP   | `85`   |     2 |      3 | M = A                   |
| - [x] | STA         | ZPX  | `95`   |     2 |      4 | M = A                   |
| - [x] | STA         | ABS  | `8D`   |     3 |      4 | M = A                   |
| - [x] | STA         | ABX  | `9D`   |     3 |      5 | M = A                   |
| - [x] | STA         | ABY  | `99`   |     3 |      5 | M = A                   |
| - [x] | STA         | INX  | `81`   |     2 |      6 | M = A                   |
| - [x] | STA         | INY  | `91`   |     2 |      6 | M = A                   |
| - [x] | STX         | ZP   | `86`   |     2 |      3 | M = X                   |
| - [x] | STX         | ZPY  | `96`   |     2 |      4 | M = X                   |
| - [x] | STX         | ABS  | `8E`   |     3 |      4 | M = X                   |
| - [x] | STY         | ZP   | `84`   |     2 |      3 | M = Y                   |
| - [x] | STY         | ZPX  | `94`   |     2 |      4 | M = Y                   |
| - [x] | STY         | ABS  | `8C`   |     3 |      4 | M = Y                   |

## Register transfer instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [x] | TAX | IMP | `AA` | 1 | 2 | X = A; NZ |
| - [x] | TAY | IMP | `A8` | 1 | 2 | Y = A; NZ |
| - [x] | TXA | IMP | `8A` | 1 | 2 | A = X; NZ |
| - [x] | TYA | IMP | `98` | 1 | 2 | A = Y; NZ |
| - [x] | TSX | IMP | `BA` | 1 | 2 | X = SP; NZ |
| - [x] | TXS | IMP | `9A` | 1 | 2 | SP = X; flags unchanged |

## Arithmetic instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | ADC         | IMM  | `69`   |     2 |      2 | A = A + M + C; NZCV     |
| - [x] | ADC         | ZP   | `65`   |     2 |      3 | A = A + M + C; NZCV     |
| - [x] | ADC         | ZPX  | `75`   |     2 |      4 | A = A + M + C; NZCV     |
| - [x] | ADC         | ABS  | `6D`   |     3 |      4 | A = A + M + C; NZCV     |
| - [x] | ADC         | ABX  | `7D`   |     3 |     4+ | A = A + M + C; NZCV     |
| - [x] | ADC         | ABY  | `79`   |     3 |     4+ | A = A + M + C; NZCV     |
| - [x] | ADC         | INX  | `61`   |     2 |      6 | A = A + M + C; NZCV     |
| - [x] | ADC         | INY  | `71`   |     2 |     5+ | A = A + M + C; NZCV     |
| - [x] | SBC         | IMM  | `E9`   |     2 |      2 | A = A - M - (1-C); NZCV |
| - [x] | SBC         | ZP   | `E5`   |     2 |      3 | A = A - M - (1-C); NZCV |
| - [x] | SBC         | ZPX  | `F5`   |     2 |      4 | A = A - M - (1-C); NZCV |
| - [x] | SBC         | ABS  | `ED`   |     3 |      4 | A = A - M - (1-C); NZCV |
| - [x] | SBC         | ABX  | `FD`   |     3 |     4+ | A = A - M - (1-C); NZCV |
| - [x] | SBC         | ABY  | `F9`   |     3 |     4+ | A = A - M - (1-C); NZCV |
| - [x] | SBC         | INX  | `E1`   |     2 |      6 | A = A - M - (1-C); NZCV |
| - [x] | SBC         | INY  | `F1`   |     2 |     5+ | A = A - M - (1-C); NZCV |
| - [x] | INC         | ZP   | `E6`   |     2 |      5 | M = M + 1; NZ           |
| - [x] | INC         | ZPX  | `F6`   |     2 |      6 | M = M + 1; NZ           |
| - [x] | INC         | ABS  | `EE`   |     3 |      6 | M = M + 1; NZ           |
| - [x] | INC         | ABX  | `FE`   |     3 |      7 | M = M + 1; NZ           |
| - [x] | INX         | IMP  | `E8`   |     1 |      2 | X = X + 1; NZ           |
| - [x] | INY         | IMP  | `C8`   |     1 |      2 | Y = Y + 1; NZ           |
| - [x] | DEC         | ZP   | `C6`   |     2 |      5 | M = M - 1; NZ           |
| - [x] | DEC         | ZPX  | `D6`   |     2 |      6 | M = M - 1; NZ           |
| - [x] | DEC         | ABS  | `CE`   |     3 |      6 | M = M - 1; NZ           |
| - [x] | DEC         | ABX  | `DE`   |     3 |      7 | M = M - 1; NZ           |
| - [x] | DEX         | IMP  | `CA`   |     1 |      2 | X = X - 1; NZ           |
| - [x] | DEY         | IMP  | `88`   |     1 |      2 | Y = Y - 1; NZ           |

## Logical instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [x] | AND | IMM | `29` | 2 | 2 | A = A AND M; NZ |
| - [x] | AND | ZP | `25` | 2 | 3 | A = A AND M; NZ |
| - [x] | AND | ZPX | `35` | 2 | 4 | A = A AND M; NZ |
| - [x] | AND | ABS | `2D` | 3 | 4 | A = A AND M; NZ |
| - [x] | AND | ABX | `3D` | 3 | 4+ | A = A AND M; NZ |
| - [x] | AND | ABY | `39` | 3 | 4+ | A = A AND M; NZ |
| - [x] | AND | INX | `21` | 2 | 6 | A = A AND M; NZ |
| - [x] | AND | INY | `31` | 2 | 5+ | A = A AND M; NZ |
| - [x] | ORA | IMM | `09` | 2 | 2 | A = A OR M; NZ |
| - [x] | ORA | ZP | `05` | 2 | 3 | A = A OR M; NZ |
| - [x] | ORA | ZPX | `15` | 2 | 4 | A = A OR M; NZ |
| - [x] | ORA | ABS | `0D` | 3 | 4 | A = A OR M; NZ |
| - [x] | ORA | ABX | `1D` | 3 | 4+ | A = A OR M; NZ |
| - [x] | ORA | ABY | `19` | 3 | 4+ | A = A OR M; NZ |
| - [x] | ORA | INX | `01` | 2 | 6 | A = A OR M; NZ |
| - [x] | ORA | INY | `11` | 2 | 5+ | A = A OR M; NZ |
| - [x] | EOR | IMM | `49` | 2 | 2 | A = A XOR M; NZ |
| - [x] | EOR | ZP | `45` | 2 | 3 | A = A XOR M; NZ |
| - [x] | EOR | ZPX | `55` | 2 | 4 | A = A XOR M; NZ |
| - [x] | EOR | ABS | `4D` | 3 | 4 | A = A XOR M; NZ |
| - [x] | EOR | ABX | `5D` | 3 | 4+ | A = A XOR M; NZ |
| - [x] | EOR | ABY | `59` | 3 | 4+ | A = A XOR M; NZ |
| - [x] | EOR | INX | `41` | 2 | 6 | A = A XOR M; NZ |
| - [x] | EOR | INY | `51` | 2 | 5+ | A = A XOR M; NZ |
| - [x] | BIT | ZP | `24` | 2 | 3 | Z = !(A AND M); N = M7; V = M6 |
| - [x] | BIT | ABS | `2C` | 3 | 4 | Z = !(A AND M); N = M7; V = M6 |

## Compare instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | CMP         | IMM  | `C9`   |     2 |      2 | Compare A with M; NZC   |
| - [x] | CMP         | ZP   | `C5`   |     2 |      3 | Compare A with M; NZC   |
| - [x] | CMP         | ZPX  | `D5`   |     2 |      4 | Compare A with M; NZC   |
| - [x] | CMP         | ABS  | `CD`   |     3 |      4 | Compare A with M; NZC   |
| - [x] | CMP         | ABX  | `DD`   |     3 |     4+ | Compare A with M; NZC   |
| - [x] | CMP         | ABY  | `D9`   |     3 |     4+ | Compare A with M; NZC   |
| - [x] | CMP         | INX  | `C1`   |     2 |      6 | Compare A with M; NZC   |
| - [x] | CMP         | INY  | `D1`   |     2 |     5+ | Compare A with M; NZC   |
| - [x] | CPX         | IMM  | `E0`   |     2 |      2 | Compare X with M; NZC   |
| - [x] | CPX         | ZP   | `E4`   |     2 |      3 | Compare X with M; NZC   |
| - [x] | CPX         | ABS  | `EC`   |     3 |      4 | Compare X with M; NZC   |
| - [x] | CPY         | IMM  | `C0`   |     2 |      2 | Compare Y with M; NZC   |
| - [x] | CPY         | ZP   | `C4`   |     2 |      3 | Compare Y with M; NZC   |
| - [x] | CPY         | ABS  | `CC`   |     3 |      4 | Compare Y with M; NZC   |

## Shift and rotate instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [x] | ASL | ACC | `0A` | 1 | 2 | A <<= 1; NZC |
| - [x] | ASL | ZP | `06` | 2 | 5 | M <<= 1; NZC |
| - [x] | ASL | ZPX | `16` | 2 | 6 | M <<= 1; NZC |
| - [x] | ASL | ABS | `0E` | 3 | 6 | M <<= 1; NZC |
| - [x] | ASL | ABX | `1E` | 3 | 7 | M <<= 1; NZC |
| - [x] | LSR | ACC | `4A` | 1 | 2 | A >>= 1; NZC |
| - [x] | LSR | ZP | `46` | 2 | 5 | M >>= 1; NZC |
| - [x] | LSR | ZPX | `56` | 2 | 6 | M >>= 1; NZC |
| - [x] | LSR | ABS | `4E` | 3 | 6 | M >>= 1; NZC |
| - [x] | LSR | ABX | `5E` | 3 | 7 | M >>= 1; NZC |
| - [x] | ROL | ACC | `2A` | 1 | 2 | Rotate A left through C; NZC |
| - [x] | ROL | ZP | `26` | 2 | 5 | Rotate M left through C; NZC |
| - [x] | ROL | ZPX | `36` | 2 | 6 | Rotate M left through C; NZC |
| - [x] | ROL | ABS | `2E` | 3 | 6 | Rotate M left through C; NZC |
| - [x] | ROL | ABX | `3E` | 3 | 7 | Rotate M left through C; NZC |
| - [x] | ROR | ACC | `6A` | 1 | 2 | Rotate A right through C; NZC |
| - [x] | ROR | ZP | `66` | 2 | 5 | Rotate M right through C; NZC |
| - [x] | ROR | ZPX | `76` | 2 | 6 | Rotate M right through C; NZC |
| - [x] | ROR | ABS | `6E` | 3 | 6 | Rotate M right through C; NZC |
| - [x] | ROR | ABX | `7E` | 3 | 7 | Rotate M right through C; NZC |

## Branch instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [x] | BCC | REL | `90` | 2 | 2+ | Branch if C = 0 |
| - [x] | BCS | REL | `B0` | 2 | 2+ | Branch if C = 1 |
| - [x] | BEQ | REL | `F0` | 2 | 2+ | Branch if Z = 1 |
| - [x] | BNE | REL | `D0` | 2 | 2+ | Branch if Z = 0 |
| - [x] | BMI | REL | `30` | 2 | 2+ | Branch if N = 1 |
| - [x] | BPL | REL | `10` | 2 | 2+ | Branch if N = 0 |
| - [x] | BVC | REL | `50` | 2 | 2+ | Branch if V = 0 |
| - [x] | BVS | REL | `70` | 2 | 2+ | Branch if V = 1 |

## Jump and subroutine instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects               |
| ----- | ----------- | ---- | ------ | ----: | -----: | ------------------------------------- |
| - [x] | JMP         | ABS  | `4C`   |     3 |      3 | PC = address                          |
| - [x] | JMP         | IND  | `6C`   |     3 |      5 | PC = address stored at pointer        |
| - [x] | JSR         | ABS  | `20`   |     3 |      6 | Push return address; PC = address     |
| - [x] | RTS         | IMP  | `60`   |     1 |      6 | Pull return address; PC = address + 1 |
| - [x] | RTI         | IMP  | `40`   |     1 |      6 | Pull P and PC from stack              |

## Stack instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [x] | PHA | IMP | `48` | 1 | 3 | Push A onto stack |
| - [x] | PHP | IMP | `08` | 1 | 3 | Push P (B and bit 5 set in pushed copy) |
| - [x] | PLA | IMP | `68` | 1 | 4 | A = pulled value; NZ |
| - [x] | PLP | IMP | `28` | 1 | 4 | P = pulled status (B is not a persistent latch) |

## Flag instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | CLC         | IMP  | `18`   |     1 |      2 | C = 0                   |
| - [x] | SEC         | IMP  | `38`   |     1 |      2 | C = 1                   |
| - [x] | CLI         | IMP  | `58`   |     1 |      2 | I = 0                   |
| - [x] | SEI         | IMP  | `78`   |     1 |      2 | I = 1                   |
| - [x] | CLV         | IMP  | `B8`   |     1 |      2 | V = 0                   |
| - [x] | CLD         | IMP  | `D8`   |     1 |      2 | D = 0                   |
| - [x] | SED         | IMP  | `F8`   |     1 |      2 | D = 1                   |

## Other instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects                     |
| ----- | ----------- | ---- | ------ | ----: | -----: | ------------------------------------------- |
| - [x] | BRK         | IMP  | `00`   |    1* |      7 | Push PC+2 and P; set I; jump via IRQ vector |
| - [x] | NOP         | IMP  | `EA`   |     1 |      2 | No effects                                  |
