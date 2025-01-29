use gtk::prelude::*;

#[derive(Debug)]
pub enum Msg {
    Update,
}

pub struct Model {
    tasks: relm4::Controller<crate::widgets::tasks::Model>,
}

impl Model {
    fn tasks() -> Vec<crate::tasks::Task> {
        let today = crate::date::today();
        let list = crate::application::tasks();
        let preferences = crate::application::preferences();

        list.tasks
            .iter()
            .filter(|x| {
                x.flagged
                    && (preferences.done || !x.finished)
                    && (preferences.hidden || !x.hidden)
                    && (preferences.defered
                        || x.threshold_date.is_none()
                        || x.threshold_date.unwrap() <= today)
            })
            .cloned()
            .collect()
    }
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = ();
    type Input = Msg;
    type Output = crate::widgets::task::MsgOutput;

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;
        use relm4::ComponentController as _;

        let tasks = crate::widgets::tasks::Model::builder()
            .launch(crate::Filter::from(Model::tasks))
            .forward(sender.output_sender(), std::convert::identity);

        let model = Self { tasks };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _: relm4::ComponentSender<Self>) {
        use relm4::ComponentController as _;

        match msg {
            Msg::Update => self.tasks.emit(crate::widgets::tasks::MsgInput::NeedUpdate),
        }
    }

    view! {
        gtk::Box {
            append: model.tasks.widget(),
        }
    }
}
