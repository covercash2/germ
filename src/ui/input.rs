use druid::widget::TextBox;
use druid::{Event, KeyCode, KeyEvent, Widget};

pub struct CommandInputBox {
    text: TextBox,
}

impl CommandInputBox {
    pub fn new() -> Self {
        CommandInputBox {
            text: TextBox::new(),
        }
    }
}

impl Widget<String> for CommandInputBox {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut String,
        env: &druid::Env,
    ) {
        self.text.event(ctx, event, data, env);

        match event {
            Event::KeyDown(KeyEvent {
                key_code: KeyCode::Return,
                is_repeat: false,
                mods: _,
		..
            }) => {
		println!("you pressed enter");
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &String,
        env: &druid::Env,
    ) {
        self.text.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &String,
        data: &String,
        env: &druid::Env,
    ) {
        self.text.update(ctx, old_data, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &String,
        env: &druid::Env,
    ) -> druid::Size {
        self.text.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &String, env: &druid::Env) {
        self.text.paint(ctx, data, env)
    }
}
