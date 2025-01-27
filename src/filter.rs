pub struct Filter(Box<dyn Fn() -> Vec<crate::tasks::Task>>);

impl Filter {
    pub fn from<F: Fn() -> Vec<crate::tasks::Task> + 'static>(f: F) -> Self {
        Self(Box::new(f))
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self(Box::new(Vec::new))
    }
}

impl From<()> for Filter {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl std::ops::Deref for Filter {
    type Target = Box<dyn Fn() -> Vec<crate::tasks::Task>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
