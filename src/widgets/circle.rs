use gtk::prelude::*;

pub struct Model {}

impl Model {
    fn draw(
        task: &crate::tasks::Task,
        drawing_area: &gtk::DrawingArea,
        context: &gtk::cairo::Context,
    ) -> Result<(), gtk::cairo::Error> {
        let center = f64::min(
            f64::from(drawing_area.width_request()) / 2.,
            f64::from(drawing_area.height_request()) / 2.,
        );

        if !task.finished
            && let Some(due_date) = task.due_date
        {
            let today = crate::date::today();

            if due_date < today {
                context.set_source_rgb(1., 0.4, 0.5);
            } else {
                context.set_source_rgb(1., 0.8, 0.2);
            }
        } else {
            context.set_source_rgb(0.8, 0.8, 0.8);
        }

        context.set_line_width(8.);
        context.arc(center, center, center - 5., 0., 2. * std::f64::consts::PI);
        context.close_path();

        if task.finished {
            let width = drawing_area.width_request();
            let height = drawing_area.height_request();

            context.save()?;
            context.fill()?;
            context.translate(f64::from(width) / -4., f64::from(height) / 2.);
            context.rotate(std::f64::consts::PI / -4.);
            context.set_source_rgb(0., 0., 0.);
            context.rectangle(20., 30., 40., 10.);
            context.rectangle(20., 20., 10., 10.);
            context.fill()?;
            context.restore()?;
        }

        context.stroke()?;

        if !task.finished && task.flagged {
            let angle = if task.due_date.is_some() {
                std::f64::consts::PI
            } else {
                0.
            };

            context.set_source_rgb(1., 0.5, 0.3);
            context.arc(
                center,
                center,
                center - 5.,
                angle,
                2. * std::f64::consts::PI,
            );
            context.stroke()?;
        }

        if !task.finished && task.recurrence.is_some() {
            context.set_line_width(2.);

            for dx in &[-12., 0., 12.] {
                context.arc(center + dx, center, 4., 0., 2. * std::f64::consts::PI);
                context.close_path();
                context.stroke()?;
            }
        }

        Ok(())
    }
}

impl relm4::SimpleComponent for Model {
    type Init = crate::tasks::Task;
    type Input = ();
    type Output = ();
    type Root = gtk::DrawingArea;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::DrawingArea::new()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        root.set_height_request(60);
        root.set_width_request(60);

        let model = Self {};

        root.set_draw_func(move |drawing_area, context, _w, _h| {
            Self::draw(&init, drawing_area, context).ok();
        });

        relm4::ComponentParts { model, widgets: () }
    }
}
