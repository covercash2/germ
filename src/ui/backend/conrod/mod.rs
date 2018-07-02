use conrod;
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::glutin;
use conrod::backend::glium::glium::glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use conrod::backend::glium::glium::texture::Texture2d;
use conrod::glium::Surface;
use conrod::text::Font;
use conrod::{color, image, widget, Colorable, Positionable, Sizeable, Widget};

use super::{load_font, Config, Ui};
use constants::{DEFAULT_DIMENSIONS, DEFAULT_FONT, DEFAULT_TITLE};

pub struct Conrod {
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    image_map: image::Map<Texture2d>,
    renderer: conrod::backend::glium::Renderer,
    ui: conrod::Ui,
}

widget_ids! {
    struct Ids { canvas, text_input, scrollbar }
}

impl Ui for Conrod {
    fn init(config: Config) -> Result<Self, String> {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(DEFAULT_TITLE) // TODO
            .with_dimensions(DEFAULT_DIMENSIONS[0], DEFAULT_DIMENSIONS[1]);
        let context = glutin::ContextBuilder::new()
            .with_vsync(config.graphics.vsync.unwrap_or(false))
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

        let font_family = config.font.family.unwrap_or(DEFAULT_FONT.to_string());

        println!("font family: {}", font_family);

        let font = load_font(&font_family)
            .map_err(|e| format!("could not load font:\n{}", e))
            .and_then(|bytes| {
                Font::from_bytes(bytes)
                    .map_err(|e| format!("could not parse font from bytes:\n{}", e))
            })?;

        ui.fonts.insert(font);

        for id in ui.fonts.ids() {
            eprintln!("id: {:?}", id);
        }

        return Ok(Conrod {
            display: display,
            events_loop: events_loop,
            image_map: image_map,
            renderer: renderer,
            ui: ui,
        });
    }

    fn show(mut self) -> Result<(), String> {
        let ids = Ids::new(self.ui.widget_id_generator());

        let mut text: String = "placeholder text".to_owned();

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
                    .color(color::DARK_CHARCOAL)
                    .set(ids.canvas, &mut ui_cell);

                for new_text in widget::TextEdit::new(text.as_str())
                    .color(color::WHITE)
                    .padded_w_of(ids.canvas, 20.0)
                    .mid_top_of(ids.canvas)
                    .left_justify()
                    .line_spacing(2.5)
                    .restrict_to_height(false)
                    .set(ids.text_input, &mut ui_cell)
                {
                    text = new_text;
                }

                widget::Scrollbar::y_axis(ids.canvas)
                    .auto_hide(true)
                    .set(ids.scrollbar, &mut ui_cell);
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
