use druid::widget::{TextBox, Flex};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};

pub fn create_ui() -> Result<(), PlatformError>{
    let main_window = WindowDesc::new(ui_builder);
    let data = String::new();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<String> {
    let input_box = TextBox::new().with_placeholder("input")
        .expand_width();

    Flex::column()
        .with_flex_child(input_box, 1.0)
}
