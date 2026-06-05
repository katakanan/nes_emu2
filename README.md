# nes_emu

A Nintendo Entertainment System (NES) emulator written in Rust, built as a learning project to explore cycle-accurate emulation techniques using Rust's nightly generator feature.

## Design Philosophy

### Generator-based cycle stepping

The central design choice is using Rust's unstable `Generator` trait (`#![feature(generators)]`) to model the CPU and PPU as coroutines. Each component (`Cpu::run`, `Ppu::run`) is a generator that `yield`s one step at a time — a single CPU cycle yields `CpuStep::Cycle`, a decoded instruction yields `CpuStep::Op(...)`, and so on. The top-level `Nes::run` generator interleaves these two streams, advancing the PPU three cycles for every one CPU cycle to match the NES hardware ratio.

This approach keeps timing logic close to the hardware model: instead of simulating time with counters and callbacks, execution literally suspends at each cycle boundary.

### Interior mutability with `Cell`

The `Nes` struct holds the CPU, PPU, RAM, and pad as owned fields, and the generators borrow `&'a Nes` for their entire lifetime. To allow mutation through a shared reference, all mutable hardware state uses `Cell<T>` (or `RefCell` for the framebuffer). This is a deliberate trade-off: it avoids complex lifetime splitting while keeping the borrow structure simple.

### Addressable bus

Memory-mapped I/O is centralised in `Nes::read8` / `Nes::write8`, which dispatch on the address range:

| Range | Device |
|---|---|
| `0x0000–0x07FF` | 2 KB WRAM (mirrored to `0x1FFF`) |
| `0x2000–0x3FFF` | PPU registers (mirrored every 8 bytes) |
| `0x4014` | OAM DMA |
| `0x4016` | Controller 1 |
| `0x8000–0xFFFF` | PRG-ROM (mirrored for NROM-128) |

### Undocumented opcodes

The CPU implements a broad set of unofficial 6502 instructions (LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA, and the various unofficial NOPs) so that games relying on them work correctly.

### Debug and testing features

Two Cargo features gate optional behaviour:

- `nestest` — sets the reset vector to `$C000` and writes a nestest-compatible log (`nestest.log`) for automated CPU verification against the reference log.
- `steprun` — reserved for single-step execution tracing.

The `yield_all!` macro in [src/misc.rs](src/misc.rs) propagates `yield` points from inner generators up through the call stack, enabling addressing-mode and operation generators to be composed cleanly.

## Architecture

```
main.rs          — event loop, piston_window integration, render sync
nes.rs           — Nes struct, bus, run() generator orchestration
cpu.rs           — 6502 CPU state, run() generator, debug formatting
ppu.rs           — PPU state, rendering pipeline, NMI generation
operations.rs    — ALU and control-flow instruction implementations
addressings.rs   — addressing mode generators (Immediate, ZeroPage, Absolute, …)
opcode.rs        — opcode enum (official + undocumented)
pad.rs           — controller shift-register emulation
view.rs          — piston_window framebuffer display
control.rs       — keyboard → NES button mapping
cparams.rs       — timing constants (frame size, scanline counts)
misc.rs          — yield_all! macro, FPS counter
```

## Dependencies

| Crate | Purpose |
|---|---|
| `piston_window` | Window creation, rendering, input events |
| `image` | RGBA framebuffer for PPU output |
| `bitflags` | CPU status register and PPU control register flags |
| `nes-rom-reader` | iNES ROM parsing (local crate) |
| `num-derive` / `num-traits` | `FromPrimitive` for opcode byte → enum conversion |

## Building

Requires a **nightly** Rust toolchain due to the generator feature.

```sh
rustup override set nightly
cargo build --release
```

To run the nestest CPU validation suite:

```sh
cargo run --features nestest
diff nestest.log /path/to/reference/nestest.log
```

## Status

- CPU: all official opcodes + common undocumented opcodes
- PPU: background rendering, sprite rendering, NMI
- Mapper: NROM (mapper 0) only
- APU: not implemented (reads return `0xFF`)
- Audio: not implemented
