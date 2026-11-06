# MOS 6502 Instruction Implementation Checklist

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
| - [ ] | LDY         | ABS  | `AC`   |     3 |      4 | Y = M; NZ               |
| - [ ] | LDY         | ABX  | `BC`   |     3 |     4+ | Y = M; NZ               |
| - [x] | STA         | ZP   | `85`   |     2 |      3 | M = A                   |
| - [x] | STA         | ZPX  | `95`   |     2 |      4 | M = A                   |
| - [x] | STA         | ABS  | `8D`   |     3 |      4 | M = A                   |
| - [ ] | STA         | ABX  | `9D`   |     3 |      5 | M = A                   |
| - [ ] | STA         | ABY  | `99`   |     3 |      5 | M = A                   |
| - [ ] | STA         | INX  | `81`   |     2 |      6 | M = A                   |
| - [ ] | STA         | INY  | `91`   |     2 |      6 | M = A                   |
| - [x] | STX         | ZP   | `86`   |     2 |      3 | M = X                   |
| - [ ] | STX         | ZPY  | `96`   |     2 |      4 | M = X                   |
| - [ ] | STX         | ABS  | `8E`   |     3 |      4 | M = X                   |
| - [ ] | STY         | ZP   | `84`   |     2 |      3 | M = Y                   |
| - [ ] | STY         | ZPX  | `94`   |     2 |      4 | M = Y                   |
| - [ ] | STY         | ABS  | `8C`   |     3 |      4 | M = Y                   |

## Register transfer instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [ ] | TAX | IMP | `AA` | 1 | 2 | X = A; NZ |
| - [ ] | TAY | IMP | `A8` | 1 | 2 | Y = A; NZ |
| - [ ] | TXA | IMP | `8A` | 1 | 2 | A = X; NZ |
| - [ ] | TYA | IMP | `98` | 1 | 2 | A = Y; NZ |
| - [ ] | TSX | IMP | `BA` | 1 | 2 | X = SP; NZ |
| - [ ] | TXS | IMP | `9A` | 1 | 2 | SP = X; flags unchanged |

## Arithmetic instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | ADC         | IMM  | `69`   |     2 |      2 | A = A + M + C; NZCV     |
| - [ ] | ADC         | ZP   | `65`   |     2 |      3 | A = A + M + C; NZCV     |
| - [ ] | ADC         | ZPX  | `75`   |     2 |      4 | A = A + M + C; NZCV     |
| - [x] | ADC         | ABS  | `6D`   |     3 |      4 | A = A + M + C; NZCV     |
| - [x] | ADC         | ABX  | `7D`   |     3 |     4+ | A = A + M + C; NZCV     |
| - [ ] | ADC         | ABY  | `79`   |     3 |     4+ | A = A + M + C; NZCV     |
| - [ ] | ADC         | INX  | `61`   |     2 |      6 | A = A + M + C; NZCV     |
| - [ ] | ADC         | INY  | `71`   |     2 |     5+ | A = A + M + C; NZCV     |
| - [ ] | SBC         | IMM  | `E9`   |     2 |      2 | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | ZP   | `E5`   |     2 |      3 | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | ZPX  | `F5`   |     2 |      4 | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | ABS  | `ED`   |     3 |      4 | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | ABX  | `FD`   |     3 |     4+ | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | ABY  | `F9`   |     3 |     4+ | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | INX  | `E1`   |     2 |      6 | A = A - M - (1-C); NZCV |
| - [ ] | SBC         | INY  | `F1`   |     2 |     5+ | A = A - M - (1-C); NZCV |
| - [ ] | INC         | ZP   | `E6`   |     2 |      5 | M = M + 1; NZ           |
| - [ ] | INC         | ZPX  | `F6`   |     2 |      6 | M = M + 1; NZ           |
| - [ ] | INC         | ABS  | `EE`   |     3 |      6 | M = M + 1; NZ           |
| - [ ] | INC         | ABX  | `FE`   |     3 |      7 | M = M + 1; NZ           |
| - [x] | INX         | IMP  | `E8`   |     1 |      2 | X = X + 1; NZ           |
| - [ ] | INY         | IMP  | `C8`   |     1 |      2 | Y = Y + 1; NZ           |
| - [ ] | DEC         | ZP   | `C6`   |     2 |      5 | M = M - 1; NZ           |
| - [ ] | DEC         | ZPX  | `D6`   |     2 |      6 | M = M - 1; NZ           |
| - [ ] | DEC         | ABS  | `CE`   |     3 |      6 | M = M - 1; NZ           |
| - [ ] | DEC         | ABX  | `DE`   |     3 |      7 | M = M - 1; NZ           |
| - [x] | DEX         | IMP  | `CA`   |     1 |      2 | X = X - 1; NZ           |
| - [ ] | DEY         | IMP  | `88`   |     1 |      2 | Y = Y - 1; NZ           |

