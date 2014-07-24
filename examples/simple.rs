#![feature(globs)]
extern crate sdl2;
extern crate sdl2_text;

use std::os;

fn main() {

    sdl2::init(sdl2::InitVideo);

    let window = match sdl2::video::Window::new("Text example",
                                                sdl2::video::PosCentered,
                                                sdl2::video::PosCentered,
                                                640, 480,
                                                sdl2::video::OpenGL) {
        Ok(win) => win,
        Err(err) => fail!("Error creating window: {}", err)

    };

    let renderer = match sdl2::render::Renderer::from_window(
        window,
        sdl2::render::DriverAuto,
        sdl2::render::Accelerated) {
        
        Ok(renderer) => renderer,
        Err(err) => fail!("Error creating renderer: {}", err)
    };

    let mut font_path = os::self_exe_path().unwrap();
    font_path.push("assets/arial.ttf");
    let font_size = 18;
    let text_renderer = match sdl2_text::TextRenderer::from_path(&font_path,
                                                    font_size,
                                                    &renderer) {
        Ok(renderer) => renderer,
        Err(err) => fail!("Failed creating text renderer {}", err)
    };

    let _ = renderer.set_draw_color(sdl2::pixels::RGB(0, 0, 0));
    let _ = renderer.clear();
    let text = String::from_str("Hello, world!");


    text_renderer.draw(&text, 200, 200);
    renderer.present();
    
    'main : loop {
        'event : loop {
            match sdl2::event::poll_event() {
                sdl2::event::QuitEvent(_) => break 'main,
                sdl2::event::NoEvent      => break 'event,
                _ => ()
            };
        }

    }

    sdl2::quit();
}
