#![crate_name  = "sdl2_text"]
#![crate_type  = "lib"]

extern crate sdl2;
extern crate sdl2_ttf;

use sdl2::SdlResult;
use sdl2::surface::Surface;

struct Glyph {
    u: u16,
    v: u16,
    advance: u8,
    min_x: i8,
}

pub struct TextRenderer<'a, T> {
    font: sdl2_ttf::Font,
    sdl_renderer: &'a sdl2::render::Renderer<T>,
    glyph_texture: sdl2::render::Texture,
    glyphs: Vec<Glyph>,
    begin_index: uint,
    max_glyph_width: i32,
    max_glyph_height: i32,
}

fn render_glyph_surfaces(font: &sdl2_ttf::Font,
                         begin_code: u32,
                         end_code: u32)
                         -> SdlResult<Vec<Surface>> {
   
    let char_count = (end_code - begin_code) as uint;
    let mut glyph_surfaces: Vec<Surface> = Vec::with_capacity(char_count); 

    for i in range(begin_code, end_code) {

        let character = std::char::from_u32(i).unwrap();
        
        let glyph_surface = try!(font.render_char_blended(character,
                                                          sdl2::pixels::RGB(255, 255, 255)));

        glyph_surfaces.push(glyph_surface);
    }

    Ok(glyph_surfaces)
}

fn find_max_glyph_size(surfaces: &Vec<Surface>) -> (i32, i32) {

    let mut max_glyph_width: i32 = 0;
    let mut max_glyph_height: i32 = 0;

    for s in surfaces.iter() {
        let surface_width = s.get_width() as i32;
        let surface_height = s.get_height() as i32;
        if surface_width > max_glyph_width { max_glyph_width = surface_width; }
        if surface_height > max_glyph_height { max_glyph_height = surface_height; }
    }

    (max_glyph_width, max_glyph_height)
}

fn generate_glyphs(font: &sdl2_ttf::Font, begin_char: u32, end_char: u32, dim: i32,
                   max_glyph_width: i32, max_glyph_height: i32) -> Vec<Glyph> {

    let char_count = (end_char - begin_char) as uint;
    let mut glyphs: Vec<Glyph> = Vec::with_capacity(char_count); 
    let mut index: i32 = 0;
    for code in range(begin_char, end_char) {
         
        let character = std::char::from_u32(code).unwrap();
        let cur_row = index / dim;
        let cur_col = index % dim;
        
        let metrics = match font.metrics_of_char(character) {
            Some(metrics) => metrics,
            None => sdl2_ttf::GlyphMetrics { minx: 0, maxx: 0, miny: 0, maxy: 0,
                                             advance: 0 }
        };

        // UV coordinates of the glyph
        let u = cur_col * max_glyph_width; 
        let v = cur_row * max_glyph_height;

        let glyph_adv = metrics.advance as u8;

        let glyph = Glyph { u: u as u16, v: v as u16, advance: glyph_adv,
                            min_x: metrics.minx as i8 };

        glyphs.push(glyph);
        
        index += 1;
    }

    glyphs
}

impl<'a, T> TextRenderer<'a, T> {

