use super::cparams::{
    FRAME_H, FRAME_W, POST_RENDER_LINE, PRE_RENDER_LINE, SET_VBLANK_LINE, VFRAME_END, VFRAME_H,
    VFRAME_W,
};
use super::Nes;
use crate::cparams;
use bitflags::bitflags;
use image;
use std::cell::{Cell, RefCell};
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum MIRROR {
    HORIZONTAL,
    VERTICAL,
    ONESCREEN_LO,
    ONESCREEN_HI,
}

#[derive(Debug)]
pub struct Ppu {
    pub chrrom: [u8; 0x2000], //readonly
    pub vram: Cell<[u8; 0x1000]>,
    pub oam: Cell<[u8; 0x0100]>,
    pub palette: Cell<[u8; 0x0020]>,
    pub addr: Cell<u16>,
    pub ctrl: Cell<PpuCtrl>,
    pub mask: Cell<PpuMask>,
    pub status: Cell<PpuStatus>,
    pub oamaddr: Cell<u8>,
    pub scroll: Cell<u16>,
    pub latch: Cell<bool>,
    pub img: RefCell<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    pub data_buf2006: Cell<u8>,
    // secondary_oam: Cell<[u8; 32]>, //4byte * 8 sprites
    secondary_oam: Cell<[SpriteInfo; 8]>,
    // sprite_zero_flag: Cell<[bool; 8]>,
    sprite_output_units: Cell<[SpriteOutputUnit; 8]>,

    //For Nabel Table
    pub nt_shift_reg_hi: Cell<u32>, /* [8bit][8bit] */
    pub nt_shift_reg_lo: Cell<u32>, /* [8bit][8bit] */

    //For Attribute Table
    pub at_shift_reg_hi: Cell<u32>, /* [8bit][1bit latch] */
    pub at_shift_reg_lo: Cell<u32>, /* [8bit][1bit latch] */

    // pub shift_reg8: Cell<u8>,
    pub mirror: Cell<MIRROR>,

