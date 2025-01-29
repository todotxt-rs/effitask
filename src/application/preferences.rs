#[derive(Clone, Default)]
pub struct Preferences {
    pub defered: bool,
    pub done: bool,
    pub hidden: bool,
}

impl Preferences {
    pub fn new() -> Self {
        Self::default()
    }
}
