use super::cparams::{VFRAME_H, VFRAME_W};
use super::nes::Nes;
use gfx_device_gl::{CommandBuffer, Resources};
use image::RgbaImage;
use piston_window::{
    clear, image as pw_image, AdvancedWindow, Event, G2dTexture, GfxFactory, PistonWindow, Texture,
    TextureContext, TextureSettings, Transformed, WindowSettings,
};

const SCALE: u32 = 3;
type TextureContextFRC = TextureContext<GfxFactory, Resources, CommandBuffer>;

pub struct NESView {
    pub window: PistonWindow,
    pub texture_context: TextureContextFRC,
    pub texture: G2dTexture,
}

impl NESView {
    pub fn new(nes: &Nes) -> Self {
        let mut window: PistonWindow =
            WindowSettings::new("nes-emu", [VFRAME_W * SCALE, VFRAME_H * SCALE])
                .exit_on_esc(true)
                // .graphics_api(OpenGL::V3_2)
                .samples(0)
                .vsync(true)
                .resizable(false)
                .build()
                .unwrap_or_else(|e| panic!("Failed to build Window: {}", e));

        let mut texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        };

        let texture: G2dTexture = Texture::from_image(
            &mut texture_context,
            &nes.ppu.img.borrow(),
            &TextureSettings::new(),
        )
        .unwrap();

        NESView {
            window,
            texture_context,
            texture,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.window.set_title(title);
    }

    pub fn update(&mut self, img: &RgbaImage) {
        self.texture.update(&mut self.texture_context, img).unwrap();
    }

    pub fn draw_2d(&mut self, event: &Event) {
        self.window.draw_2d(event, |context, graphics, device| {
            self.texture_context.encoder.flush(device);
            clear([0.0; 4], graphics);
            pw_image(
                &self.texture,
                context.transform.scale(SCALE as f64, SCALE as f64),
                graphics,
            )
        });
    }
}

// window.events.set_max_fps(60);
// window.events.set_ups(60);
