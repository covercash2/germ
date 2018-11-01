use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use gio::prelude::*;
use gtk::prelude::*;

use gdk;
use gio;
use glib;
use gtk;

use ui;

use shell::Shell;
use ui::Ui;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

thread_local! (
    static OUTPUT_BUFFER: RefCell<Option<(gtk::TextBuffer, Receiver<String>)>> = RefCell::new(None)
);

type Key = u32;

const APP_ID: &str = "biz.covercash.germ";
const KEY_ENTER: Key = 65293;

pub struct Gtk {
    app: gtk::Application,
}

impl Gtk {
    pub fn create(title: String, dimensions: [i32; 2]) -> Result<Gtk, glib::BoolError> {
        let app = gtk::Application::new(APP_ID, gio::ApplicationFlags::empty())?;
        return Ok(Gtk { app: app });
    }
}

impl Ui for Gtk {
    type Error = String;
    fn show(&mut self, mut shell: Shell) -> Result<(), Self::Error> {
        let shell = Rc::new(RefCell::new(shell));

        self.app.connect_startup(move |app| {
            let builder = gtk::Builder::new_from_string(include_str!("main_window.glade"));

            let main_window: gtk::ApplicationWindow = builder
                .get_object("main_window")
                .expect("could not get main window");

            main_window.set_application(app);
            main_window.connect_delete_event(|win, _| {
                win.destroy();
                Inhibit(false)
            });

            main_window.show_all();

            let input_view: gtk::TextView = builder
                .get_object("input_view")
                .expect("could not get input view from builder");

            input_view.connect_key_press_event(clone!(shell => move |view, key| {
                match process_key_event(view, key) {
                    Some(ui::Event::Submit(string)) => {
                        eprintln!("submitted: {}", string);
                        shell.borrow_mut().execute(&string).expect("shell could not execute command");
                        // TODO
                        // figure out how to keep the enter key from making a new line
                        view.get_buffer().unwrap().set_text("");
                    }
                    _ => (),
                }

                Inhibit(false)
            }));
        });

        self.app.connect_activate(|_| {});

        self.app.run(&::std::env::args().collect::<Vec<_>>());
        return Ok(());
    }
}

fn process_key_event(text_view: &gtk::TextView, key_event: &gdk::EventKey) -> Option<ui::Event> {
    // shift modified
    if key_event
        .get_state()
        .contains(gdk::ModifierType::SHIFT_MASK)
    {
        // enter pressed
        match key_event.get_keyval() {
            KEY_ENTER => {
                return Some(ui::Event::Submit(
                    text_view
                        .get_buffer()
                        .map(|buffer| {
                            let (start, end) = buffer.get_bounds();
                            buffer.get_text(&start, &end, true).unwrap_or("".into())
                        })
                        .unwrap_or("".into()),
                ))
            }
            _ => (),
        }
    }

    return None;
}
