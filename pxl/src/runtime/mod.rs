//! Input and window handling

extern crate cpal;
extern crate gl;
extern crate glutin;

mod common;
mod display;
mod error;
mod shader_cache;
mod speaker;

use runtime::common::*;

static DEFAULT_PIXEL: Pixel = Pixel {
  red: 0.0,
  green: 0.0,
  blue: 0.0,
  alpha: 1.0,
};

pub struct Runtime {
  events: Vec<Event>,
  window_event_loop: glutin::EventsLoop,
  pixels: Vec<Pixel>,
  program: Box<Program>,
  should_quit: bool,
  gl_window: GlWindow,
  current_title: String,
  display: Display,
}

impl Runtime {
  pub fn new(program: Box<Program>) -> Result<Runtime, Error> {
    let window_event_loop = glutin::EventsLoop::new();

    let current_title = program.title().to_string();
    let resolution = program.resolution();
    let synthesizer = program.synthesizer();

    // Initially select dimensions using the requested resolution
    let mut dimensions = LogicalSize::new(resolution.0 as f64, resolution.1 as f64);

    let window = glutin::WindowBuilder::new()
      .with_title(current_title.as_str())
      .with_dimensions(dimensions);

    let context = glutin::ContextBuilder::new()
      .with_double_buffer(Some(true))
      .with_vsync(true);

    let gl_window = GlWindow::new(window, context, &window_event_loop)?;

    let monitor = gl_window.get_current_monitor();

    let mut maximum_dimensions = monitor
      .get_dimensions()
      .to_logical(monitor.get_hidpi_factor());

    // subtract a 100px border on all sides
    maximum_dimensions.width -= 200.0;
    maximum_dimensions.height -= 200.0;

    // calculate a scaling factor to scale the dimensions up to as
    // large as is allowed by maximum_dimensions
    let scale = (maximum_dimensions.width / dimensions.width)
      .min(maximum_dimensions.height / dimensions.height);

    dimensions.width *= scale;
    dimensions.height *= scale;

    gl_window.set_inner_size(dimensions);

    unsafe {
      gl_window.make_current()?;
      gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    let display = Display::new()?;

    if let Some(synthesizer) = synthesizer {
      let speaker = Speaker::new(synthesizer)?;

      thread::spawn(move || {
        speaker.play();
      });
    }

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
        if let glutin::Event::WindowEvent { event, .. } = event {
          match event {
            CloseRequested => should_quit = true,
            Resized(logical_size) => new_size = Some(logical_size),
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
          }
        }
      });

      mem::replace(&mut self.events, events);

      if let Some(new_size) = new_size {
        self
          .gl_window
          .resize(new_size.to_physical(self.gl_window.get_hidpi_factor()));
      }

      self.program.tick(&self.events);

      let resolution = self.program.resolution();

      let pixel_count = resolution.0 * resolution.1;
      if self.pixels.len() != pixel_count {
        self.pixels.resize(pixel_count, DEFAULT_PIXEL);
      }

      self.display.set_shaders(
        self.program.vertex_shader(),
        self.program.fragment_shader(),
        self.program.filter_shaders(),
      )?;

      self.program.render(&mut self.pixels);
      self.should_quit = self.program.should_quit() | should_quit;
      let title = self.program.title();
      if title != self.current_title {
        self.gl_window.set_title(title);
        self.current_title.clear();
        self.current_title.push_str(&title);
      }

      if let Some(inner_size) = self.gl_window.get_inner_size() {
        let PhysicalSize { width, height } =
          inner_size.to_physical(self.gl_window.get_hidpi_factor());
        self
          .display
          .present(&self.pixels, resolution, (width as u32, height as u32));
      }

      self.gl_window.swap_buffers()?;
    }

    Ok(())
  }
}
