use async_std::prelude::FutureExt as _;

#[derive(Clone, Debug, Default)]
pub struct List {
    pub inner: todo_txt::task::List<super::Task>,
    todo: String,
    done: String,
}

impl List {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_files(todo: &str, done: &str) -> Self {
        let mut list = Self::new();

        list.todo = todo.to_string();
        list.done = done.to_string();

        async_std::task::block_on(async {
            let todo = list.load_file(0, todo).await;
            list.inner.extend(todo);

            let done = list.load_file(list.inner.len(), done).await;
            list.inner.extend(done);
        });

        list
    }

    async fn load_file(&self, first_id: usize, path: &str) -> Vec<crate::tasks::Task> {
        use async_std::io::BufReadExt as _;
        use async_std::stream::StreamExt as _;

        let mut tasks = Vec::new();
        let Ok(file) = async_std::fs::File::open(path).await else {
            log::error!("Unable to open {path:?}");

            return tasks;
        };

        let mut last_id = first_id;
        let mut lines = async_std::io::BufReader::new(file).lines();

        while let Some(line) = lines.next().await {
            let line = line.unwrap();

            if line.is_empty() {
                continue;
            }

            let mut task = crate::tasks::Task::from(line);
            task.id = last_id;
            last_id += 1;
            tasks.push(task);
        }

        tasks
    }

    pub fn projects(&self) -> Vec<String> {
        let today = crate::date::today();

        self.inner
            .iter()
            .filter(|x| {
                !x.finished && (x.threshold_date.is_none() || x.threshold_date.unwrap() <= today)
            })
            .collect::<todo_txt::task::List<_>>()
            .projects()
    }

    pub fn contexts(&self) -> Vec<String> {
        let today = crate::date::today();

        self.inner
            .iter()
            .filter(|x| {
                !x.finished && (x.threshold_date.is_none() || x.threshold_date.unwrap() <= today)
            })
            .collect::<todo_txt::task::List<_>>()
            .contexts()
    }

    pub fn write(&self) -> Result<(), String> {
        async_std::task::block_on(async {
            let (done, todo) = self
                .inner
                .tasks
                .clone()
                .into_iter()
                .partition(|x| x.finished);

            let (a, b) = async { self.write_tasks(&self.todo, todo).await }
                .join(async { self.write_tasks(&self.done, done).await })
                .await;

            a.and(b)
        })?;

        Ok(())
    }

    async fn write_tasks(&self, file: &str, tasks: Vec<crate::tasks::Task>) -> Result<(), String> {
        use async_std::io::WriteExt as _;

        self.backup(file).await?;

        let mut f = match async_std::fs::File::create(file).await {
            Ok(f) => f,
            Err(err) => return Err(format!("Unable to write tasks: {err}")),
        };

        for mut task in tasks {
            if let Err(err) = task.note.write() {
                log::error!("Unable to save note: {err}");
                task.note = todo_txt::task::Note::None;
            }

            match f.write_all(format!("{task}\n").as_bytes()).await {
                Ok(_) => (),
                Err(err) => log::error!("Unable to write tasks: {err}"),
            };
        }

        f.sync_all().await.map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn backup(&self, file: &str) -> Result<(), String> {
        let bak = format!("{file}.bak");

        match async_std::fs::copy(file, bak).await {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Unable to backup {file}")),
        }
    }

    pub fn add(&mut self, text: &str) -> Result<(), String> {
        use std::str::FromStr as _;

        let mut task = crate::tasks::Task::from_str(text)
            .map_err(|_| format!("Unable to convert task: '{text}'"))?;

        task.create_date = Some(crate::date::today());

        self.append(task);
        self.write()
    }

    pub fn append(&mut self, task: crate::tasks::Task) {
        self.inner.push(task);
    }
}

impl std::ops::Deref for List {
    type Target = todo_txt::task::List<super::Task>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
