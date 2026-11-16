use gtk::prelude::*;

#[derive(Debug)]
pub enum MsgInput {
    Click,
    Toggle,
}
#[derive(Debug)]
pub enum MsgOutput {
    Complete(Box<crate::tasks::Task>),
    Edit(Box<crate::tasks::Task>),
}

pub struct Model {
    task: crate::tasks::Task,
    circle: relm4::Controller<crate::widgets::circle::Model>,
}

impl Model {
    fn date_alias(&self, date: jiff::civil::Date) -> String {
        let today = todo_txt::date::today();

        if date == today {
            String::from("today")
        } else if Some(date) == today.yesterday().ok() {
            String::from("yesterday")
        } else if Some(date) == today.tomorrow().ok() {
            String::from("tomorrow")
        } else {
            date.strftime("%Y-%m-%d").to_string()
        }
    }
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for Model {
    type Init = crate::tasks::Task;
    type Input = MsgInput;
    type Output = MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use crate::tasks::Markup as _;
        use relm4::Component as _;
        use relm4::ComponentController as _;

        let circle = crate::widgets::circle::Model::builder()
            .launch(init.clone())
            .detach();

        let model = Self { task: init, circle };

        let widgets = view_output!();

        if let Some(due) = model.task.due_date {
            let today = todo_txt::date::today();

            if due < today {
                widgets.due_label.add_css_class("past");
            }
        }

        let gesture = gtk::GestureClick::new();
        gesture.connect_pressed(move |_, n_press, _, _| {
            if n_press == 2 {
                sender.input(MsgInput::Click);
            }
        });
        root.add_controller(gesture);

        if !model.task.priority.is_lowest() {
            let priority = (b'a' + u8::from(model.task.priority.clone())) as char;
            root.add_css_class(&format!("pri_{priority}"));
        }

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        match msg {
            MsgInput::Toggle => sender
                .output(MsgOutput::Complete(Box::new(self.task.clone())))
                .ok(),
            MsgInput::Click => sender
                .output(MsgOutput::Edit(Box::new(self.task.clone())))
                .ok(),
        };
    }

    view! {
        gtk::Box {
            add_css_class: "task",
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,

                    gtk::CheckButton {
                        set_active: model.task.finished,

                        connect_toggled => MsgInput::Toggle,
                    },
                    gtk::Label {
                        set_markup: model.task.markup_subject().as_str(),
                        set_xalign: 0.,
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,

                    gtk::MenuButton {
                        set_icon_name: "text-x-generic",
                        set_visible: model.task.has_note(),

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            set_position: gtk::PositionType::Right,

                            gtk::Label {
                                set_markup?: &model.task.note.markup(),
                            },
                        },
                    },
                    #[name="keywords"]
                    gtk::Box {
                        set_visible: !model.task.tags.is_empty(),

                        gtk::Image {
                            set_icon_name: Some("mail-attachment"),
                        },
                        #[name="keywords_label"]
                        gtk::Label {
                            set_text: &model.task.tags.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(" · "),
                        },
                    },
                    gtk::Box {
                        add_css_class: "date",
                        set_halign: gtk::Align::End,
                        set_hexpand: true,
                        set_spacing: 5,
                        set_valign: gtk::Align::End,

                        gtk::Label {
                            add_css_class: "threshold",
                            set_text?: &model.task.threshold_date.map(|x| format!("Deferred until {}", model.date_alias(x))),
                            set_visible: model.task.threshold_date.is_some(),
                        },
                        gtk::Label {
                            set_text: " ➡ ",
                            set_visible: model.task.threshold_date.is_some() && model.task.due_date.is_some(),
                        },
                        #[name="due_label"]
                        gtk::Label {
                            add_css_class: "due",
                            set_text?: &model.task.due_date.map(|x| format!("due {}", model.date_alias(x))),
                            set_visible: model.task.due_date.is_some(),
                        },
                    },
                },
            },
            append: model.circle.widget(),
        }
    }
}