## Logical instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [ ] | AND | IMM | `29` | 2 | 2 | A = A AND M; NZ |
| - [ ] | AND | ZP | `25` | 2 | 3 | A = A AND M; NZ |
| - [ ] | AND | ZPX | `35` | 2 | 4 | A = A AND M; NZ |
| - [ ] | AND | ABS | `2D` | 3 | 4 | A = A AND M; NZ |
| - [ ] | AND | ABX | `3D` | 3 | 4+ | A = A AND M; NZ |
| - [ ] | AND | ABY | `39` | 3 | 4+ | A = A AND M; NZ |
| - [ ] | AND | INX | `21` | 2 | 6 | A = A AND M; NZ |
| - [ ] | AND | INY | `31` | 2 | 5+ | A = A AND M; NZ |
| - [ ] | ORA | IMM | `09` | 2 | 2 | A = A OR M; NZ |
| - [ ] | ORA | ZP | `05` | 2 | 3 | A = A OR M; NZ |
| - [ ] | ORA | ZPX | `15` | 2 | 4 | A = A OR M; NZ |
| - [ ] | ORA | ABS | `0D` | 3 | 4 | A = A OR M; NZ |
| - [ ] | ORA | ABX | `1D` | 3 | 4+ | A = A OR M; NZ |
| - [ ] | ORA | ABY | `19` | 3 | 4+ | A = A OR M; NZ |
| - [ ] | ORA | INX | `01` | 2 | 6 | A = A OR M; NZ |
| - [ ] | ORA | INY | `11` | 2 | 5+ | A = A OR M; NZ |
| - [ ] | EOR | IMM | `49` | 2 | 2 | A = A XOR M; NZ |
| - [ ] | EOR | ZP | `45` | 2 | 3 | A = A XOR M; NZ |
| - [ ] | EOR | ZPX | `55` | 2 | 4 | A = A XOR M; NZ |
| - [ ] | EOR | ABS | `4D` | 3 | 4 | A = A XOR M; NZ |
| - [ ] | EOR | ABX | `5D` | 3 | 4+ | A = A XOR M; NZ |
| - [ ] | EOR | ABY | `59` | 3 | 4+ | A = A XOR M; NZ |
| - [ ] | EOR | INX | `41` | 2 | 6 | A = A XOR M; NZ |
| - [ ] | EOR | INY | `51` | 2 | 5+ | A = A XOR M; NZ |
| - [ ] | BIT | ZP | `24` | 2 | 3 | Z = !(A AND M); N = M7; V = M6 |
| - [ ] | BIT | ABS | `2C` | 3 | 4 | Z = !(A AND M); N = M7; V = M6 |

## Compare instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | CMP         | IMM  | `C9`   |     2 |      2 | Compare A with M; NZC   |
| - [ ] | CMP         | ZP   | `C5`   |     2 |      3 | Compare A with M; NZC   |
| - [ ] | CMP         | ZPX  | `D5`   |     2 |      4 | Compare A with M; NZC   |
| - [ ] | CMP         | ABS  | `CD`   |     3 |      4 | Compare A with M; NZC   |
| - [ ] | CMP         | ABX  | `DD`   |     3 |     4+ | Compare A with M; NZC   |
| - [ ] | CMP         | ABY  | `D9`   |     3 |     4+ | Compare A with M; NZC   |
| - [ ] | CMP         | INX  | `C1`   |     2 |      6 | Compare A with M; NZC   |
| - [ ] | CMP         | INY  | `D1`   |     2 |     5+ | Compare A with M; NZC   |
| - [ ] | CPX         | IMM  | `E0`   |     2 |      2 | Compare X with M; NZC   |
| - [ ] | CPX         | ZP   | `E4`   |     2 |      3 | Compare X with M; NZC   |
| - [ ] | CPX         | ABS  | `EC`   |     3 |      4 | Compare X with M; NZC   |
| - [ ] | CPY         | IMM  | `C0`   |     2 |      2 | Compare Y with M; NZC   |
| - [ ] | CPY         | ZP   | `C4`   |     2 |      3 | Compare Y with M; NZC   |
| - [ ] | CPY         | ABS  | `CC`   |     3 |      4 | Compare Y with M; NZC   |

