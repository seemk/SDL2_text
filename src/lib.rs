#![crate_name  = "sdl2_text"]
#![crate_type  = "lib"]

extern crate sdl2;
extern crate freetype;

use sdl2::SdlResult;
use sdl2::surface::Surface;
use ft as freetype;
use binpack::BinPack;

mod binpack;

// Some metrics in Freetype are expressed in 1/64th of pixels.
static FT_SCALE_SHIFT: uint = 6;

struct Glyph {
    u: u16,
    v: u16,
    width: i32,
    height: i32,
    advance: u8,
    offset_x: i8,
    offset_y: i8,
}

struct Vec2 {
    pub x: i32,
    pub y: i32
}

impl Vec2 {
    
    fn new(x: i32, y: i32) -> Vec2 {
        Vec2 { x: x, y: y }
    }
}

pub struct TextRenderer {
    font: ft::Face,
    glyph_texture: sdl2::render::Texture,
    glyphs: Vec<Glyph>,
    begin_index: uint,
    line_height: i32,
    // ft::Library is reference counted, need to hold to it.
    #[allow(dead_code)]
    freetype: ft::Library,
}

fn render_char(slot: &ft::GlyphSlot) -> sdl2::surface::Surface {
    
    let bitmap = slot.bitmap();
    let width = bitmap.width() as int;
    let height = bitmap.rows() as int;

    let surface = match sdl2::surface::Surface::new(
            sdl2::surface::SWSurface, width, height,
            32, 0x000000FFu32,
            0x0000FF00u32, 0x00FF0000u32, 0xFF000000u32) {
        Ok(s) => s,
        Err(e) => fail!("Failed to create surface {}", e)
    };


    surface.with_lock(|pixels: &mut [u8]| {
        let buffer = bitmap.buffer();
        for pixel_idx in range(0, buffer.len()) {
            let grayscale_pixel = buffer[pixel_idx];
            let surface_idx = pixel_idx * 4;
            pixels[surface_idx] = 255u8;
            pixels[surface_idx + 1] = 255u8;
            pixels[surface_idx + 2] = 255u8;
            pixels[surface_idx + 3] = grayscale_pixel;
        }
    });

    surface
}

fn make_glyph(slot: &ft::GlyphSlot) -> Glyph {

    let width = slot.bitmap().width();
    let height = slot.bitmap().rows();
    let offset_x = slot.bitmap_left();
    let offset_y = slot.bitmap_top();

    let advance = slot.advance();

    let advance_x = advance.x >> FT_SCALE_SHIFT;

    Glyph { u: 0, v: 0, width: width, height: height,
            advance: advance_x as u8,
            offset_x: offset_x as i8,
            offset_y: offset_y as i8 }

}

impl TextRenderer {

    /// Creates a new text renderer from the given TTF font path.
    ///
    /// # Arguments
    ///
    /// * font_path - Path of the TTF font.
    /// * font_size - Requested font size.
    /// 
    /// # Returns
    /// 
    /// Returns a SdlResult (Ok(renderer) or Err(error)).
    pub fn from_path<T>(font_path: &Path,
                     font_size: int,
                     renderer: &sdl2::render::Renderer<T>)
                     -> SdlResult<TextRenderer> {


        let freetype = ft::Library::init().unwrap();

        let font = match freetype.new_face(font_path.as_str().unwrap(), 0) {
            Ok(font) => font,
            Err(e) => return Err(format!("{}", e))
        };

        font.set_pixel_sizes(font_size as u32, 0).unwrap();

        let char_begin = 0x20u32; // Space
        let char_end = 0xFFu32;

        let texture_width = 512i;
        let texture_height = 512i;

        let glyph_atlas_surface = try!(sdl2::surface::Surface::new(
                sdl2::surface::SWSurface, texture_width, texture_height, 32,
                0xFF000000u32, 0x00FF0000u32, 0x0000FF00u32, 0x000000FFu32));

        let mut packer = BinPack::new(texture_width as i32, texture_height as i32);

        let mut glyphs: Vec<Glyph> = Vec::new();

        for c in range(char_begin, char_end) {

            font.load_char(c as u64, ft::face::LoadTargetNormal
                           | ft::face::Render
                           | ft::face::ForceAutohint).unwrap();

            let slot = font.glyph();

            let surface = render_char(slot);

            // Create a new glyph with (u, v) = (0, 0)
            let mut glyph = make_glyph(slot);

            let dst_rect = match packer.insert(glyph.width, glyph.height) {
                Some(rect) => rect,
                None => fail!("Couldn't pack glyph for {}", c)
            };

            glyph.u = dst_rect.x as u16;
            glyph.v = dst_rect.y as u16;


            let sdl_dst_rect = sdl2::rect::Rect::new(dst_rect.x, dst_rect.y,
                                                      dst_rect.width, dst_rect.height);

            let _ = glyph_atlas_surface.blit(&surface, Some(sdl_dst_rect), None);

            glyphs.push(glyph);
        }

        let glyph_texture_atlas = try!(renderer.create_texture_from_surface(
            &glyph_atlas_surface));

        let _ = glyph_texture_atlas.set_blend_mode(sdl2::render::BlendBlend);

        let atlas_rect = sdl2::rect::Rect::new(0, 0, texture_width as i32,
                                               texture_height as i32);
        let _ = renderer.copy(&glyph_texture_atlas, None, Some(atlas_rect));

        Ok(TextRenderer { font: font,
                          glyph_texture: glyph_texture_atlas,
                          begin_index: char_begin as uint,
                          glyphs: glyphs, line_height: font_size as i32,
                          freetype: freetype })
    }

