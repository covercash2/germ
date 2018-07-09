pub mod backend;

use font_loader::system_fonts;
use font_loader::system_fonts::FontPropertyBuilder;

pub use super::config::Config;
pub use super::constants::{DEFAULT_DIMENSIONS, DEFAULT_FONT, DEFAULT_TITLE};

pub trait Ui {
    type Events: IntoIterator<Item = Event>;

    fn draw(&mut self) -> Result<(), String>;
    fn events(&mut self) -> Self::Events;
    fn set_output(&mut self, string: &str);
}

pub trait TextView {
    fn get_text(&self) -> &String;
    fn set_text(&mut self, string: &str);
}

#[derive(Debug)]
pub enum Event {
    Submit(String),
    Exit,
}

pub fn load_font(family: &str) -> Result<Vec<u8>, String> {
    let property = FontPropertyBuilder::new().family(family).build();

    return system_fonts::get(&property)
        .map(|(font_bytes, _)| font_bytes) // get rid of c_int font Note (?)
        .ok_or(format!("could not load font: {}", family));
}

#[cfg(test)]
mod tests {
    use super::load_font;
    use constants::DEFAULT_FONT;
    use font_loader::system_fonts;

    #[test]
    fn test_load_font() {
        for font in system_fonts::query_all() {
            eprintln!("font found: {:?}", font);
        }

        let font_name = DEFAULT_FONT;
        let font = load_font(font_name).expect("could not load default font");

        // TODO finish
        // assert!(false);
    }
}