## Shift and rotate instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [ ] | ASL | ACC | `0A` | 1 | 2 | A <<= 1; NZC |
| - [ ] | ASL | ZP | `06` | 2 | 5 | M <<= 1; NZC |
| - [ ] | ASL | ZPX | `16` | 2 | 6 | M <<= 1; NZC |
| - [ ] | ASL | ABS | `0E` | 3 | 6 | M <<= 1; NZC |
| - [ ] | ASL | ABX | `1E` | 3 | 7 | M <<= 1; NZC |
| - [ ] | LSR | ACC | `4A` | 1 | 2 | A >>= 1; NZC |
| - [ ] | LSR | ZP | `46` | 2 | 5 | M >>= 1; NZC |
| - [ ] | LSR | ZPX | `56` | 2 | 6 | M >>= 1; NZC |
| - [ ] | LSR | ABS | `4E` | 3 | 6 | M >>= 1; NZC |
| - [ ] | LSR | ABX | `5E` | 3 | 7 | M >>= 1; NZC |
| - [ ] | ROL | ACC | `2A` | 1 | 2 | Rotate A left through C; NZC |
| - [ ] | ROL | ZP | `26` | 2 | 5 | Rotate M left through C; NZC |
| - [ ] | ROL | ZPX | `36` | 2 | 6 | Rotate M left through C; NZC |
| - [ ] | ROL | ABS | `2E` | 3 | 6 | Rotate M left through C; NZC |
| - [ ] | ROL | ABX | `3E` | 3 | 7 | Rotate M left through C; NZC |
| - [ ] | ROR | ACC | `6A` | 1 | 2 | Rotate A right through C; NZC |
| - [ ] | ROR | ZP | `66` | 2 | 5 | Rotate M right through C; NZC |
| - [ ] | ROR | ZPX | `76` | 2 | 6 | Rotate M right through C; NZC |
| - [ ] | ROR | ABS | `6E` | 3 | 6 | Rotate M right through C; NZC |
| - [ ] | ROR | ABX | `7E` | 3 | 7 | Rotate M right through C; NZC |

## Branch instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [ ] | BCC | REL | `90` | 2 | 2+ | Branch if C = 0 |
| - [ ] | BCS | REL | `B0` | 2 | 2+ | Branch if C = 1 |
| - [ ] | BEQ | REL | `F0` | 2 | 2+ | Branch if Z = 1 |
| - [ ] | BNE | REL | `D0` | 2 | 2+ | Branch if Z = 0 |
| - [ ] | BMI | REL | `30` | 2 | 2+ | Branch if N = 1 |
| - [ ] | BPL | REL | `10` | 2 | 2+ | Branch if N = 0 |
| - [ ] | BVC | REL | `50` | 2 | 2+ | Branch if V = 0 |
| - [ ] | BVS | REL | `70` | 2 | 2+ | Branch if V = 1 |

## Jump and subroutine instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects               |
| ----- | ----------- | ---- | ------ | ----: | -----: | ------------------------------------- |
| - [x] | JMP         | ABS  | `4C`   |     3 |      3 | PC = address                          |
| - [ ] | JMP         | IND  | `6C`   |     3 |      5 | PC = address stored at pointer        |
| - [ ] | JSR         | ABS  | `20`   |     3 |      6 | Push return address; PC = address     |
| - [ ] | RTS         | IMP  | `60`   |     1 |      6 | Pull return address; PC = address + 1 |
| - [ ] | RTI         | IMP  | `40`   |     1 |      6 | Pull P and PC from stack              |

## Stack instructions

| Done | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
|---|---|---|---|---:|---:|---|
| - [ ] | PHA | IMP | `48` | 1 | 3 | Push A onto stack |
| - [ ] | PHP | IMP | `08` | 1 | 3 | Push P (B and bit 5 set in pushed copy) |
| - [ ] | PLA | IMP | `68` | 1 | 4 | A = pulled value; NZ |
| - [ ] | PLP | IMP | `28` | 1 | 4 | P = pulled status (B is not a persistent latch) |

## Flag instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects |
| ----- | ----------- | ---- | ------ | ----: | -----: | ----------------------- |
| - [x] | CLC         | IMP  | `18`   |     1 |      2 | C = 0                   |
| - [ ] | SEC         | IMP  | `38`   |     1 |      2 | C = 1                   |
| - [ ] | CLI         | IMP  | `58`   |     1 |      2 | I = 0                   |
| - [ ] | SEI         | IMP  | `78`   |     1 |      2 | I = 1                   |
| - [ ] | CLV         | IMP  | `B8`   |     1 |      2 | V = 0                   |
| - [ ] | CLD         | IMP  | `D8`   |     1 |      2 | D = 0                   |
| - [ ] | SED         | IMP  | `F8`   |     1 |      2 | D = 1                   |

## Other instructions

| Done  | Instruction | Mode | Opcode | Bytes | Cycles | Register / flag effects                     |
| ----- | ----------- | ---- | ------ | ----: | -----: | ------------------------------------------- |
| - [x] | BRK         | IMP  | `00`   |    1* |      7 | Push PC+2 and P; set I; jump via IRQ vector |
| - [x] | NOP         | IMP  | `EA`   |     1 |      2 | No effects                                  |
