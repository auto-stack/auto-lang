// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Init,
    AddTodo,
    ToggleTodo(i32),
    DeleteTodo(i32),
    ToggleAll,
    EditTodo(i32),
    CommitEdit,
    EditInputChanged,
    FilterAll,
    FilterActive,
    FilterCompleted,
    ClearCompleted,
}

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub todos: Vec<serde_json::Value>,
    pub next_id: i32,
    pub filter: String,
    pub editing_id: i32,
    pub edit_text: String,
    pub active_count: i32,
}

impl App {
    pub fn new() -> Self {
        let mut __self = Self {
            input: "".to_string(),
            todos: vec![],
            next_id: 1,
            filter: "all".to_string(),
            editing_id: -1,
            edit_text: "".to_string(),
            active_count: 0,
        };
        __self.on(AppMsg::Init);
        __self
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = AppMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::CommitEdit => {
                if self.editing_id >= 0 { if self.edit_text == "".to_string() { let idx = self.todos.iter().position(|t| t["id"].as_i64().unwrap_or(0) as i32 == self.editing_id).map(|i| i as i32).unwrap_or(-1); if idx >= 0 { let mut todo = &mut self.todos[idx as usize]; if todo["done"].as_bool().unwrap_or(false) == false { self.active_count = self.active_count - 1 }; drop(self.todos.remove(idx as usize)) } } else { let idx = self.todos.iter().position(|t| t["id"].as_i64().unwrap_or(0) as i32 == self.editing_id).map(|i| i as i32).unwrap_or(-1); if idx >= 0 { self.todos[idx as usize]["text"] = serde_json::json!(self.edit_text) } }; self.editing_id = -1; self.edit_text = "".to_string() }
            }
            AppMsg::AddTodo => {
                if self.input != "".to_string() { self.todos.push(serde_json::json!({"id": self.next_id, "text": self.input, "done": false})); self.next_id = self.next_id + 1; self.active_count = self.active_count + 1; self.input = "".to_string() }
            }
            AppMsg::FilterActive => {
                self.filter = "active".to_string()
            }
            AppMsg::EditInputChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.input = _text.clone();
                self.edit_text = _text;
                self.input = self.input.clone()
            }
            AppMsg::FilterCompleted => {
                self.filter = "completed".to_string()
            }
            AppMsg::ClearCompleted => {
                let mut i = self.todos.len() as i32 - 1;
                while i >= 0 { if i < self.todos.len() as i32 { let mut todo = &mut self.todos[i as usize]; if todo["done"].as_bool().unwrap_or(false) { drop(self.todos.remove(i as usize)) } }; i = i - 1 }
            }
            AppMsg::EditTodo(id) => {
                self.editing_id = id;
                let idx = self.todos.iter().position(|t| t["id"].as_i64().unwrap_or(0) as i32 == id).map(|i| i as i32).unwrap_or(-1);
                if idx >= 0 { self.edit_text = self.todos[idx as usize]["text"].as_str().unwrap_or_default().to_string().to_string() }
            }
            AppMsg::ToggleTodo(id) => {
                let idx = self.todos.iter().position(|t| t["id"].as_i64().unwrap_or(0) as i32 == id).map(|i| i as i32).unwrap_or(-1);
                if idx >= 0 { let mut todo = &mut self.todos[idx as usize]; if todo["done"].as_bool().unwrap_or(false) { self.active_count = self.active_count + 1 } else { self.active_count = self.active_count - 1 }; todo["done"] = serde_json::json!(!(todo["done"].as_bool().unwrap_or(false))) }
            }
            AppMsg::FilterAll => {
                self.filter = "all".to_string()
            }
            AppMsg::DeleteTodo(id) => {
                let idx = self.todos.iter().position(|t| t["id"].as_i64().unwrap_or(0) as i32 == id).map(|i| i as i32).unwrap_or(-1);
                if idx >= 0 { let mut todo = &mut self.todos[idx as usize]; if todo["done"].as_bool().unwrap_or(false) == false { self.active_count = self.active_count - 1 }; drop(self.todos.remove(idx as usize)) }
            }
            AppMsg::ToggleAll => {
                if self.active_count > 0 { for mut todo in self.todos.iter_mut() { if todo["done"].as_bool().unwrap_or(false) == false { todo["done"] = serde_json::json!(true) } }; self.active_count = 0 } else { for mut todo in self.todos.iter_mut() { if todo["done"].as_bool().unwrap_or(false) { todo["done"] = serde_json::json!(false) } }; self.active_count = self.todos.len() as i32 }
            }
            AppMsg::Init => {
                self.active_count = 0
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("items-center bg-[#f5f5f5] h-full pt-[100px] text-[14px] text-[#111] leading-[1.4] font-['Helvetica_Neue',Helvetica,Arial,sans-serif] font-light antialiased").child(View::text_styled("todos".to_string(), "text-4xl font-bold text-[80px] font-extralight text-center text-[#b83f45]")).child(View::col().style("bg-white mb-10 shadow-[0_2px_4px_0_rgba(0,0,0,0.2),0_25px_50px_0_rgba(0,0,0,0.1)] max-w-[550px] mx-auto").child(View::input("What needs to be done?").value(format!("{}", self.input)).style("w-full text-2xl font-inherit leading-[1.4] text-inherit px-4 pl-[60px] h-[65px] border-none outline-none placeholder:italic placeholder:font-normal placeholder:text-black/40").on_submit(AppMsg::AddTodo).on_change(AppMsg::EditInputChanged).build()).child(if self.todos.len ( ) > 0 { View::col().style("relative z-10 border-t border-[#e6e6e6]").child(View::col().style("relative h-0").child(View::Checkbox { is_checked: false, label: "".to_string(), on_toggle: Some(AppMsg::ToggleAll), style: Some(auto_lang::ui::style::Style::parse("w-px h-px opacity-0 absolute").unwrap()) }).child(View::text_styled("❯".to_string(), "absolute -top-[65px] left-[8px] text-[22px] text-[#949494] cursor-pointer px-[9px] py-[10px] rotate-90")).build()).child(View::col().style("m-0 p-0 list-none").children(self.todos.iter().map(|todo| { if self.filter == "all" || ( self.filter == "active" && todo["done"].as_bool().unwrap_or(false) == false ) || ( self.filter == "completed" && todo["done"].as_bool().unwrap_or(false) == true ) { View::col().with_style(auto_lang::ui::style::Style::parse(&format!("{} {}", if todo["done"].as_bool().unwrap_or(false) { "completed" } else { "" }, if self.editing_id == todo["id"].as_i64().unwrap_or(0) as i32 { "editing" } else { "" })).unwrap_or_default()).style("text-2xl border-b border-[#ededed]").child(if self.editing_id == todo["id"].as_i64().unwrap_or(0) as i32 { View::input("").value(format!("{}", self.edit_text)).style("w-[calc(100%-43px)] py-3 px-4 ml-[43px] text-2xl font-inherit leading-[1.4] text-inherit border border-[#999] shadow-[inset_0_-1px_5px_0_rgba(0,0,0,0.2)] outline-none").on_change(AppMsg::EditInputChanged).on_submit(AppMsg::CommitEdit).build() } else { View::Empty }).child(if self.editing_id != todo["id"].as_i64().unwrap_or(0) as i32 { View::row().style("flex items-center").child(View::Checkbox { is_checked: todo["done"].as_bool().unwrap_or(false), label: "".to_string(), on_toggle: Some(AppMsg::ToggleTodo(todo["id"].as_i64().unwrap_or(0) as i32)), style: Some(auto_lang::ui::style::Style::parse("w-[30px] h-[30px] shrink-0 ml-2").unwrap()) }).child(View::text_styled(todo["text"].as_str().unwrap_or_default().to_string(), "py-[15px] px-[15px] font-normal flex-1 text-[24px]")).child(View::button("×").style("bg-transparent border-none w-[40px] h-[40px] text-[24px] text-[#949494]").on_click(|_| AppMsg::DeleteTodo(todo["id"].as_i64().unwrap_or(0) as i32)).build()).build() } else { View::Empty }).build() } else { View::Empty } })).build()).build() } else { View::Empty }).child(if self.todos.len ( ) > 0 { View::row().style("flex items-center justify-between px-4 py-[10px] min-h-[40px] text-[15px] border-t border-[#e6e6e6] relative").child(View::text_styled(format!("{} items left", self.active_count), "text-left whitespace-nowrap")).child(View::row().style("flex flex-1 justify-center gap-1 m-0 p-0").child(if self.filter == "all" { View::button("All").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-[#CE4646] rounded-[3px] cursor-pointer bg-transparent shadow-[0_0_2px_1px_#CF7D7D]").on_click(|_| AppMsg::FilterAll).build() } else { View::Empty }).child(if self.filter != "all" { View::button("All").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-transparent rounded-[3px] cursor-pointer bg-transparent hover:border-[#DB7676]").on_click(|_| AppMsg::FilterAll).build() } else { View::Empty }).child(if self.filter == "active" { View::button("Active").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-[#CE4646] rounded-[3px] cursor-pointer bg-transparent shadow-[0_0_2px_1px_#CF7D7D]").on_click(|_| AppMsg::FilterActive).build() } else { View::Empty }).child(if self.filter != "active" { View::button("Active").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-transparent rounded-[3px] cursor-pointer bg-transparent hover:border-[#DB7676]").on_click(|_| AppMsg::FilterActive).build() } else { View::Empty }).child(if self.filter == "completed" { View::button("Completed").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-[#CE4646] rounded-[3px] cursor-pointer bg-transparent shadow-[0_0_2px_1px_#CF7D7D]").on_click(|_| AppMsg::FilterCompleted).build() } else { View::Empty }).child(if self.filter != "completed" { View::button("Completed").style("text-inherit mx-[3px] px-[7px] py-[3px] border border-transparent rounded-[3px] cursor-pointer bg-transparent hover:border-[#DB7676]").on_click(|_| AppMsg::FilterCompleted).build() } else { View::Empty }).build()).child(View::button("Clear completed").style("whitespace-nowrap cursor-pointer hover:underline bg-transparent border-none text-inherit").on_click(|_| AppMsg::ClearCompleted).build()).build() } else { View::Empty }).build()).child(View::col().style("mt-16 mx-auto text-[11px] text-[#4d4d4d]").child(View::text_styled("Double-click to edit a todo".to_string(), "block leading-none text-center w-full")).child(View::text_styled("Written with Auto Language".to_string(), "block leading-none text-center w-full")).child(View::text_styled("Part of TodoMVC".to_string(), "block leading-none text-center w-full")).build()).build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("todo");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
