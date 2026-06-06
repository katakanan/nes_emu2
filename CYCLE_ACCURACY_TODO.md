# Cycle Accuracy TODO

## Current Status (cycle-accuracy branch)

- ✅ nestest.log matches reference byte-for-byte (CPU instruction cycles correct)
- ✅ Loopy registers (v, t, x, w) implemented per spec
- ✅ NMI: 7 cycles
- ✅ OAMDMA: 513 cycles stall
- ✅ PPU warmup blocks $2000/$2005/$2006 until first VBlank
- ✅ fine_x latched at pre-render line

## Known Issue: giko015 BG enable timing

Our implementation enables BG at sl=103, dot=124 (via $2001 write).
Nestopia enables BG at ~sl=119 (16 scanlines = ~1819 CPU cycles later).

This causes ground to appear ~16 scanlines earlier than expected.

## Investigation Plan

### 1. $2002 race condition
- Real NES: reading $2002 within 2 PPU cycles of VBlank set causes:
  - Returns 0 (VBlank seen as clear)
  - VBlank flag NOT set this frame (suppressed)
- Implementation: track "vblank_set_pending", suppress on race

### 2. NMI assertion delay
- Real NES: 0-3 PPU cycle delay between VBlank flag set and NMI assertion
- Currently: we assert NMI immediately at VBlank set
- May need to delay NMI assertion by 1-3 PPU cycles

### 3. OAMDMA odd/even alignment
- Real NES: 513 cycles if started on "get" cycle, 514 if "put" cycle
- Currently: always 513
- Need: track CPU cycle parity to choose

### 4. CPU-PPU sync order
- Currently: CPU cycle → PPU 3 cycles
- Real NES: concurrent execution
- CPU reads see PPU state from cycle X-1
- CPU writes affect PPU state from cycle X+

### 5. $2007 dual-clock during rendering
- Real NES: $2007 write during rendering triggers coarse_x AND y increments simultaneously
- Currently: simple v += 1 or 32
- Already attempted but caused issues — needs revisit

### 6. Additional PPU edge cases
- Sprite 0 hit detection cycle timing (currently per-pixel via shift register check)
- Sprite overflow flag (not implemented)
- 8x16 sprite mode (not implemented)
- Color emphasis bits (not implemented)
- Grayscale bit (not implemented)
