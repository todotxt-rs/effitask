use gtk::prelude::*;

#[derive(Debug)]
pub enum Msg {
    Update(Vec<crate::tasks::Task>),
}

pub struct Model {
    children: Vec<relm4::Controller<super::task::Model>>,
}

impl Model {
    fn update_tasks(
        &mut self,
        widgets: &ModelWidgets,
        sender: relm4::ComponentSender<Self>,
        tasks: &[crate::tasks::Task],
    ) {
        use relm4::Component as _;
        use relm4::ComponentController as _;

        self.clear(widgets);

        if tasks.is_empty() {
            widgets.label.set_visible(true);
            widgets.list_box.set_visible(false);
            return;
        }

        widgets.label.set_visible(false);
        widgets.list_box.set_visible(true);

        let mut sorted_tasks = tasks.to_owned();
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

    fn clear(&mut self, widgets: &ModelWidgets) {
        use relm4::RelmRemoveAllExt as _;

        widgets.list_box.remove_all();
        self.children = Vec::new();
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = ();
    type Input = Msg;
    type Output = crate::widgets::task::MsgOutput;

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            children: Vec::new(),
        };

        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        use Msg::*;

        match msg {
            Update(tasks) => self.update_tasks(widgets, sender, &tasks),
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
                #[name = "label"]
                gtk::Label {
                    set_hexpand: true,
                    set_text: "Nothing to do :)",
                    set_vexpand: true,
                },
            },
        },
    }
}
