// Unified List Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates list rendering and item management.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run -p auto-lang --example ui_list --features ui-iced
//   cargo run -p auto-lang --example ui_list --features ui-gpui
//
use auto_lang::ui::{Component, View};

#[derive(Debug)]
struct Task {
    id: usize,
    title: String,
    completed: bool,
}

#[derive(Debug, Default)]
struct ListApp {
    tasks: Vec<Task>,
    selected_filter: Filter,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ToggleTask(usize),
    RemoveTask(usize),
    SetFilter(Filter),
}

impl ListApp {
    fn new() -> Self {
        Self {
            tasks: vec![
                Task {
                    id: 1,
                    title: "Learn AutoUI".to_string(),
                    completed: true,
                },
                Task {
                    id: 2,
                    title: "Build components".to_string(),
                    completed: true,
                },
                Task {
                    id: 3,
                    title: "Create examples".to_string(),
                    completed: false,
                },
                Task {
                    id: 4,
                    title: "Write documentation".to_string(),
                    completed: false,
                },
                Task {
                    id: 5,
                    title: "Test all features".to_string(),
                    completed: false,
                },
                Task {
                    id: 6,
                    title: "Polish UI design".to_string(),
                    completed: false,
                },
                Task {
                    id: 7,
                    title: "Optimize performance".to_string(),
                    completed: false,
                },
                Task {
                    id: 8,
                    title: "Add styling system".to_string(),
                    completed: false,
                },
            ],
            selected_filter: Filter::All,
        }
    }

    fn filtered_tasks(&self) -> Vec<&Task> {
        match self.selected_filter {
            Filter::All => self.tasks.iter().collect(),
            Filter::Active => self.tasks.iter().filter(|t| !t.completed).collect(),
            Filter::Completed => self.tasks.iter().filter(|t| t.completed).collect(),
        }
    }
}

impl Component for ListApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ToggleTask(id) => {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    task.completed = !task.completed;
                }
            }
            Message::RemoveTask(id) => {
                self.tasks.retain(|t| t.id != id);
            }
            Message::SetFilter(filter) => {
                self.selected_filter = filter;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Task List Demo".to_string()))
            .child(self.view_filter_buttons())
            .child(self.view_task_list())
            .build()
    }
}

impl ListApp {
    fn view_filter_buttons(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button(
                format!("All ({})", self.tasks.len()),
                Message::SetFilter(Filter::All),
            ))
            .child(View::button(
                format!(
                    "Active ({})",
                    self.tasks.iter().filter(|t| !t.completed).count()
                ),
                Message::SetFilter(Filter::Active),
            ))
            .child(View::button(
                format!(
                    "Completed ({})",
                    self.tasks.iter().filter(|t| t.completed).count()
                ),
                Message::SetFilter(Filter::Completed),
            ))
            .build()
    }

    fn view_task_list(&self) -> View<Message> {
        let filtered = self.filtered_tasks();

        if filtered.is_empty() {
            return View::container(View::text("No tasks to display".to_string()))
                .padding(20)
                .center()
                .build();
        }

        let task_items: Vec<View<Message>> = filtered
            .iter()
            .map(|task| self.view_task_item(task))
            .collect();

        // Wrap list in scrollable container for better UX
        View::scrollable(View::list(task_items).spacing(8).build())
            .height(400)
            .build()
    }

    fn view_task_item(&self, task: &Task) -> View<Message> {
        let status_icon = if task.completed { "\u{2713}" } else { "\u{25cb}" };

        View::row()
            .spacing(12)
            .padding(12)
            .child(View::text(format!("{} {}", status_icon, task.title)))
            .child(View::button(
                "Toggle".to_string(),
                Message::ToggleTask(task.id),
            ))
            .child(View::button(
                "Remove".to_string(),
                Message::RemoveTask(task.id),
            ))
            .build()
    }
}



// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<ListApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend (with auto-conversion!)");
        return auto_lang::ui::gpui::run_app::<ListApp>("List Demo - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "No backend enabled!\n\n\
             Please run with a backend feature:\n\
             \u{2022} cargo run --features ui-iced\n\
             \u{2022} cargo run --features ui-gpui"
                .into(),
        )
    }
}
