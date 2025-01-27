use gtk::prelude::*;
use relm4::ComponentController as _;

static CURRENT_FILTER: std::sync::LazyLock<std::sync::RwLock<String>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(String::new()));

#[derive(Debug)]
pub enum MsgInput {
    Update,
    UpdateFilter(String),
}

pub struct Model {
    tasks: relm4::Controller<crate::widgets::tasks::Model>,
}

impl Model {
    fn tasks() -> Vec<crate::tasks::Task> {
        let current_filter = CURRENT_FILTER.read().unwrap();

        let filter = current_filter.to_lowercase();
        let list = crate::application::tasks();

        list.tasks
            .iter()
            .filter(|x| x.subject.to_lowercase().contains(filter.as_str()))
            .cloned()
            .collect()
    }
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = ();
    type Input = MsgInput;
    type Output = crate::widgets::task::MsgOutput;

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;

        let tasks = crate::widgets::tasks::Model::builder()
            .launch(crate::Filter::from(Self::tasks))
            .forward(sender.output_sender(), std::convert::identity);

        let model = Self { tasks };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        use MsgInput::*;

        match msg {
            Update => self.tasks.emit(crate::widgets::tasks::MsgInput::NeedUpdate),
            UpdateFilter(filter) => {
                let mut current_filter = CURRENT_FILTER.write().unwrap();
                *current_filter = filter;
                sender.input(Update);
            }
        }
    }

    view! {
        gtk::Box {
            append: model.tasks.widget(),
        }
    }
}
