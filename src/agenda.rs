use chrono::Datelike as _;
use gtk::prelude::*;

#[derive(Debug)]
pub enum MsgInput {
    CalendarChange(Change),
    DateSelect(chrono::NaiveDate),
    Update,
}

#[derive(Debug)]
pub enum Change {
    PrevMonth,
    PrevYear,
    NextMonth,
    NextYear,
}

#[derive(Debug)]
pub enum MsgOutput {
    Complete(Box<crate::tasks::Task>),
    Edit(Box<crate::tasks::Task>),
}

macro_rules! create {
    ($sender:ident) => {{
        let component = crate::widgets::tasks::Model::builder().launch(()).forward(
            $sender.output_sender(),
            |output| match output {
                crate::widgets::task::MsgOutput::Complete(task) => MsgOutput::Complete(task),
                crate::widgets::task::MsgOutput::Edit(task) => MsgOutput::Edit(task),
            },
        );
        component
            .widget()
            .set_vscrollbar_policy(gtk::PolicyType::Never);

        component
    }};
}

macro_rules! update {
    ($self:ident, $exp:expr, $task:ident, $get:ident, $list:ident, $date:ident) => {{
        use relm4::ComponentController as _;

        let tasks = $self.$get(&$list, $date);

        $exp.set_expanded(!tasks.is_empty());
        $exp.set_sensitive(!tasks.is_empty());
        $self.$task.emit(crate::widgets::tasks::Msg::Update(tasks));
    }};
}

pub struct Model {
    date: chrono::NaiveDate,
    month: relm4::Controller<crate::widgets::tasks::Model>,
    past: relm4::Controller<crate::widgets::tasks::Model>,
    today: relm4::Controller<crate::widgets::tasks::Model>,
    tomorrow: relm4::Controller<crate::widgets::tasks::Model>,
    week: relm4::Controller<crate::widgets::tasks::Model>,
}

impl Model {
    fn update_tasks(&self, widgets: &ModelWidgets) {
        let list = crate::application::tasks();
        let date = crate::date::from_glib(widgets.calendar.date());

        update!(self, widgets.past_exp, past, past_tasks, list, date);
        update!(self, widgets.today_exp, today, today_tasks, list, date);
        update!(
            self,
            widgets.tomorrow_exp,
            tomorrow,
            tomorrow_tasks,
            list,
            date
        );
        update!(self, widgets.week_exp, week, week_tasks, list, date);
        update!(self, widgets.month_exp, month, month_tasks, list, date);
    }

    fn past_tasks(
        &self,
        list: &crate::tasks::List,
        date: chrono::naive::NaiveDate,
    ) -> Vec<crate::tasks::Task> {
        self.tasks(list, None, Some(date))
    }

    fn today_tasks(
        &self,
        list: &crate::tasks::List,
        date: chrono::naive::NaiveDate,
    ) -> Vec<crate::tasks::Task> {
        self.tasks(list, Some(date), Some(date + chrono::Duration::days(1)))
    }

    fn tomorrow_tasks(
        &self,
        list: &crate::tasks::List,
        date: chrono::naive::NaiveDate,
    ) -> Vec<crate::tasks::Task> {
        self.tasks(
            list,
            Some(date + chrono::Duration::days(1)),
            Some(date + chrono::Duration::days(2)),
        )
    }

    fn week_tasks(
        &self,
        list: &crate::tasks::List,
        date: chrono::naive::NaiveDate,
    ) -> Vec<crate::tasks::Task> {
        self.tasks(
            list,
            Some(date + chrono::Duration::days(2)),
            Some(date + chrono::Duration::weeks(1)),
        )
    }

    fn month_tasks(
        &self,
        list: &crate::tasks::List,
        date: chrono::naive::NaiveDate,
    ) -> Vec<crate::tasks::Task> {
        self.tasks(
            list,
            Some(date + chrono::Duration::weeks(1)),
            Some(date + chrono::Duration::weeks(4)),
        )
    }

