use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

use futures;

use gio::prelude::*;
use gtk::prelude::*;

use gdk;
use gio;
use glib;
use gtk;
use gtk::TextBuffer;

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

#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

thread_local! (
    static GLOBAL_CONTEXT: RefCell<Option<Context>> = RefCell::new(None)
);

type Key = u32;

const APP_ID: &str = "biz.covercash.germ";
const KEY_ENTER: Key = 65293;

struct Context {
    stdin_buffer: TextBuffer,
    stdout_buffer: TextBuffer,
    shell: Shell,
}

impl Context {
    fn create_global_context(stdin_buffer: TextBuffer, stdout_buffer: TextBuffer, shell: Shell) {
        GLOBAL_CONTEXT.with(|global_ref| {
            *global_ref.borrow_mut() = Some(Context {
                stdin_buffer: stdin_buffer,
                stdout_buffer: stdout_buffer,
                shell: shell,
            });
        });
    }
}

fn destroy_default_context() {
    GLOBAL_CONTEXT.with(|global_ref| {
        if let Some(ref mut context) = *global_ref.borrow_mut() {
            context.shell.exit();
        }
        *global_ref.borrow_mut() = None;
    });
}

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
        let builder = gtk::Builder::new_from_string(include_str!("main_window.glade"));
        let main_window: gtk::ApplicationWindow = builder
            .get_object("main_window")
            .expect("could not get main window");

        let input_view: gtk::TextView = builder
            .get_object("input_view")
            .expect("could not get input view from builder");

        let stdout_view: gtk::TextView = builder
            .get_object("output_view")
            .expect("could not get output view from builder");

        let stdout_buffer: gtk::TextBuffer = stdout_view
            .get_buffer()
            .expect("could not get buffer from output view");

        let buffer = input_view
            .get_buffer()
            .expect("couldn't get input text buffer");

        let buffer_clone = buffer.clone();

        Context::create_global_context(buffer.clone(), stdout_buffer, shell);

        self.app.connect_startup(move |app| {
            main_window.set_application(app);
            main_window.connect_delete_event(|win, _| {
                win.destroy();
                Inhibit(false)
            });

            main_window.show_all();

            input_view.connect_key_press_event(move |view, key| {
                let mut buffer = view.get_buffer().expect("couldn't get input text buffer");

                GLOBAL_CONTEXT.with(|global_ref| {
                    if let Some(ref mut context) = *global_ref.borrow_mut() {
                        let ref mut shell = context.shell;
                        let ref mut buffer = context.stdin_buffer;

                        match process_key_event(view, key) {
                            Some(ui::Event::Submit(string)) => {
                                shell
                                    .execute(&string)
                                    .expect("shell could not execute command");
                                // TODO
                                // figure out how to keep the enter key from making a new line
                                buffer.set_text("");
                            }
                            _ => (),
                        }
                    }
                });

                Inhibit(false)
            });
        });

        gtk::idle_add(receive_stdout);

        // included to suppress warnings
        self.app.connect_activate(|_| {});

        self.app.run(&::std::env::args().collect::<Vec<_>>());
        return Ok(());
    }
}

fn receive_stdout() -> glib::Continue {
    return glib::Continue(GLOBAL_CONTEXT.with(|global_ref| {
        if let Some(ref mut context) = *global_ref.borrow_mut() {
            let ref mut shell = context.shell;
            let ref mut stdout_buffer = context.stdout_buffer;

            match shell.poll_stdout() {
                Ok(Some(bytes)) => match ::std::str::from_utf8(&bytes) {
                    Ok(s) => {
                        let mut end_iter = stdout_buffer.get_end_iter();
                        stdout_buffer.insert(&mut end_iter, s);
                        return true;
                    }
                    Err(e) => {
                        eprintln!("unable to parse string from shell output:\n{}", e);
                        return false;
                    }
                },
                Err(e) => {
                    eprintln!("shell output stream closed:\n{}", e);
                    return false;
                }
                _ => return true,
            }
        } else {
            eprintln!("couldn't get context");
            return false;
        }
    }));
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
                            let mut text = buffer.get_text(&start, &end, true).unwrap_or("".into());
                            text.push('\n');
                            text
                        })
                        .unwrap_or("".into()),
                ))
            }
            _ => (),
        }
    }

    return None;
}
