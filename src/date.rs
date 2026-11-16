pub fn from_glib(value: gtk::glib::DateTime) -> jiff::civil::Date {
    let y = value.year() as i16;
    let m = value.month() as i8;
    let d = value.day_of_month() as i8;

    jiff::civil::Date::new(y, m, d).unwrap()
}