    fn tasks(
        &self,
        list: &crate::tasks::List,
        start: Option<chrono::naive::NaiveDate>,
        end: Option<chrono::naive::NaiveDate>,
    ) -> Vec<crate::tasks::Task> {
        let preferences = crate::application::preferences();

        let tasks = list
            .tasks
            .iter()
            .filter(|x| {
                if let Some(due_date) = x.due_date {
                    (preferences.done || !x.finished)
                        && (preferences.defered
                            || x.threshold_date.is_none()
                            || start.is_none()
                            || x.threshold_date.unwrap() <= start.unwrap())
                        && (start.is_none() || due_date >= start.unwrap())
                        && (end.is_none() || due_date < end.unwrap())
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        tasks
    }

    fn update_marks(&self, widgets: &ModelWidgets) {
        use chrono::Datelike as _;

        widgets.calendar.clear_marks();

        let list = crate::application::tasks();
        let date = widgets.calendar.date();
        let month = date.month() as u32;
        let year = date.year();

        for task in &list.tasks {
            let Some(due_date) = task.due_date else {
                continue;
            };

            if due_date.year() == year && due_date.month() == month {
                widgets.calendar.mark_day(due_date.day());
            }
        }
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = chrono::NaiveDate;
    type Input = MsgInput;
    type Output = MsgOutput;

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        use relm4::ComponentController as _;

        let model = Self {
            date: init,
            month: create!(sender),
            past: create!(sender),
            today: create!(sender),
            tomorrow: create!(sender),
            week: create!(sender),
        };

        let widgets = view_output!();
        sender.input(MsgInput::DateSelect(init));

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        use MsgInput::*;

        match msg {
            CalendarChange(change) => {
                self.date = match change {
                    Change::NextMonth => self.date.checked_add_months(chrono::Months::new(1)),
                    Change::NextYear => self.date.checked_add_months(chrono::Months::new(12)),
                    Change::PrevMonth => self.date.checked_sub_months(chrono::Months::new(1)),
                    Change::PrevYear => self.date.checked_sub_months(chrono::Months::new(12)),
                }
                .unwrap();

                self.update_marks(widgets);
            }
            DateSelect(date) => {
                widgets.calendar.set_day(date.day() as i32);
                widgets.calendar.set_month(date.month0() as i32);
                widgets.calendar.set_year(date.year());
                self.date = date;
            }
            Update => (),
        }

        self.update_tasks(widgets);
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            set_margin_end: 10,
            set_margin_start: 10,
            set_margin_top: 10,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                #[name = "calendar"]
                gtk::Calendar {
                    connect_day_selected[sender] => move |this| {
                        sender.input(MsgInput::DateSelect(crate::date::from_glib(this.date())));
                    },
                    connect_next_month => MsgInput::CalendarChange(Change::NextMonth),
                    connect_next_year => MsgInput::CalendarChange(Change::NextYear),
                    connect_prev_month => MsgInput::CalendarChange(Change::PrevMonth),
                    connect_prev_year => MsgInput::CalendarChange(Change::PrevYear),
                },
                gtk::Button {
                    set_label: "Today",
                    connect_clicked => MsgInput::DateSelect(crate::date::today()),
                },
            },
            gtk::ScrolledWindow {
                gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,

                    #[name = "past_exp"]
                    gtk::Expander {
                        set_child: Some(model.past.widget()),
                        set_label: Some("Past due"),
                    },
                    #[name = "today_exp"]
                    gtk::Expander {
                        set_child: Some(model.today.widget()),
                        set_label: Some("Today"),
                    },
                    #[name = "tomorrow_exp"]
                    gtk::Expander {
                        set_child: Some(model.tomorrow.widget()),
                        set_label: Some("Tomorrow"),
                    },
                    #[name = "week_exp"]
                    gtk::Expander {
                        set_child: Some(model.week.widget()),
                        set_label: Some("This week"),
                    },
                    #[name = "month_exp"]
                    gtk::Expander {
                        set_child: Some(model.month.widget()),
                        set_label: Some("This month"),
                    },
                },
            },
        }
    }
}
