use gtk::prelude::*;

#[derive(Debug)]
pub enum MsgInput {
    Map,
    NeedUpdate,
    Outdated,
    Update(Vec<crate::tasks::Task>),
}

#[derive(Default)]
pub struct Model {
    children: Vec<relm4::Controller<super::task::Model>>,
    tasks: Vec<crate::tasks::Task>,
    outdated: bool,
    filter: crate::Filter,
}

impl Model {
    fn map(&mut self, widgets: &ModelWidgets, sender: &relm4::ComponentSender<Self>) {
        use relm4::Component as _;
        use relm4::ComponentController as _;

        self.outdated = false;
        self.clear(widgets);

        widgets.outdated.set_visible(false);
        widgets.outdated.stop();

        if self.tasks.is_empty() {
            widgets.list_box.set_visible(false);
            widgets.nothing.set_visible(true);
            return;
        }

        widgets.list_box.set_visible(true);
        widgets.nothing.set_visible(false);

        let mut sorted_tasks = self.tasks.clone();
        sorted_tasks.sort();
        sorted_tasks.reverse();

        for task in &sorted_tasks {
            let child = super::task::Model::builder()
                .launch(task.clone())
                .forward(sender.output_sender(), std::convert::identity);

            widgets.list_box.append(child.widget());

            self.children.push(child);
        }
    }

    fn outdated(&mut self, widgets: &ModelWidgets) {
        self.outdated = true;
        widgets.list_box.set_visible(false);
        widgets.nothing.set_visible(false);
        widgets.outdated.set_visible(true);
        widgets.outdated.start();
    }

    fn clear(&mut self, widgets: &ModelWidgets) {
        use relm4::RelmRemoveAllExt as _;

        widgets.list_box.remove_all();
        self.children = Vec::new();
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = crate::Filter;
    type Input = MsgInput;
    type Output = crate::widgets::task::MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            filter: init,

            ..Default::default()
        };

        let widgets = view_output!();
        sender.input(MsgInput::Outdated);

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        use MsgInput::*;

        match msg {
            Outdated => self.outdated(widgets),
            Map => self.map(widgets, &sender),
            NeedUpdate => {
                self.tasks = (self.filter)();

                if root.is_drawable() {
                    sender.input(MsgInput::Map);
                }
            }
            Update(tasks) => {
                self.tasks = tasks.clone();

                if root.is_drawable() {
                    sender.input(MsgInput::Map);
                }
            }
        }
    }

    view! {
        gtk::ScrolledWindow {
            gtk::Box {
                #[name = "list_box"]
                gtk::ListBox {
                    set_hexpand: true,
                    set_vexpand: true,
                },
                #[name = "nothing"]
                gtk::Label {
                    set_hexpand: true,
                    set_text: "Nothing to do :)",
                    set_vexpand: true,
                },
                #[name = "outdated"]
                gtk::Spinner {
                },
            },

            connect_map => MsgInput::Map,
        },
    }
}
