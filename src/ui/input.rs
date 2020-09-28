use druid::KeyModifiers;
use druid::widget::TextBox;
use druid::{Data, Event, KeyCode, KeyEvent, Lens, Widget};

#[derive(Lens, Data, Clone)]
pub struct CommandInputBuffer {
    input: String,
    pub command: Option<String>,
}

impl Default for CommandInputBuffer {
    fn default() -> Self {
        CommandInputBuffer {
	    input: String::new(),
	    command: None,
	}
    }
}

impl CommandInputBuffer {
    fn submit_command(&mut self) {
	self.command.replace(self.input.clone());
    }

    fn clear(&mut self) {
	self.input.clear()
    }
}

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

impl Widget<CommandInputBuffer> for CommandInputBox {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut CommandInputBuffer,
        env: &druid::Env,
    ) {
        self.text.event(ctx, event, &mut data.input, env);

        match event {
            Event::KeyDown(KeyEvent {
                key_code: KeyCode::Return,
                is_repeat: false,
                mods: _,
                ..
            }) => {
                println!("you pressed enter");
		data.submit_command();
		println!("command submittted: {}", data.command.as_ref().unwrap());
            },
	    Event::KeyDown(KeyEvent {
		key_code: KeyCode::Backspace,
		is_repeat: false,
		mods: KeyModifiers {
		    shift: _,
		    alt: _,
		    ctrl: true,
		    meta: _,
		},
		..
	    }) => {
		println!("backspace");
		data.clear()
	    }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &CommandInputBuffer,
        env: &druid::Env,
    ) {
        self.text.lifecycle(ctx, event, &data.input, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &CommandInputBuffer,
        data: &CommandInputBuffer,
        env: &druid::Env,
    ) {
        self.text.update(ctx, &old_data.input, &data.input, env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &CommandInputBuffer,
        env: &druid::Env,
    ) -> druid::Size {
        self.text.layout(ctx, bc, &data.input, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &CommandInputBuffer, env: &druid::Env) {
        self.text.paint(ctx, &data.input, env)
    }
}
