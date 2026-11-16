use gtk::prelude::*;
use relm4::ComponentController as _;

#[derive(Clone, Copy)]
pub enum Type {
    Projects,
    Contexts,
    Hashtags,
}

#[derive(Debug)]
pub enum MsgInput {
    Complete(Box<crate::tasks::Task>),
    Edit(Box<crate::tasks::Task>),
    UpdateFilters(Vec<String>),
    Update,
}

#[derive(Debug)]
pub enum MsgOutput {
    Complete(Box<crate::tasks::Task>),
    Edit(Box<crate::tasks::Task>),
}

pub struct Model {
    tag: Type,
    filter: relm4::Controller<super::filter::Model>,
}

impl Model {
    fn update_tags(&self) {
        let list = crate::application::tasks();
        let tags = match self.tag {
            Type::Projects => list.projects(),
            Type::Contexts => list.contexts(),
            Type::Hashtags => list.hashtags(),
        };

        let tags = tags
            .iter()
            .map(|x| (x.clone(), self.progress(&list, x)))
            .filter(|&(_, (done, total))| done != total)
            .collect();

        self.filter
            .emit(crate::widgets::filter::MsgInput::UpdateFilters(tags));
    }

    fn progress(&self, list: &crate::tasks::List, current: &str) -> (u32, u32) {
        list.tasks
            .iter()
            .filter(|x| {
                for tag in self.tags(x) {
                    if tag == current || tag.starts_with(&format!("{current}-")) {
                        return true;
                    }
                }

                false
            })
            .fold((0, 0), |(mut done, total), x| {
                if x.finished {
                    done += 1;
                }

                (done, total + 1)
            })
    }

    fn update_tasks(&self, filters: &[String]) {
        let today = todo_txt::date::today();
        let preferences = crate::application::preferences();
        let list = crate::application::tasks();

        let tasks = list
            .tasks
            .iter()
            .filter(|x| {
                let tags = self.tags(x);

                (preferences.done || !x.finished)
                    && !tags.is_empty()
                    && Self::has_filter(tags, filters)
                    && (preferences.defered
                        || x.threshold_date.is_none()
                        || x.threshold_date.unwrap() <= today)
            })
            .cloned()
            .collect();

        self.filter
            .emit(crate::widgets::filter::MsgInput::UpdateTasks(tasks));
    }

    fn tags<'a>(&self, task: &'a crate::tasks::Task) -> &'a [String] {
        match self.tag {
            Type::Projects => &task.projects,
            Type::Contexts => &task.contexts,
            Type::Hashtags => &task.hashtags,
        }
    }

    fn has_filter(tags: &[String], filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        }

        for filter in filters {
            if tags.contains(filter) {
                return true;
            }
        }

        false
    }
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = Type;
    type Input = MsgInput;
    type Output = MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::Component as _;

        let filter =
            super::filter::Model::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    super::filter::MsgOutput::Complete(task) => MsgInput::Complete(task),
                    super::filter::MsgOutput::Edit(task) => MsgInput::Edit(task),
                    super::filter::MsgOutput::Filters(filters) => MsgInput::UpdateFilters(filters),
                });

        let model = Self { tag: init, filter };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        use MsgInput::*;

        match msg {
            Complete(task) => {
                sender.output(MsgOutput::Complete(task)).ok();
            }
            Edit(task) => {
                sender.output(MsgOutput::Edit(task)).ok();
            }
            Update => {
                self.update_tags();
                self.update_tasks(&[]);
            }
            UpdateFilters(filters) => self.update_tasks(&filters),
        }
    }

    view! {
        gtk::Box {
            append: model.filter.widget(),
        }
    }
}
