pub mod text;

use conrod;
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::glutin;
use conrod::backend::glium::glium::glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use conrod::backend::glium::glium::texture::Texture2d;
use conrod::glium::Surface;
use conrod::text::Font;
use conrod::{color, image, widget, Colorable, UiCell, Widget};

use super::{load_font, Config, Ui};
use constants::{DEFAULT_DIMENSIONS, DEFAULT_FONT, DEFAULT_TITLE};

use ui::TextView;

use self::text::Text;

widget_ids! {
    struct Ids { canvas, command_input, command_output, scrollbar }
}

pub struct Conrod {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    image_map: image::Map<Texture2d>,
    renderer: conrod::backend::glium::Renderer,
    ui: conrod::Ui,
}

pub trait Update {
    fn update(&mut self, ui_cell: &mut UiCell);
}

impl Conrod {
    pub fn new(font_family: String, vsync: bool) -> Result<Self, String> {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(DEFAULT_TITLE) // TODO
            .with_dimensions(DEFAULT_DIMENSIONS[0], DEFAULT_DIMENSIONS[1]);
        let context = glutin::ContextBuilder::new()
            .with_vsync(vsync)
            .with_multisampling(4); // TODO ??
        let display = match glium::Display::new(window, context, &events_loop) {
            Ok(d) => d,
            Err(e) => return Err(format!("could not create the display: {}", e)),
        };
        let mut ui =
            conrod::UiBuilder::new([DEFAULT_DIMENSIONS[0] as f64, DEFAULT_DIMENSIONS[1] as f64])
                .build();

        let renderer = match conrod::backend::glium::Renderer::new(&display) {
            Ok(r) => r,
            Err(e) => return Err(format!("could not create renderer: {}", e)),
        };

        let image_map = image::Map::<Texture2d>::new();

        let font = load_font(&font_family)
            .map_err(|e| format!("could not load font:\n{}", e))
            .and_then(|bytes| {
                Font::from_bytes(bytes)
                    .map_err(|e| format!("could not parse font from bytes:\n{}", e))
            })?;
        ui.fonts.insert(font);

        return Ok(Conrod {
            display: display,
            events_loop: events_loop,
            image_map: image_map,
            renderer: renderer,
            ui: ui,
        });
    }
}

impl Ui for Conrod {
    fn show(mut self) -> Result<(), String> {
        let ids = Ids::new(self.ui.widget_id_generator());

        let mut text_input = Text::new(ids.command_input, ids.canvas, true);
        let mut text_output = Text::new(ids.command_output, ids.canvas, false);

        text_input.set_text("some text");
        text_output.set_text("output text");

        'main: loop {
            let mut events = Vec::new();

            self.events_loop.poll_events(|event| {
                events.push(event);
            });

            // wait for events
            if events.is_empty() {
                self.events_loop.run_forever(|event| {
                    events.push(event);
                    glium::glutin::ControlFlow::Break
                });
            }

            for event in events {
                if let Some(event) =
                    conrod::backend::winit::convert_event(event.clone(), &self.display)
                {
                    self.ui.handle_event(event);
                }

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Closed
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            {
                let mut ui_cell: conrod::UiCell = self.ui.set_widgets();

                widget::Canvas::new()
                    .scroll_kids_vertically()
                    .color(color::BLACK)
                    .set(ids.canvas, &mut ui_cell);

                widget::Scrollbar::y_axis(ids.canvas)
                    .auto_hide(true)
                    .set(ids.scrollbar, &mut ui_cell);

                text_input.update(&mut ui_cell);
                text_output.update(&mut ui_cell);
            }

            if let Some(primitives) = self.ui.draw_if_changed() {
                self.renderer
                    .fill(&self.display, primitives, &self.image_map);
                let mut target = self.display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                self.renderer
                    .draw(&self.display, &mut target, &self.image_map)
                    .unwrap();
                target.finish().unwrap();
            }
        }

        return Ok(());
    }
}
