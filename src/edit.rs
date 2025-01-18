use gtk::prelude::*;
use relm4::ComponentController as _;

#[derive(Debug)]
pub enum MsgInput {
    Ok,
    Flag(bool),
    Set(Box<crate::tasks::Task>),
    UpdateDate(DateType, Option<chrono::NaiveDate>),
    UpdateKeywords(std::collections::BTreeMap<String, String>),
    UpdatePriority(todo_txt::Priority),
    UpdateRecurrence(Option<todo_txt::task::Recurrence>),
}

#[derive(Debug)]
pub enum MsgOutput {
    Cancel,
    Done(Box<crate::tasks::Task>),
}

pub struct Model {
    created: relm4::Controller<crate::widgets::calendar::Model>,
    due: relm4::Controller<crate::widgets::calendar::Model>,
    finish: relm4::Controller<crate::widgets::calendar::Model>,
    keywords: relm4::Controller<crate::widgets::keywords::Model>,
    priority: relm4::Controller<crate::widgets::priority::Model>,
    recurrence: relm4::Controller<crate::widgets::recurrence::Model>,
    threshold: relm4::Controller<crate::widgets::calendar::Model>,
    task: crate::tasks::Task,
}

impl Model {
    fn update_date(&mut self, date_type: DateType, date: Option<chrono::NaiveDate>) {
        use DateType::*;

        match date_type {
            Due => self.task.due_date = date,
            Threshold => self.task.threshold_date = date,
            Finish => {
                self.task.finish_date = date;
                self.task.finished = date.is_some();
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DateType {
    Due,
    Threshold,
    Finish,
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = crate::tasks::Task;
    type Input = MsgInput;
    type Output = MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let created = crate::widgets::calendar::Model::builder()
            .launch("Created")
            .detach();
        created.widget().set_sensitive(false);

        let due = crate::widgets::calendar::Model::builder()
            .launch("Due")
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::calendar::MsgOutput::Updated(date) => {
                    MsgInput::UpdateDate(DateType::Due, date)
                }
            });

        let keywords = crate::widgets::keywords::Model::builder()
            .launch(init.tags.clone())
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::keywords::MsgOutput::Updated(keywords) => {
                    MsgInput::UpdateKeywords(keywords)
                }
            });

        let finish = crate::widgets::calendar::Model::builder()
            .launch("Completed")
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::calendar::MsgOutput::Updated(date) => {
                    MsgInput::UpdateDate(DateType::Finish, date)
                }
            });

        let priority = crate::widgets::priority::Model::builder()
            .launch(init.priority.clone())
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::priority::MsgOutput::Updated(priority) => {
                    MsgInput::UpdatePriority(priority)
                }
            });

        let recurrence = crate::widgets::recurrence::Model::builder()
            .launch(init.recurrence.clone())
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::recurrence::MsgOutput::Updated(recurrence) => {
                    MsgInput::UpdateRecurrence(recurrence)
                }
            });

        let threshold = crate::widgets::calendar::Model::builder()
            .launch("Defer until")
            .forward(sender.input_sender(), |output| match output {
                crate::widgets::calendar::MsgOutput::Updated(date) => {
                    MsgInput::UpdateDate(DateType::Threshold, date)
                }
            });

        let model = Self {
            created,
            due,
            finish,
            threshold,
            keywords,
            priority,
            task: init,
            recurrence,
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
        use MsgInput::*;

        match msg {
            Flag(flagged) => self.task.flagged = flagged,
            Ok => {
                let start = widgets.buffer.start_iter();
                let end = widgets.buffer.start_iter();
                self.task.note = widgets.buffer.text(&start, &end, true).to_string().into();

                sender
                    .output(MsgOutput::Done(Box::new(self.task.clone())))
                    .ok();
            }
            Set(task) => {
                self.task = *task;
                self.created.emit(crate::widgets::calendar::MsgInput::Set(
                    self.task.create_date,
                ));
                self.due
                    .emit(crate::widgets::calendar::MsgInput::Set(self.task.due_date));
                self.finish.emit(crate::widgets::calendar::MsgInput::Set(
                    self.task.finish_date,
                ));
                self.keywords.emit(crate::widgets::keywords::MsgInput::Set(
                    self.task.tags.clone(),
                ));
                self.threshold.emit(crate::widgets::calendar::MsgInput::Set(
                    self.task.threshold_date,
                ));
            }
            UpdateDate(date_type, date) => self.update_date(date_type, date),
            UpdateKeywords(keywords) => self.task.tags = keywords,
            UpdatePriority(priority) => self.task.priority = priority,
            UpdateRecurrence(recurrence) => self.task.recurrence = recurrence,
        }
    }

    view! {
        gtk::ScrolledWindow {
            set_visible: false,
            set_width_request: 172,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Frame {
                    set_label: Some("Subject"),
                    gtk::Entry {
                        #[watch]
                        set_text: &model.task.subject,
                        connect_activate => MsgInput::Ok,
                    },
                },
                gtk::Frame {
                    set_label: Some("Priority"),
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,

                        append: model.priority.widget(),

                        gtk::ToggleButton {
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            set_icon_name: "emblem-favorite",
                            set_tooltip_text: Some("Flag"),
                            #[watch]
                            set_active: model.task.flagged,

                            connect_toggled[sender] => move |button| {
                                sender.input(MsgInput::Flag(button.is_active()));
                            },
                        },
                    },
                },
                gtk::Frame {
                    set_label: Some("Repeat"),
                    set_child: Some(model.recurrence.widget()),
                },
                gtk::Frame {
                    set_label: Some("Date"),
                    gtk::Box {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Vertical,

                        append: model.threshold.widget(),
                        append: model.due.widget(),
                        append: model.finish.widget(),
                        append: model.created.widget(),
                    },
                },
                gtk::Frame {
                    set_label: Some("Keywords"),

                    set_child: Some(model.keywords.widget()),
                },
                gtk::Frame {
                    set_label: Some("Note"),

                    #[name = "note"]
                    gtk::TextView {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_size_request: (-1, 100),
                        #[wrap(Some)]
                        #[name = "buffer"]
                        set_buffer = &gtk::TextBuffer {
                            #[watch]
                            set_text?: &model.task.note.content(),
                        },
                    },
                },
                gtk::ActionBar {
                    pack_start = &gtk::Button {
                        set_label: "Ok",

                        connect_clicked => MsgInput::Ok,
                    },
                    pack_start = &gtk::Button {
                        set_label: "Cancel",

                        connect_clicked[sender] => move |_| {
                            sender.output(MsgOutput::Cancel).ok();
                        },
                    },
                },
            },
        }
    }
}