    fn get_glyph(&self, character: char) -> Glyph {

        let index = character as uint - self.begin_index;

        let char_index = if index >= self.glyphs.len() {
            0u
        } else {
            index
        };

        self.glyphs[char_index]
    }


    fn blit_glyph<T>(&self, glyph: &Glyph, x: i32, y: i32,
                     renderer: &sdl2::render::Renderer<T>) {

        let u = glyph.u as i32;
        let v = glyph.v as i32;
        let offset_x = glyph.offset_x as i32;
        let offset_y = glyph.offset_y as i32;


        let src_rect = sdl2::rect::Rect::new(u, v, glyph.width, glyph.height);

        let dst_x = x + offset_x;
        let dst_y = y - offset_y + self.get_line_height();

        let dst_rect = sdl2::rect::Rect::new(dst_x, dst_y, glyph.width, glyph.height);
        
        let _ = renderer.copy(&self.glyph_texture, Some(src_rect), Some(dst_rect));
    }

    fn render_char<T>(&self, character: char, pos: Vec2, initial_pos: Vec2, kerning: i32,
                   renderer: &sdl2::render::Renderer<T>)
        -> Vec2 {
       
        match character {
            '\n' => {
                Vec2::new(initial_pos.x, pos.y + self.get_line_height())
            },
            _ => {
                let glyph = self.get_glyph(character);

                self.blit_glyph(&glyph, pos.x + kerning, pos.y, renderer);
                let advance = glyph.advance as i32;
                Vec2::new(pos.x + advance + kerning, pos.y)
            }
        }
    }

    fn get_kerning(&self, left_char: char, right_char: char) -> i32 {
       
        let prev_char_idx = self.font.get_char_index(left_char as u64);
        let cur_char_idx = self.font.get_char_index(right_char as u64);
       
        match self.font.get_kerning(prev_char_idx, cur_char_idx, ft::face::KerningDefault) {
            Ok(kerning) => {
                (kerning.x >> FT_SCALE_SHIFT) as i32
            },
            _ => 0i32
        }
    }

    /// # Arguments
    ///
    /// * `text` - the text to be drawn. Newlines start from x, y + line_height.
    /// * `x` & `y` - x, y coordinates of the text's top left corner.
    /// * `renderer` - SDL renderer to do the drawing
    ///
    /// # Returns
    ///
    /// A tuple containing the x, y coordinates of the pen after rendering the text.
    pub fn draw<T: Str, R>(&self, text: &T, x: i32, y: i32,
                           renderer: &sdl2::render::Renderer<R>) -> (i32, i32) {
       self.draw_str(text.as_slice(), x, y, renderer) 
    }

    pub fn draw_str<T>(&self, text: &str, x: i32, y: i32,
                       renderer: &sdl2::render::Renderer<T>) -> (i32, i32) {

        if text.len() == 0 {
            return (x, y);
        }

        let mut pen_pos = Vec2::new(x, y);
        let initial_pos = pen_pos;

        let mut prev_range = text.char_range_at(0);
        pen_pos = self.render_char(prev_range.ch, pen_pos, initial_pos, 0, renderer);

        let mut i = prev_range.next;
        while i < text.len() {


            let cur_range = text.char_range_at(i);
            let cur_char = cur_range.ch;
            let prev_char = prev_range.ch;

            let kerning = self.get_kerning(prev_char, cur_char);

            pen_pos = self.render_char(cur_char, pen_pos, initial_pos, kerning, renderer);
            i = cur_range.next;
            prev_range = cur_range;
                                                              
        }

        (pen_pos.x, pen_pos.y)
    }

    pub fn set_color(&mut self, color: sdl2::pixels::Color) {
        
        match color {
            sdl2::pixels::RGB(r, g, b) => { 
                let _ = self.glyph_texture.set_color_mod(r, g, b);
                let _ = self.glyph_texture.set_alpha_mod(255);
            },
            sdl2::pixels::RGBA(r, g, b, a) => {
                let _ = self.glyph_texture.set_color_mod(r, g, b);
                let _ = self.glyph_texture.set_alpha_mod(a);
            }
        };
    }

    pub fn get_color(&self) -> sdl2::pixels::Color {

       let (r, g, b) = match self.glyph_texture.get_color_mod() {
            Ok(color) => color,
            Err(_) => (0, 0, 0)
       };

       let alpha = match self.glyph_texture.get_alpha_mod() {
           Ok(alpha) => alpha,
           Err(_) => 0
       };

       sdl2::pixels::RGBA(r, g, b, alpha)
    }

    /// Returns the line height in pixels.
    pub fn get_line_height(&self) -> i32 {
        self.line_height
    }

    /// Returns a reference to the texture containing the character bitmaps.
    pub fn get_atlas_texture(&self) -> &sdl2::render::Texture {
        &self.glyph_texture
    }
}
