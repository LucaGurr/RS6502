# RS6502

A MOS 6502 emulator and small assembly runner written in Rust.

## CPU support

The CPU implements all 151 official NMOS 6502 opcodes, including decimal-mode
arithmetic, stack and interrupt operations, zero-page pointer wrapping,
page-cross cycle penalties, and the original indirect-`JMP` page-boundary
behavior. Unofficial opcodes are rejected.

See the [instruction checklist](./docs/instructions.md) and the
[MOS 6500 Family Hardware Manual](./docs/mcs6500_family_hardware_manual.pdf)
and [MOS 6500 Family Programming Manual](./docs/mcs6500_family_programming_manual.pdf).

## Build and test

```sh
cargo test
cargo run
```

The runner prompts for an assembly file in `assembly/`. The built-in assembler
accepts a small, line-oriented 6502 syntax and the MACRO-10-style source used
by the included Microsoft BASIC for 6502. To launch BASIC, run `cargo run` and
enter `m6502.asm`. At startup, enter the available memory size and terminal
width (for example, `8192` and `80`), then type BASIC commands such as
`PRINT 2` or `PRINT "HELLO"`.

The BASIC build uses its simulator configuration (`REALIO=0`) and routes
terminal input and output through emulator hooks. The source remains unchanged
apart from that configuration. Hardware-specific disk and terminal routines
are not emulated.
