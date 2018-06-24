extern crate cpal;
extern crate gl;
extern crate glutin;

mod display;
mod error;
mod speaker;

use std::{ffi::CString,
          mem,
          ptr,
          str,
          sync::{Arc, Mutex},
          thread};

use super::*;

use self::{display::Display,
           error::Error,
           glutin::{GlContext, GlWindow},
           speaker::Speaker};

static DEFAULT_PIXEL: Pixel = Pixel {
  red:   0.0,
  green: 0.0,
  blue:  0.0,
  alpha: 1.0,
};

pub struct Runtime {
  events: Vec<Event>,
  window_event_loop: glutin::EventsLoop,
  pixels: Vec<Pixel>,
  program: Arc<Mutex<Program>>,
  should_quit: bool,
  gl_window: GlWindow,
  current_title: String,
  display: Display,
}

impl Runtime {
  pub fn new(program: Arc<Mutex<Program + Send>>) -> Result<Runtime, Error> {
    let window_event_loop = glutin::EventsLoop::new();

    let current_title;
    let dimensions;
    {
      let program = program.lock().unwrap();
      current_title = program.title().to_string();
      dimensions = program.dimensions();
    }

    let window = glutin::WindowBuilder::new()
      .with_title(current_title.as_str())
      .with_dimensions(dimensions.0 as u32, dimensions.1 as u32);

    let context = glutin::ContextBuilder::new().with_vsync(true);

    let gl_window = GlWindow::new(window, context, &window_event_loop)?;

    unsafe {
      gl_window.make_current()?;
      gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    let display = Display::new()?;

    let speaker = Speaker::new(program.clone())?;

    thread::spawn(move || {
      speaker.play();
    });

    Ok(Runtime {
      should_quit: false,
      events: Vec::new(),
      pixels: Vec::new(),
      program,
      window_event_loop,
      gl_window,
      current_title,
      display,
    })
  }

  pub fn run(mut self) -> Result<(), Error> {
    while !self.should_quit {
      let mut new_size = None;
      let mut should_quit = false;
      let mut events = mem::replace(&mut self.events, Vec::new());

      events.clear();

      self.window_event_loop.poll_events(|event| {
        use self::glutin::WindowEvent::*;
        match event {
          glutin::Event::WindowEvent { event, .. } => match event {
            CloseRequested => should_quit = true,
            Resized(w, h) => new_size = Some((w, h)),
            KeyboardInput { input, .. } => if let Some(virtual_keycode) = input.virtual_keycode {
              use self::glutin::VirtualKeyCode::*;
              let button = match virtual_keycode {
                Up => Button::Up,
                Down => Button::Down,
                Left => Button::Left,
                Right => Button::Right,
                Space => Button::Action,
                _ => return,
              };

              use self::glutin::ElementState::*;
              let state = match input.state {
                Pressed => ButtonState::Pressed,
                Released => ButtonState::Released,
              };
              events.push(Event::Button { state, button });
            },
            ReceivedCharacter(character) => events.push(Event::Key { character }),
            _ => (),
          },
          _ => (),
        }
      });

      mem::replace(&mut self.events, events);

      if let Some((w, h)) = new_size {
        self.gl_window.resize(w, h);
      }

      // We try to avoid doing any work while holding the lock to give the
      // audio callback a chance to obtain the lock.

      self.program.lock().unwrap().tick(&self.events);

      let dimensions = self.program.lock().unwrap().dimensions();

      let pixel_count = dimensions.0 * dimensions.1;
      if self.pixels.len() != pixel_count {
        self.pixels.resize(pixel_count, DEFAULT_PIXEL);
      }

      {
        let program = self.program.lock().unwrap();
        self.display.set_shaders(program.vertex_shader(), program.fragment_shader())?;
      }

      {
        let mut program = self.program.lock().unwrap();
        program.render(&mut self.pixels);
        self.should_quit = program.should_quit() | should_quit;
        let title = program.title();
        if title != self.current_title {
          self.gl_window.set_title(title);
          self.current_title.clear();
          self.current_title.push_str(&title);
        }
      }

      self.display.present(&self.pixels, dimensions);

      self.gl_window.swap_buffers()?;
    }

    Ok(())
  }
}
