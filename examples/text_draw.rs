#![feature(globs)]
extern crate sdl2;
extern crate sdl2_text;

use sdl2::keycode::*;
use std::os;

fn main() {

    let args = os::args();
    let font_path = match args.len() {
        0..1 => {
            let mut path = os::self_exe_path().unwrap();
            path.push("assets/bitstream.ttf");
            path
        }
        _ => Path::new(args.get(1).as_slice())  
    };

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

    let font_size = 18;
    let mut text_renderer = match sdl2_text::TextRenderer::from_path(&font_path,
                                                    font_size,
                                                    &renderer) {
        Ok(renderer) => renderer,
        Err(err) => fail!("Failed creating text renderer {}", err)
    };

    text_renderer.set_color(sdl2::pixels::RGB(100, 149, 237));


    let mut text = String::from_str("Type type");

    let _ = renderer.fill_rect(&sdl2::rect::Rect::new(250, 400, 5, 5));
    sdl2::keyboard::start_text_input();


    let mut frame = 1u;
    'main : loop {
        'event : loop {
            match sdl2::event::poll_event() {
                sdl2::event::QuitEvent(_) => break 'main,
                sdl2::event::KeyDownEvent(_, _, key, _, _) => {
                    match key {
                        BackspaceKey => { let _ = text.pop_char(); },
                        ReturnKey | KpEnterKey => { text.push_char('\n'); },
                        _ => ()
                    }
                },
                sdl2::event::TextInputEvent(_, _, s) => {
                    let slice = s.as_slice(); 
                    text = text.append(slice);    
                },
                sdl2::event::NoEvent      => break 'event,
                _ => ()
            };
        }

        let _ = renderer.set_draw_color(sdl2::pixels::RGB(0, 0, 0));
        let _ = renderer.clear();

        let (cur_x, cur_y) = text_renderer.draw(&text, 0, 0);
     
        let line_height = text_renderer.get_line_height();
        let _ = renderer.set_draw_color(sdl2::pixels::RGB(255, 255, 255));
        let _ = renderer.fill_rect(&sdl2::rect::Rect::new(cur_x + 1, cur_y, 1, line_height));

        text_renderer.draw(&format!("frame={}", frame), 0, 460);
        text_renderer.draw_str("This is text?", 400, 400);
        renderer.present();

        frame += 1;
    }

    sdl2::keyboard::stop_text_input();
    sdl2::quit();
}
