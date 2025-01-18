use gtk::prelude::*;

#[derive(Debug)]
pub enum MsgInput {
    Set(todo_txt::Priority),
    More,
}

#[derive(Debug)]
pub enum MsgOutput {
    Updated(todo_txt::Priority),
}

pub struct Model {
    priority: u8,
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = todo_txt::Priority;
    type Input = MsgInput;
    type Output = MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self { priority: 26 };

        let widgets = view_output!();

        sender.input(MsgInput::Set(init));

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            MsgInput::More => {
                widgets.spin_button.set_visible(true);
                widgets.r#box.set_visible(false);
            }
            MsgInput::Set(priority) => {
                self.priority = priority.into();
                let show_more = (5..=25).contains(&self.priority);
                widgets.r#box.set_visible(!show_more);
                widgets.spin_button.set_visible(show_more);
                widgets.spin_button.set_value(self.priority as f64);

                widgets.a.set_active(self.priority == 0);
                widgets.b.set_active(self.priority == 1);
                widgets.c.set_active(self.priority == 2);
                widgets.d.set_active(self.priority == 3);
                widgets.e.set_active(self.priority == 4);
                widgets.z.set_active(self.priority == 26);
            }
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "r#box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                append: a = &gtk::ToggleButton {
                    set_active: model.priority == 0,
                    set_label: "A",

                    connect_toggled[sender] => move |_| {
                        sender.output(MsgOutput::Updated(0.into())).ok();
                    },
                },
                #[name="b"]
                gtk::ToggleButton {
                    set_active: model.priority == 1,
                    set_group: Some(&a),
                    set_label: "B",

                    connect_toggled[sender] => move |_| {
                        sender.output(MsgOutput::Updated(1.into())).ok();
                    },
                },
                #[name="c"]
                gtk::ToggleButton {
                    set_active: model.priority == 2,
                    set_group: Some(&a),
                    set_label: "C",

                    connect_toggled[sender] => move |_| {
                        sender.output(MsgOutput::Updated(2.into())).ok();
                    },
                },
                #[name="d"]
                gtk::ToggleButton {
                    set_active: model.priority == 3,
                    set_group: Some(&a),
                    set_label: "D",

                    connect_toggled[sender] => move |_| {
                        sender.output(MsgOutput::Updated(3.into())).ok();
                    },
                },
                #[name="e"]
                gtk::ToggleButton {
                    set_active: model.priority == 4,
                    set_group: Some(&a),
                    set_label: "E",

                    connect_toggled[sender] => move |_| {
                        sender.output(MsgOutput::Updated(4.into())).ok();
                    },
                },
                gtk::Button {
                    set_label: "…",
                    set_tooltip_text: Some("More"),

                    connect_clicked => MsgInput::More,
                },
                #[name="z"]
                gtk::ToggleButton {
                    set_active: model.priority == 26,
                    set_group: Some(&a),
                    set_label: "Z",

                    connect_clicked[sender] => move |_| {
                        sender.output(MsgOutput::Updated(26.into())).ok();
                    },
                },
            },
            #[name = "spin_button"]
            gtk::SpinButton {
                set_adjustment: &gtk::Adjustment::new(0., 0., 27., 1., 5., 1.),
                set_climb_rate: 1.,
                set_digits: 0,

                connect_value_changed[sender] => move |button| {
                    let priority = (button.value() as u8).into();
                    sender.output(MsgOutput::Updated(priority)).ok();
                },
            },
        },
    }
}
