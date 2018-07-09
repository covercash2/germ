use config::Config;
use shell::Shell;
use ui::Ui;

pub struct App<U: Ui> {
    config: Config,
    shell: Shell,
    ui: U,
}

impl<U: Ui> App<U> {
    pub fn new(config: Config, shell: Shell, ui: U) -> Self {
        return App {
            config: config,
            shell: shell,
            ui: ui,
        };
    }

    pub fn run(mut self) -> Result<(), String> {
        self.ui.draw()
    }

    pub fn runa(mut self) -> Result<(), String> {
        'main: loop {
            let events = self.ui.events();

            self.ui.draw();
        }
    }
}