    //for debue
    // pub sel: RefCell<u8>,
    pub grid_on: Cell<bool>,
    pub debug_num: Cell<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum PpuStep {
    Cycle(RenderStep, u32),
    Vblank,
}

//RenderStep is same timing as PpuStep
#[derive(Debug, Clone, Copy)]
pub enum RenderStep {
    Cycle(u32, u32, u32), //frame, x, y
    Vblank,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct PpuCtrl: u8 {
        const NAMETABLE_LO = 1 << 0;
        const NAMETABLE_HI = 1 << 1;
        const VRAM_ADDR_INCREMENT = 1 << 2;
        const SPRITE_PATTERN_TABLE_ADDR = 1 << 3;
        const BACKGROUND_PATTERN_TABLE_ADDR = 1 << 4;
        const SPRITE_SIZE = 1 << 5;
        const PPU_MASTER_SLAVE_SELECT = 1 << 6;
        const VBLANK_INTERRUPT = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct PpuMask: u8 {
        const GREYSCALE = 1 << 0;
        const SHOW_BACKGROUND_IN_LEFT_MARGIN = 1 << 1;
        const SHOW_SPRITES_IN_LEFT_MARGIN = 1 << 2;
        const SHOW_BACKGROUND = 1 << 3;
        const SHOW_SPRITES = 1 << 4;
        const EMPHASIZE_RED = 1 << 5;
        const EMPHASIZE_GREEN = 1 << 6;
        const EMPHASIZE_BLUE = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct PpuStatus: u8{
        const OVERFLOW = 1 << 5;
        const ZERO_HIT = 1 << 6;
        const VBLANK_STARTED = 1 << 7;
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TileInfo {
    ld_x: u32,
    ld_y: u32,
    sprite_index: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteInfo {
    y: u8,
    id: u8,
    attr: u8,
    x: u8,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct SpriteOutputUnit {
    sprite_lsbits: u8,
    sprite_msbits: u8,
    x: u8,
    sprite_info: SpriteInfo,
}

impl Default for SpriteInfo {
    fn default() -> Self {
        Self {
            y: 0xFF,
            id: 0xFF,
            attr: 0xFF,
            x: 0xFF,
        }
    }
}

impl Ppu {
    fn renderer<'a>(nes: &'a Nes) -> impl Coroutine<Yield = RenderStep, Return = !> + 'a {
        let mut buf = image::RgbaImage::new(VFRAME_W, VFRAME_H);

        #[coroutine] move || loop {
            for frame in 0.. {
                for scanline in 0..FRAME_H {
                    let y = scanline as u32;

                    match y {
                        0..=VFRAME_END => {
                            //0 - 239
                            //Visible Frame
                            for x in 0..FRAME_W as u32 {
                                if x < VFRAME_W {
                                    let bg_color_sel = Ppu::calc_bg_pixel_color_sel(nes);
                                    // let sp_color_sel = Ppu::calc_sp_pixel_color_sel(nes);

                                    let bg_palette_sel = Ppu::calc_bg_pixel_palette_sel(nes);
                                    // let (sp_palette_sel, sp_priority) =
                                    //     Ppu::calc_sp_pixel_palette_sel_and_priority(nes);

                                    let (sp_color_sel, sp_palette_sel, sp_priority) =
                                        Ppu::calc_sp_pixel_color_sel_and_palette_sel_and_priority(
                                            nes,
                                        );

                                    // Sprite 0 hit detection
                                    let sprite_output_units = nes.ppu.sprite_output_units.get();
                                    let is_sprite0_active = sprite_output_units
                                        .first()
                                        .map(|u| u.x == 0 && (u.sprite_lsbits & 0x80 != 0 || u.sprite_msbits & 0x80 != 0))
                                        .unwrap_or(false);
                                    if bg_color_sel != 0 && sp_color_sel != 0 && is_sprite0_active && x < 255 {
                                        nes.ppu.status.update(|s| s | PpuStatus::ZERO_HIT);
                                    }

                                    let (color_sel, palette_sel) =
                                        match (bg_color_sel, sp_color_sel) {
                                            _ if bg_color_sel == 0 && sp_color_sel == 0 => (0, 0),
                                            _ if bg_color_sel == 0 && sp_color_sel > 0 => {
                                                (sp_color_sel, sp_palette_sel)
                                            }
                                            _ if bg_color_sel > 0 && sp_color_sel == 0 => {
                                                (bg_color_sel, bg_palette_sel)
                                            }
                                            _ if bg_color_sel > 0 && sp_color_sel > 0 => {
                                                //check sprite_priority
                                                if sp_priority {
                                                    (sp_color_sel, sp_palette_sel)
                                                } else {
                                                    (bg_color_sel, bg_palette_sel)
                                                }
                                            }
                                            _ => unreachable!(),
                                        };
                                    // let (color_sel, palette_sel) = (bg_color_sel, bg_palette_sel);

                                    let color_index =
                                        Ppu::ld_color_index(nes, palette_sel, color_sel);

                                    let color = cparams::COLORS[color_index];
                                    let r = color[0];
                                    let g = color[1];
                                    let b = color[2];

                                    let pixel = if Ppu::is_grid_line(nes, x, y, 8) {
                                        image::Rgba([255, 255, 0, 100])
                                    } else {
                                        image::Rgba([r, g, b, 255])
                                    };
                                    buf.put_pixel(x, y, pixel);
                                }
                                yield RenderStep::Cycle(frame, x, y);
                            }
                        }
                        POST_RENDER_LINE => {
                            //240
                            //nothing
                            for x in 0..FRAME_W as u32 {
                                yield RenderStep::Cycle(frame, x, y);
                            }
                        }
                        SET_VBLANK_LINE => {
                            for x in 0..FRAME_W as u32 {
                                if x == 1 {
                                    nes.ppu
                                        .status
                                        .update(|status| status | PpuStatus::VBLANK_STARTED);

                                    if nes.ppu.ctrl.get().contains(PpuCtrl::VBLANK_INTERRUPT) {
                                        nes.cpu.nmi.set(true);
                                    }

                                    *nes.ppu.img.borrow_mut() = buf.clone();
                                    yield RenderStep::Vblank;
                                }

                                yield RenderStep::Cycle(frame, x, y);
                            }

                            // *nes.ppu.img.borrow_mut() = buf.clone();
                        }
                        PRE_RENDER_LINE => {
                            //261
                            for x in 0..FRAME_W as u32 {
                                if x == 1 {
                                    nes.ppu.status.update(|status| {
                                        status & !(PpuStatus::VBLANK_STARTED | PpuStatus::ZERO_HIT)
                                    });
                                }
                                yield RenderStep::Cycle(frame, x, y);
                            }
                        }
                        _ => {
                            for x in 0..FRAME_W as u32 {
                                yield RenderStep::Cycle(frame, x, y);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn is_grid_line(nes: &Nes, x: u32, y: u32, step: u32) -> bool {
        let grid_on = nes.ppu.grid_on.get();
        let grid_x = ((x + 1) % step) == 0;
        let grid_y = ((y + 1) % step) == 0;
        grid_on && (grid_x || grid_y)
    }

    pub fn calc_sp_pixel_color_sel_and_palette_sel_and_priority(nes: &Nes) -> (usize, usize, bool) {
        let calc_color_sel = |sprite_output_unit: SpriteOutputUnit| {
            let lo_bit = (sprite_output_unit.sprite_lsbits & 0x80 != 0) as usize;
            let hi_bit = (sprite_output_unit.sprite_msbits & 0x80 != 0) as usize;
            let color_sel = hi_bit << 1 | lo_bit;
            color_sel
        };

        let calc_palette_and_priority = |sprite_output_unit: SpriteOutputUnit| {
            let palette_sel = (sprite_output_unit.sprite_info.attr & 0x03) as usize + 0x04;
            let priority = sprite_output_unit.sprite_info.attr & 0x20 == 0;
            (palette_sel, priority)
        };

        let sprite_output_units = nes.ppu.sprite_output_units.get();
        let res = sprite_output_units.into_iter().find(|&sprite_output_unit| {
            (sprite_output_unit.x == 0) && (calc_color_sel(sprite_output_unit) != 0)
        });

        let ret = match res {
            Some(sprite_output_unit) => {
                let color_sel = calc_color_sel(sprite_output_unit);
                let (palette, priority) = calc_palette_and_priority(sprite_output_unit);
                (color_sel, palette, priority)
            }
            None => (0, 0, false),
        };

        ret
    }

    pub fn calc_sp_pixel_color_sel(nes: &Nes) -> usize {
        //if x == 0
        //then return color code
        let sprite_output_units = nes.ppu.sprite_output_units.get();
        let res = sprite_output_units
            .into_iter()
            .find(|&sprite_output_unit| sprite_output_unit.x == 0);
        let color_sel = match res {
            Some(sprite_output_unit) => {
                let lo_bit = sprite_output_unit.sprite_lsbits & 0x80 != 0;
                let hi_bit = sprite_output_unit.sprite_msbits & 0x80 != 0;

                let sel = match (hi_bit, lo_bit) {
                    (false, false) => 0,
                    (false, true) => 1,
                    (true, false) => 2,
                    (true, true) => 3,
                };

                sel
            }
            _ => 0,
        };

        color_sel
    }

    pub fn calc_sp_pixel_palette_sel_and_priority(nes: &Nes) -> (usize, bool) {
        let sprite_output_units = nes.ppu.sprite_output_units.get();
        let res = sprite_output_units
            .into_iter()
            .find(|&sprite_output_unit| sprite_output_unit.x == 0);
        let sel_and_priority = match res {
            Some(sprite_output_unit) => {
                let sel = (sprite_output_unit.sprite_info.attr & 0x03 + 0x04) as usize;
                let priority = sprite_output_unit.sprite_info.attr & 0x20 == 0;
                (sel, priority)
            }
            _ => (0, false),
        };
        sel_and_priority
    }

    pub fn calc_bg_pixel_color_sel(nes: &Nes) -> usize {
        let hi_byte = nes.ppu.nt_shift_reg_hi.get();
        let lo_byte = nes.ppu.nt_shift_reg_lo.get();
        let mask = 0x8000_0000;
        let hi_bit = (hi_byte & mask) != 0;
        let lo_bit = (lo_byte & mask) != 0;

        let index = match (hi_bit, lo_bit) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        index
    }

    pub fn calc_bg_pixel_palette_sel(nes: &Nes) -> usize {
        let hi_byte = nes.ppu.at_shift_reg_hi.get();
        let lo_byte = nes.ppu.at_shift_reg_lo.get();
        let mask = 0x8000_0000;
        let hi_bit = (hi_byte & mask) != 0;
        let lo_bit = (lo_byte & mask) != 0;

        let index = match (hi_bit, lo_bit) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        index
    }

    pub fn ld_color_index(nes: &Nes, palette_sel: usize, color_sel: usize) -> usize {
        let palette_base_addr = 0x3F00;
        let palette_ld_addr = 4 * palette_sel + color_sel + palette_base_addr;

        let addr = if color_sel == 0 {
            0x3F00
        } else {
            palette_ld_addr
        } as u16;

        let color_index = nes.ppu.read8(addr);
        color_index as usize
    }

    fn nt_evaluation<'a>(nes: &'a Nes) -> impl Coroutine<Yield = RenderStep, Return = !> + 'a {
        #[coroutine] move || loop {
            let mut tile_info = TileInfo::default();
            for frame in 0.. {
                for scanline in 0..FRAME_H {
                    let y = scanline as u32;
                    match y {
                        PRE_RENDER_LINE | 0..=VFRAME_END => {
                            for x in 0..FRAME_W as u32 {
                                if Ppu::ppu_line_timing(x, y, 1) {
                                    let (ld_x, ld_y) = Ppu::calc_ld_nt_coord(x, y);
                                    let nt_addr = Ppu::calc_ld_nt_addr(nes, ld_x, ld_y);

                                    let sprite_index = nes.ppu.read8(nt_addr);

                                    tile_info = TileInfo {
                                        ld_x,
                                        ld_y,
                                        sprite_index,
                                    };
                                } else if Ppu::ppu_line_timing(x, y, 3) {
                                    let ld_x = tile_info.ld_x;
                                    let ld_y = tile_info.ld_y;
                                    let at_addr = Ppu::calc_ld_at_addr(nes, ld_x, ld_y);
                                    let bg_at_byte = nes.ppu.read8(at_addr);
                                    let (hi_bit, lo_bit) =
                                        Ppu::calc_at_at_bit(ld_x, ld_y, bg_at_byte);
                                    let shift_offset = if x < 249 { 0 } else { 5 };
                                    Ppu::update_at_shiftregs_w_offset(
                                        nes,
                                        hi_bit,
                                        lo_bit,
                                        shift_offset,
                                    );
                                } else if Ppu::ppu_line_timing(x, y, 7) {
                                    let ld_x = tile_info.ld_x;
                                    let ld_y = tile_info.ld_y;
                                    let sprite_index = tile_info.sprite_index;
                                    let bg_sprite_line_addr =
                                        Ppu::calc_nt_sprite_addr(nes, ld_x, ld_y, sprite_index);
                                    let bitmap_bytes =
                                        Ppu::calc_nt_color_index(nes, bg_sprite_line_addr);
                                    let shift_offset = if x < 249 { 0 } else { 5 };
                                    Ppu::update_nt_shiftregs_w_offset(
                                        nes,
                                        bitmap_bytes.0,
                                        bitmap_bytes.1,
                                        shift_offset,
                                    );
                                }
                                yield RenderStep::Cycle(0, 0, 0);
                            }
                        }
                        _ => {
                            for _ in 0..FRAME_W {
                                yield RenderStep::Cycle(0, 0, 0);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn ppu_line_timing(ev_x: u32, ev_y: u32, offset: u32) -> bool {
        let a = ev_x % 8 == offset;
        let b = ev_x < 249 || (321 <= ev_x && ev_x < 337);
        // let b = true;
        let c = ev_y < VFRAME_H || ev_y == PRE_RENDER_LINE;
        // let c = true;
        a && b && c
    }

    pub fn calc_ld_at_coord(ev_x: u32, ev_y: u32) -> (u32, u32) {
        // let ld_x = (ev_x + 8 * 4) % 336;
        let ld_x_offset = 20;
        let ld_x = (ev_x + ld_x_offset) % FRAME_W as u32;
        let ld_y = (if ld_x < ev_x { ev_y + 1 } else { ev_y }) % (PRE_RENDER_LINE + 1);
        (ld_x, ld_y)
    }

    pub fn calc_ld_at_addr(nes: &Nes, ld_x: u32, ld_y: u32) -> u16 {
        let at_x = (ld_x >> 5) % 32;
        let at_y = (ld_y >> 5) % 32;
        let at_addr_offset = at_x + at_y * (VFRAME_W >> 5);
        let at_addr_base = 0x23C0;
        (at_addr_base + at_addr_offset) as u16
    }

    pub fn calc_at_at_bit(ld_x: u32, ld_y: u32, at_byte: u8) -> (u8, u8) {
        let lsb = ((ld_x & (1 << 4)) != 0) as u8;
        let msb = ((ld_y & (1 << 4)) != 0) as u8;

        let shift = msb * 2 + lsb;
        let at_2bits = (at_byte >> shift * 2) & 0b0000_0011;
        let upper_bit = if (at_2bits & 0b10) != 0 { 0xFF } else { 0 };
        let lower_bit = if (at_2bits & 0b01) != 0 { 0xFF } else { 0 };
        (upper_bit, lower_bit)
    }

    pub fn update_at_shiftregs_w_offset(nes: &Nes, hi_byte: u8, lo_byte: u8, offset: u8) {
        let hi = nes.ppu.at_shift_reg_hi.get();
        let lo = nes.ppu.at_shift_reg_lo.get();
        let hi_shift = (hi_byte as u32) << (16 - offset);
        let lo_shift = (lo_byte as u32) << (16 - offset);
        nes.ppu.at_shift_reg_hi.set(hi | hi_shift);
        nes.ppu.at_shift_reg_lo.set(lo | lo_shift);
    }

    pub fn calc_ld_nt_coord(ev_x: u32, ev_y: u32) -> (u32, u32) {
        //ev_x : 1, 9, 17, ... , 65, ...
        //ev_y : 0, 1, 2, ... 239, 261
        // let ld_x = (ev_x + 8 * 1) % 337;
        let ld_x_offset = if ev_x < 249 { 15 } else { 20 };
        // println!("{}", ev_x);
        // let ld_x_offset = 0;
        let ld_x = (ev_x + ld_x_offset) % FRAME_W as u32;
        let ld_y = (if ld_x < ev_x { ev_y + 1 } else { ev_y }) % FRAME_H as u32;
        (ld_x, ld_y)
    }

    pub fn calc_ld_nt_addr(nes: &Nes, ld_x: u32, ld_y: u32) -> u16 {
        let nt_x = (ld_x >> 3) % 32;
        let nt_y = (ld_y >> 3) % 32;
        let nt_addr_offset = nt_x + nt_y * (VFRAME_W >> 3);
        let nt_addr_base = 0x2000;
        (nt_addr_base + nt_addr_offset) as u16
    }

    pub fn calc_nt_sprite_addr(nes: &Nes, _ld_x: u32, ld_y: u32, sprite_index: u8) -> u16 {
        let nt_pattern_dy = (ld_y % 8) as u16;
        let nt_pattern_id = (sprite_index as u16) << 4;
        let nt_offset_addr = (nes
            .ppu
            .ctrl
            .get()
            .contains(PpuCtrl::BACKGROUND_PATTERN_TABLE_ADDR) as u16)
            << 12;

        let nt_sprite_addr = nt_offset_addr | nt_pattern_id | nt_pattern_dy;
        nt_sprite_addr
    }

    pub fn calc_nt_color_index(nes: &Nes, bg_sprite_line_addr: u16) -> (u8, u8) {
        let bitmap_lo_byte = nes.ppu.read8(bg_sprite_line_addr);
        let bitmap_hi_byte = nes.ppu.read8(bg_sprite_line_addr.wrapping_add(8));
        (bitmap_hi_byte, bitmap_lo_byte)
    }

    pub fn update_nt_shiftregs_w_offset(nes: &Nes, hi_byte: u8, lo_byte: u8, offset: u8) {
        let hi = nes.ppu.nt_shift_reg_hi.get();
        let lo = nes.ppu.nt_shift_reg_lo.get();
        let hi_shift = (hi_byte as u32) << (16 - offset);
        let lo_shift = (lo_byte as u32) << (16 - offset);
        nes.ppu.nt_shift_reg_hi.set(hi | hi_shift);
        nes.ppu.nt_shift_reg_lo.set(lo | lo_shift);
    }

    fn sprite_evaluation<'a>(nes: &'a Nes) -> impl Coroutine<Yield = RenderStep, Return = !> + 'a {
        #[coroutine] move || loop {
            let mut oam_scan_index = 0;
            let mut oam_buf_index = 0;
            let mut oam_buf = [SpriteInfo::default(); 8];
            let mut sprite_output_units_buf = [SpriteOutputUnit::default(); 8];
            for frame in 0.. {
                for scanline in 0..FRAME_H {
                    let y = scanline as u32;
                    let oam = nes.ppu.oam();
                    nes.ppu.status.update(|s| s & !PpuStatus::OVERFLOW);

                    if y < 240 {
                        for x in 0..FRAME_W as u32 {
                            if x == 64 {
                                // nes.ppu.secondary_oam.set([0xFF; 32]);
                                oam_scan_index = 0;
                                oam_buf = [SpriteInfo::default(); 8];
                                oam_buf_index = 0;
                            } else if Ppu::sprite_evaluate_timing(x, 0) {
                                let sprite_y_m_1 = oam[oam_scan_index * 4].get() as u32;

                                if sprite_y_m_1 <= y && y < sprite_y_m_1 + 8 {
                                    if oam_buf_index < 8 {
                                        let y = oam[oam_scan_index * 4 + 0].get();
                                        let id = oam[oam_scan_index * 4 + 1].get();
                                        let attr = oam[oam_scan_index * 4 + 2].get();
                                        let x = oam[oam_scan_index * 4 + 3].get();
                                        // println!("{}", id); //162

                                        let sprite_info = SpriteInfo { y, id, attr, x };
                                        oam_buf[oam_buf_index] = sprite_info;
                                        oam_buf_index = oam_buf_index + 1;
                                    } else {
                                        nes.ppu.status.update(|s| s | PpuStatus::OVERFLOW);
                                    }
                                }

                                if oam_scan_index < 63 {
                                    oam_scan_index = oam_scan_index + 1;
                                }
                            } else if x == 257 {
                                nes.ppu.secondary_oam.set(oam_buf);
                            } else if Ppu::sprite_loading_timing(x, 3) {
                                let index = (x as usize - 257) / 8;

                                let sprite_info = nes.ppu.secondary_oam()[index].get();
                                let sprite_pattern_addr_lo =
                                    Ppu::sprite_loading_addr(nes, y, &sprite_info);
                                let sprite_lsbits = nes.ppu.read8(sprite_pattern_addr_lo);
                                let sprite_msbits =
                                    nes.ppu.read8(sprite_pattern_addr_lo.wrapping_add(8));
                                let x = sprite_info.x;
                                let (sprite_lsbits, sprite_msbits) = Ppu::sprite_flipbyte(
                                    &sprite_info,
                                    sprite_lsbits,
                                    sprite_msbits,
                                );

                                let sprite_output = SpriteOutputUnit {
                                    sprite_lsbits,
                                    sprite_msbits,
                                    x,
                                    sprite_info,
                                };

                                sprite_output_units_buf[index] = sprite_output;
                            } else if x == 340 {
                                nes.ppu.sprite_output_units.set(sprite_output_units_buf);
                            }
                            yield RenderStep::Cycle(0, 0, 0);
                        }
                    } else {
                        for x in 0..FRAME_W {
                            yield RenderStep::Cycle(0, 0, 0);
                        }
                    }
                }
            }
        }
    }

    pub fn sprite_evaluate_timing(ev_x: u32, offset: u32) -> bool {
        65 <= ev_x && ev_x <= 256 && (ev_x - 65) % 2 == offset
    }

    pub fn sprite_loading_timing(ev_x: u32, offset: u32) -> bool {
        257 <= ev_x && ev_x <= 320 && (ev_x - 257) % 8 == offset
    }

    pub fn sprite_loading_addr(nes: &Nes, y: u32, sprite_info: &SpriteInfo) -> u16 {
        //sprite 8x8 mode
        let sprite_offset_addr = (nes
            .ppu
            .ctrl
            .get()
            .contains(PpuCtrl::SPRITE_PATTERN_TABLE_ADDR) as u16)
            << 12;

        let sprite_patten_dy = if sprite_info.attr & 0x80 == 0x00 {
            y.wrapping_sub(sprite_info.y as u32) as u16
        } else {
            //flip vertically
            7u16.wrapping_sub(y.wrapping_sub(sprite_info.y as u32) as u16)
        };
        let sprite_pattern_id = (sprite_info.id as u16) << 4;

        let sprite_pattern_addr_lo = sprite_offset_addr | sprite_pattern_id | sprite_patten_dy;
        sprite_pattern_addr_lo
    }

    pub fn sprite_flipbyte(sprite_info: &SpriteInfo, lo_byte: u8, hi_byte: u8) -> (u8, u8) {
        let flipbyte = |b| {
            let b = (b & 0xF0 as u8) >> 4 | (b & 0x0F as u8) << 4;
            let b = (b & 0xCC as u8) >> 2 | (b & 0x33 as u8) << 2;
            let b = (b & 0xAA as u8) >> 1 | (b & 0x55 as u8) << 1;
            b
        };

        let (new_lo, new_hi) = if sprite_info.attr & 0x40 == 0x00 {
            (lo_byte, hi_byte)
        } else {
            (flipbyte(lo_byte), flipbyte(hi_byte))
        };

        (new_lo, new_hi)
    }

    pub fn sprite_output_units_update<'a>(
        nes: &'a Nes,
    ) -> impl Coroutine<Yield = RenderStep, Return = !> + 'a {
        #[coroutine] move || loop {
            let mut sprite_output_units = nes.ppu.sprite_output_units.get();

            for unit in sprite_output_units.iter_mut() {
                if unit.x > 0 {
                    unit.x = unit.x - 1;
                } else {
                    unit.sprite_lsbits = unit.sprite_lsbits << 1;
                    unit.sprite_msbits = unit.sprite_msbits << 1;
                }
            }

            nes.ppu.sprite_output_units.set(sprite_output_units);
            yield RenderStep::Cycle(0, 0, 0);
        }
    }

    pub fn run<'a>(nes: &'a Nes) -> impl Coroutine<Yield = PpuStep, Return = !> + 'a {
        // let mut renderer = Ppu::renderer(nes);
        let mut renderer = Ppu::renderer(nes);
        let mut nt_evaluation = Ppu::nt_evaluation(nes);
        let mut sprite_evaluation = Ppu::sprite_evaluation(nes);
        let mut sprite_output_unit_update = Ppu::sprite_output_units_update(nes);
        let mut renderstep = RenderStep::Cycle(0, 0, 0);

        #[coroutine] move || loop {
            loop {
                match Pin::new(&mut renderer).resume(()) {
                    CoroutineState::Yielded(step @ RenderStep::Cycle(_, _, _)) => {
                        renderstep = step;
                        // println!("render");
                        break;
                    }
                    CoroutineState::Yielded(RenderStep::Vblank) => {
                        yield PpuStep::Vblank;
                    }
                }
            }

            //@always cycle
            nes.ppu.nt_shift_reg_hi.update(|value| value << 1);
            nes.ppu.nt_shift_reg_lo.update(|value| value << 1);

            nes.ppu.at_shift_reg_hi.update(|value| value << 1);
            nes.ppu.at_shift_reg_lo.update(|value| value << 1);

            //@always cycle
            loop {
                match Pin::new(&mut sprite_output_unit_update).resume(()) {
                    CoroutineState::Yielded(RenderStep::Cycle(_, _, _)) => {
                        break;
                    }
                    _ => {}
                }
            }

            loop {
                match Pin::new(&mut nt_evaluation).resume(()) {
                    CoroutineState::Yielded(RenderStep::Cycle(_, _, _)) => {
                        // println!("eval");
                        break;
                    }
                    _ => {}
                }
            }

            loop {
                match Pin::new(&mut sprite_evaluation).resume(()) {
                    CoroutineState::Yielded(RenderStep::Cycle(_, _, _)) => {
                        // println!("eval");
                        break;
                    }
                    _ => {}
                }
            }

            yield PpuStep::Cycle(renderstep, 0);
        }
    }

    pub fn power_up(chrrom: [u8; 0x2000], mirror: MIRROR) -> Self {
        let img = image::RgbaImage::new(VFRAME_W, VFRAME_H);

        Ppu {
            chrrom: chrrom,
            vram: cell_zero(),
            oam: Cell::new([0xEF; 0x0100]),
            palette: cell_zero(),
            addr: Cell::default(),
            ctrl: Cell::default(),
            mask: Cell::default(),
            status: Cell::default(),
            oamaddr: Cell::default(),
            scroll: Cell::default(),
            latch: Cell::default(),
            img: RefCell::new(img),
            data_buf2006: Cell::default(),
            secondary_oam: Cell::default(),
            sprite_output_units: Cell::default(),
            nt_shift_reg_hi: Cell::default(),
            nt_shift_reg_lo: Cell::default(),
            at_shift_reg_hi: Cell::default(),
            at_shift_reg_lo: Cell::default(),
            mirror: Cell::new(mirror),
            grid_on: Cell::default(),
            debug_num: Cell::default(),
        }
    }

    pub fn read_reg(&self, reg: u16) -> u8 {
        match reg {
            2 => {
                let ret = self.status.get().bits();
                // Reading $2002 clears VBlank flag and resets address latch
                self.status.update(|s| s & !PpuStatus::VBLANK_STARTED);
                self.latch.set(false);
                ret
            }
            4 => 0xFF,
            7 => {
                let addr = self.addr.get();
                let buffered_data = self.read8(addr);

                let ret = if addr > 0x3F00 {
                    buffered_data
                } else {
                    self.data_buf2006.get()
                };

                self.data_buf2006.set(buffered_data);
                let step = match self.ctrl.get().contains(PpuCtrl::VRAM_ADDR_INCREMENT) {
                    true => 32,
                    false => 1,
                };

                self.addr.set(addr.wrapping_add(step));

                ret
            }
            _ => {
                unreachable!("PPU reg {} is read Only!", reg)
            }
        }
    }

    pub fn write_reg(&self, reg: u16, data: u8) {
        match reg {
            0 => {
                self.ctrl.set(PpuCtrl::from_bits_truncate(data));
            }
            1 => {
                self.mask.set(PpuMask::from_bits_truncate(data));
            }
            2 => {
                //Status is readonly
                unreachable!("Status is Read Only");
            }
            3 => {
                self.oamaddr.set(data);
            }
            4 => {
                let oamaddr = self.oamaddr.get();
                let oam = self.oam();
                oam[oamaddr as usize].set(data);
                let next_oamaddr = (oamaddr.wrapping_add(1) as usize % oam.len()) as u8;
                self.oamaddr.set(next_oamaddr);
            }
            5 => {
                let latch = self.latch.get();
                let newscroll = if latch {
                    //to high
                    let scroll_hi = (data as u16) << 8;
                    let scroll_lo = self.scroll.get() & 0x00FF;
                    scroll_hi | scroll_lo
                } else {
                    //to low
                    let scroll_lo = data as u16;
                    let scroll_hi = self.scroll.get() & 0xFF00;
                    scroll_hi | scroll_lo
                };

                self.scroll.set(newscroll);
                self.latch.set(!latch);
            }
            6 => {
                let latch = self.latch.get();

                let newaddr = if latch {
                    //to low
                    let addr_lo = data as u16;
                    let addr_hi = self.addr.get() & 0xFF00;
                    addr_hi | addr_lo
                } else {
                    //to high
                    let addr_hi = (data as u16) << 8;
                    let addr_lo = self.addr.get() & 0x00FF;
                    addr_hi | addr_lo
                };
                self.addr.set(newaddr);
                self.latch.set(!latch);
            }
            7 => {
                let addr = self.addr.get();
                let ctrl = self.ctrl.get();
                let step = match ctrl.contains(PpuCtrl::VRAM_ADDR_INCREMENT) {
                    true => 32,
                    false => 1,
                };

                self.write8(addr, data);
                self.addr.set(addr.wrapping_add(step));
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                //pattern table 0 and 1
                // self.chrrom()[addr as usize].get()
                self.chrrom[addr as usize]
            }
            0x2000..=0x3EFF => {
                let addr = addr - 0x2000;
                //0x0000 ~ 0x1EFF
                let data = match self.mirror.get() {
                    MIRROR::VERTICAL => match addr {
                        0x0000..=0x03FF => self.vram()[addr as usize].get(),
                        0x0400..=0x07FF => self.vram()[addr as usize].get(),
                        0x0800..=0x0BFF => self.vram()[(addr - 0x0800) as usize].get(),
                        0x0C00..=0x0FFF => self.vram()[(addr - 0x0800) as usize].get(),
                        _ => 0,
                    },
                    MIRROR::HORIZONTAL => match addr {
                        0x0000..=0x03FF => self.vram()[addr as usize].get(),
                        0x0400..=0x07FF => self.vram()[(addr - 0x0400) as usize].get(),
                        0x0800..=0x0BFF => self.vram()[(addr - 0x0400) as usize].get(),
                        0x0C00..=0x0FFF => self.vram()[(addr - 0x0800) as usize].get(),
                        _ => 1,
                    },
                    _ => 2,
                };
                data
            }
            // 0x2000..=0x2FFF => {
            //     //name table 0 ~ attribute table 0
            //     //name table 1 ~ attribute table 1
            //     //name table 2 ~ attribute table 2
            //     //name table 3 ~ attribute table 3
            //     let offset = addr - 0x2000;
            //     self.vram()[offset as usize].get()
            // }
            // 0x3000..=0x3EFF => {
            //     //mirror to 0x2000..=0x2EFF
            //     let offset = (addr - 0x3000) % (0x2EFF - 0x2000);
            //     self.vram()[offset as usize].get()
            // }
            // 0x3F10 => self.read8(0x3F00),
            // 0x3F14 => self.read8(0x3F04),
            // 0x3F18 => self.read8(0x3F08),
            // 0x3F1C => self.read8(0x3F0C),
            0x3F00..=0x3F1F => {
                //BG palette : =0x3F0F
                //SP palette : =0x3F1F
                let offset = addr - 0x3F00;
                self.palette()[offset as usize].get()
            }
            0x3F20..=0x3FFF => {
                //mirror to 0x3F00..=0x3F1F
                let offset = (addr - 0x3F20) % 0x20;
                self.palette()[offset as usize].get()
            }
            _ => {
                // panic!("Out of VMemory 0x{:04X}", addr);
                0
            }
        }
    }

    pub fn write8(&self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                //pattern table 0 and 1
                //maybe readonly
                unreachable!("Chrrom is Read Only @0x{:04X}  0x{:04X}", addr, data);
            }
            0x2000..=0x3EFF => {
                let addr = addr - 0x2000;
                //0x0000 ~ 0x1EFF
                match self.mirror.get() {
                    MIRROR::VERTICAL => match addr {
                        0x0000..=0x03FF => self.vram()[addr as usize].set(data),
                        0x0400..=0x07FF => self.vram()[addr as usize].set(data),
                        0x0800..=0x0BFF => self.vram()[(addr - 0x0800) as usize].set(data),
                        0x0C00..=0x0FFF => self.vram()[(addr - 0x0800) as usize].set(data),
                        _ => unreachable!(),
                    },
                    MIRROR::HORIZONTAL => match addr {
                        0x0000..=0x03FF => self.vram()[addr as usize].set(data),
                        0x0400..=0x07FF => self.vram()[(addr - 0x0400) as usize].set(data),
                        0x0800..=0x0BFF => self.vram()[(addr - 0x0400) as usize].set(data),
                        0x0C00..=0x0FFF => self.vram()[(addr - 0x0800) as usize].set(data),
                        _ => unreachable!(),
                    },
                    _ => unimplemented!(),
                };
            }
            // 0x2000..=0x2FFF => {
            //     //name table 0 ~ attribute table 0
            //     //name table 1 ~ attribute table 1
            //     //name table 2 ~ attribute table 2
            //     //name table 3 ~ attribute table 3
            //     //(0x3C0 + 0x040) * 4 = 0x1000
            //     let offset = addr - 0x2000;
            //     self.vram()[offset as usize].set(data);
            // }
            // 0x3000..=0x3EFF => {
            //     //mirror to 0x2000..=0x2EFF
            //     let offset = addr - 0x3000;
            //     self.vram()[offset as usize].set(data);
            // }
            0x3F00..=0x3F1F => {
                //BG palette : =0x3F0F
                //SP palette : =0x3F1F
                let addr = match addr {
                    0x3F10 | 0x3F14 | 0x3F19 | 0x3F1C => addr - 0x0010,
                    _ => addr,
                };

                let offset = addr - 0x3F00;
                self.palette()[offset as usize].set(data);
            }
            0x3F20..=0x3FFF => {
                //mirror to 0x3F00..=0x3F1F
                let offset = (addr - 0x3F20) & 0x20;
                self.palette()[offset as usize].set(data);
            }
            0x4000..=0xFFFF => {
                // panic!("Out of VMemory 0x{:04X}", addr);
                unimplemented!("Out of PPU Memory address ${:04X}", addr);
            }
        }
    }

    pub fn vram(&self) -> &[Cell<u8>] {
        let ram: &Cell<[u8]> = &self.vram;
        ram.as_slice_of_cells()
    }

    fn oam(&self) -> &[Cell<u8>] {
        let oam: &Cell<[u8]> = &self.oam;
        oam.as_slice_of_cells()
    }

    fn secondary_oam(&self) -> &[Cell<SpriteInfo>] {
        let secondary_oam: &Cell<[SpriteInfo]> = &self.secondary_oam;
        secondary_oam.as_slice_of_cells()
    }

    pub fn palette(&self) -> &[Cell<u8>] {
        let ram: &Cell<[u8]> = &self.palette;
        ram.as_slice_of_cells()
    }
}

fn cell_zero<const N: usize>() -> Cell<[u8; N]> {
    Cell::new([0; N])
}

#[test]
fn test_calc_ld_nt_coord() {
    assert_eq!(Ppu::calc_ld_nt_coord(1, 0), (17, 0));
    assert_eq!(Ppu::calc_ld_nt_coord(321, 0), (1, 1));
    assert_eq!(Ppu::calc_ld_nt_coord(329, 0), (9, 1));
    assert_eq!(Ppu::calc_ld_nt_coord(321, 261), (1, 0));
    assert_eq!(Ppu::calc_ld_nt_coord(329, 261), (9, 0));
}

#[test]
fn test_reg_shift() {
    let rom_dir = "../nes-roms/nestest.nes".to_string();
    let nes = Nes::start(&rom_dir);
    nes.ppu.nt_shift_reg_hi.set(0x0001);
    nes.ppu.nt_shift_reg_lo.set(0x0001);
    let mut nes_run = nes.run();
    println!("{:?}", Pin::new(&mut nes_run).resume(()));
    println!("{:?}", Pin::new(&mut nes_run).resume(()));
    println!("{:?}", Pin::new(&mut nes_run).resume(()));
    println!("{:?}", Pin::new(&mut nes_run).resume(()));

    println!("{:?}", nes.ppu.nt_shift_reg_hi.get());
    println!("{:?}", nes.ppu.nt_shift_reg_lo.get());
}
