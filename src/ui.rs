use druid::widget::{Flex, Label};
use druid::{AppLauncher, Data, Env, Lens, PlatformError, Widget, WidgetExt, WindowDesc};

mod input;

use input::CommandInputBuffer;

#[derive(Clone, Data, Lens)]
struct AppState {
    input: CommandInputBuffer,
    test_msg: String,
}

pub fn create_ui() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = AppState {
        input: Default::default(),
        test_msg: String::new(),
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<AppState> {
    let input_box = input::CommandInputBox::new()
        .lens(AppState::input)
        .expand_width();

    let output_label = Label::new(|data: &AppState, _: &Env| data.test_msg.clone());

    Flex::column()
        .with_flex_child(input_box, 1.0)
        .with_flex_child(output_label, 1.0)
}