    pub fn from_path(font_path: &Path,
                     font_size: int,
                     sdl_renderer: &'a sdl2::render::Renderer<T>)
                     -> SdlResult<TextRenderer<'a, T>> {


        sdl2_ttf::init();

        let font = try!(sdl2_ttf::Font::from_file(font_path, font_size));
        let char_begin = 0x20u32; // Space
        let char_end = 0xFFu32;
        let glyph_surfaces = try!(render_glyph_surfaces(&font, char_begin, char_end));

        let (max_glyph_width, max_glyph_height) = find_max_glyph_size(&glyph_surfaces);      

        
        let num_glyphs = glyph_surfaces.len() as i32;
        let rows = (num_glyphs as f32).sqrt().ceil() as i32;

        let texture_width = rows * max_glyph_width;
        let texture_height = rows * max_glyph_height;

        let glyph_atlas_surface = try!(sdl2::surface::Surface::new(
                sdl2::surface::SWSurface, texture_width as int,
                texture_height as int, 32, 0xFF000000u32,
                0x00FF0000u32, 0x0000FF00u32, 0x000000FFu32));


        let glyphs = generate_glyphs(&font,
                                     char_begin,
                                     char_end,
                                     rows,
                                     max_glyph_width,
                                     max_glyph_height);

        { 
            let mut iter = glyph_surfaces.iter().zip(glyphs.iter());

            for (s, g) in iter {
                let u = g.u as i32;
                let v = g.v as i32;

                let dst_rect = sdl2::rect::Rect::new(
                    u, v, max_glyph_width, max_glyph_height);

                let _ = glyph_atlas_surface.blit(s, Some(dst_rect), None);

            }
        }

        let glyph_texture_atlas = try!(sdl_renderer.create_texture_from_surface(
            &glyph_atlas_surface));

        let _ = glyph_texture_atlas.set_blend_mode(sdl2::render::BlendBlend);

        let atlas_rect = sdl2::rect::Rect::new(0, 0, texture_width, texture_height);
        let _ = sdl_renderer.copy(&glyph_texture_atlas, None, Some(atlas_rect));

        Ok(TextRenderer { font: font, sdl_renderer: sdl_renderer,
                                  glyph_texture: glyph_texture_atlas,
                                  begin_index: char_begin as uint,
                                  glyphs: glyphs, max_glyph_width: max_glyph_width,
                                  max_glyph_height: max_glyph_height })
    }

    fn blit_glyph(&self, glyph: &Glyph, x: i32, y: i32) {

        let u = glyph.u as i32;
        let v = glyph.v as i32;
        let min_x = glyph.min_x as i32;


        let src_rect = sdl2::rect::Rect::new(u, v, self.max_glyph_width, self.max_glyph_height);

        let dst_x = x + min_x;
        let dst_y = y;

        let dst_rect = sdl2::rect::Rect::new(dst_x, dst_y, 
                                             self.max_glyph_width, self.max_glyph_height);
        
        let _ = self.sdl_renderer.copy(&self.glyph_texture, Some(src_rect), Some(dst_rect));
    }

    pub fn draw<T: Str>(&self, text: &T, x: i32, y: i32) -> (i32, i32) {
       self.draw_str(text.as_slice(), x, y) 
    }

    pub fn draw_str(&self, text: &str, x: i32, y: i32) -> (i32, i32) {

        let mut char_offset_x = x;
        let mut char_offset_y = y;

        for c in text.chars() {

            match c { 
                '\n' => { 
                    char_offset_y += self.get_line_height();
                    char_offset_x = x;
                    continue; },
                _ => ()
            };

            let code = c as uint;
            let mut code_index = code - self.begin_index;

            if code_index >= self.glyphs.len() {
                code_index = 0;
            }

            let glyph = self.glyphs.get(code_index);
            let advance = glyph.advance as i32;

            self.blit_glyph(glyph, char_offset_x, char_offset_y);
         
            char_offset_x += advance;
        }

        (char_offset_x, char_offset_y) 
    }


    pub fn set_color(&mut self, color: sdl2::pixels::Color) {
        
        match color {
            sdl2::pixels::RGB(r, g, b) => { let _ = self.glyph_texture.set_color_mod(r, g, b); },
            sdl2::pixels::RGBA(r, g, b, a) => {
                let _ = self.glyph_texture.set_color_mod(r, g, b);
                let _ = self.glyph_texture.set_alpha_mod(a);
            }
        };
    }

    pub fn get_color(&self) -> SdlResult<sdl2::pixels::Color> {

       let (r, g, b) = try!(self.glyph_texture.get_color_mod());
       let alpha = try!(self.glyph_texture.get_alpha_mod());

       Ok(sdl2::pixels::RGBA(r, g, b, alpha))
    }

    pub fn get_line_height(&self) -> i32 {
        self.font.ascent() as i32
    }

}
