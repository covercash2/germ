pub mod text;

use conrod;
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::glutin;
use conrod::backend::glium::glium::glutin::{
    ElementState, Event, EventsLoop, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent,
};
use conrod::backend::glium::glium::texture::Texture2d;
use conrod::glium::Surface;
use conrod::text::Font;
use conrod::{color, image, widget, Borderable, Colorable, UiCell, Widget};

use super::{load_font, Ui};
use constants::{DEFAULT_DIMENSIONS, DEFAULT_TITLE};

use ui;
use ui::TextView;

use self::text::Text;

widget_ids! {
    struct Ids {
        main_canvas,
        input_canvas,
        output_canvas,

        command_input,
        command_output,
        scrollbar
    }
}

pub struct Conrod {
    display: glium::Display,
    events_loop: EventsLoop,
    image_map: image::Map<Texture2d>,
    renderer: conrod::backend::glium::Renderer,
    ui: conrod::Ui,

    ids: Ids,
    input_view: Text,
    output_view: Text,
}

pub trait Update {
    fn update(&mut self, ui_cell: &mut UiCell);
}

impl Conrod {
    // TODO change name to fit semantics
    pub fn new(font_family: String, vsync: bool) -> Result<Self, String> {
        let events_loop = EventsLoop::new();
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

        let ids = Ids::new(ui.widget_id_generator());

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

        let input_view = Text::new(ids.command_input, ids.input_canvas, true);
        let output_view = Text::new(ids.command_output, ids.output_canvas, false);

        return Ok(Conrod {
            display: display,
            events_loop: events_loop,
            ids: ids,
            image_map: image_map,
            renderer: renderer,
            ui: ui,

            input_view: input_view,
            output_view: output_view,
        });
    }

    /// convert glutin event into app level event
    /// returns an event and whether to capture it
    fn process_event(&mut self, event: &Event) -> Option<(ui::Event, bool)> {
        match *event {
            Event::WindowEvent { ref event, .. } => match event {
                // closed or ESC pressed
                WindowEvent::Closed
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => Some((ui::Event::Exit, false)),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            modifiers:
                                ModifiersState {
                                    shift: true,
                                    ctrl: false,
                                    alt: false,
                                    logo: false,
                                },
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Return),
                            ..
                        },
                    ..
                } => Some((ui::Event::Submit(self.input_view.submit()), true)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Ui for Conrod {
    type Events = Vec<ui::Event>;

    fn draw(&mut self) -> Result<(), String> {
        // put ui in a memory cage and draw elements
        {
            let mut ui_cell: conrod::UiCell = self.ui.set_widgets();

            widget::Canvas::new()
                .color(color::BLACK)
                .flow_down(&[
                    (
                        self.ids.input_canvas,
                        widget::Canvas::new()
                            .color(color::BLACK)
                            .length_weight(0.25)
                            .parent(self.ids.main_canvas),
                    ),
                    (
                        self.ids.output_canvas,
                        widget::Canvas::new()
                            .color(color::BLACK)
                            .border(2.0)
                            .border_color(color::WHITE)
                            .scroll_kids_vertically()
                            .parent(self.ids.main_canvas),
                    ),
                ]).set(self.ids.main_canvas, &mut ui_cell);

            widget::Scrollbar::y_axis(self.ids.output_canvas)
                .auto_hide(true)
                .set(self.ids.scrollbar, &mut ui_cell);

            self.input_view.update(&mut ui_cell);
            self.output_view.update(&mut ui_cell);
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

        return Ok(());
    }

    fn events(&mut self) -> Vec<ui::Event> {
        let mut events = Vec::new();
        let mut app_events = Vec::new();

        self.events_loop.poll_events(|event| {
            events.push(event);
        });

        // TODO move idle check somewhere else
        // if events.is_empty() {
        //     self.events_loop.run_forever(|event| {
        //         events.push(event);
        //         glium::glutin::ControlFlow::Break
        //     });
        // }

        for event in events {
            let input_captured = match self.process_event(&event) {
                Some((app_event, capture)) => {
                    app_events.push(app_event);
                    capture
                }
                None => false,
            };
            if !input_captured {
                if let Some(event) =
                    conrod::backend::winit::convert_event(event.clone(), &self.display)
                {
                    self.ui.handle_event(event);
                }
            }
        }

        return app_events;
    }

    fn set_output(&mut self, string: &str) {
        self.output_view.set_text(string);
    }
}
