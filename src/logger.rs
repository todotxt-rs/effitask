use gtk::prelude::*;

type ChannelData = (log::Level, String);
type Sender = std::sync::mpsc::Sender<ChannelData>;
type Receiver = std::sync::mpsc::Receiver<ChannelData>;

pub struct Log {
    tx: std::sync::Mutex<Sender>,
}

impl Log {
    pub fn new(tx: Sender) -> Self {
        Self {
            tx: std::sync::Mutex::new(tx),
        }
    }
}

impl log::Log for Log {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target() == crate::application::NAME && metadata.level() >= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if let Ok(tx) = self.tx.lock() {
            tx.send((record.level(), format!("{}", record.args()))).ok();
        }
    }

    fn flush(&self) {}
}

thread_local!(
    static GLOBAL: std::cell::RefCell<Option<(relm4::ComponentSender<Model>, Receiver)>>
        = const { std::cell::RefCell::new(None) }
);

#[derive(Debug)]
pub enum Msg {
    Add(ChannelData),
    Clear,
    Read(gtk::ListBoxRow),
}

pub struct Model {
    count: usize,
}

impl Model {
    fn receive() -> gtk::glib::ControlFlow {
        GLOBAL.with(|global| {
            if let Some((ref sender, ref rx)) = *global.borrow() {
                if let Ok(message) = rx.try_recv() {
                    sender.input(Msg::Add(message));
                }
            }
        });

        gtk::glib::ControlFlow::Continue
    }

    fn add_message(&self, widgets: &ModelWidgets, level: log::Level, text: &str) {
        let class = level.to_string();

        let label = gtk::Label::new(Some(text));
        label.add_css_class(&class.to_lowercase());

        widgets.list_box.append(&label);
    }
}

#[relm4::component(pub)]
impl relm4::Component for Model {
    type CommandOutput = ();
    type Init = ();
    type Input = Msg;
    type Output = ();

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let log = Log::new(tx);

        log::set_max_level(log::LevelFilter::Info);
        log::set_boxed_logger(Box::new(log)).unwrap_or_default();

        let model = Self { count: 0 };

        let widgets = view_output!();

        GLOBAL.with(move |global| *global.borrow_mut() = Some((sender, rx)));
        gtk::glib::idle_add(Self::receive);

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
            Msg::Add((level, text)) => {
                self.add_message(widgets, level, &text);
                self.count += 1;
            }
            Msg::Clear => {
                use relm4::RelmRemoveAllExt as _;

                widgets.list_box.remove_all();
                self.count = 0;
            }
            Msg::Read(row) => {
                widgets.list_box.remove(&row);
                self.count = self.count.saturating_sub(1);
            }
        }
    }

    view! {
        gtk::MenuButton {
            #[watch]
            set_visible: model.count > 0,
            #[watch]
            set_label: &format!("Notifications {}", model.count),
            set_direction: gtk::ArrowType::Down,

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                add_css_class: "log",
                set_height_request: 500,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),

                        #[name = "list_box"]
                        gtk::ListBox {
                            connect_row_activated[sender] => move |_, row| sender.input(Msg::Read(row.clone())),
                        }
                    },
                    gtk::Button {
                        set_label: "Clear all",
                        set_icon_name: "list-remove-all",
                        connect_clicked => Msg::Clear,
                    },
                },
            },
        },
    }
}
