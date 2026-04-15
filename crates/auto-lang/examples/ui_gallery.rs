// Unified Gallery - 展示所有 Unified 组件
//
// 这个应用展示了 auto-ui 的所有统一抽象组件，包括：
// - 基础组件：Button, Text, Input, Checkbox, Radio
// - 导航组件：Accordion, Sidebar, Tabs, NavigationRail
// - 高级组件：Slider, Progress, Select, Table, List
//
// Run with:
//   cargo run -p auto-lang --example ui_gallery --features ui-iced
//   cargo run -p auto-lang --example ui_gallery --features ui-gpui

use auto_lang::ui::{Component, View};
use auto_lang::ui::view::{AccordionItem, NavigationRailItem, SidebarPosition};

#[derive(Debug)]
struct GalleryApp {
    expanded_groups: Vec<bool>,
    current_page: Page,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Page {
    Welcome,
    // Getting Started
    HelloWorld,

    // Basic Components
    Button,
    Text,
    Input,
    Checkbox,
    Radio,

    // Navigation Components (Plan 010)
    Accordion,
    Sidebar,
    Tabs,
    NavigationRail,

    // Advanced Components
    Slider,
    Progress,
    Select,
    Table,
    List,

    // Additional pages
    Settings,
    About,
}

impl Default for Page {
    fn default() -> Self {
        Page::Welcome
    }
}

#[derive(Clone, Debug)]
enum Message {
    GroupToggled(usize, bool),
    PageSelected(Page),

    // Page-specific messages
    ButtonClicked,
    InputChanged(String),
    CheckboxToggled,
    RadioSelected,
    SliderChanged(f32),
    TabChanged(usize),
    RailNavigate(usize),
}

impl Component for GalleryApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::GroupToggled(index, expanded) => {
                if index < self.expanded_groups.len() {
                    self.expanded_groups[index] = expanded;
                }
            }
            Message::PageSelected(page) => {
                self.current_page = page;
            }
            _ => {
                // Handle page-specific messages
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // 创建左侧导航面板（使用 Accordion）
        let navigation_panel = View::accordion()
            .items(vec![
                // Getting Started
                AccordionItem::new("Getting Started")
                    .with_icon('\u{1f3e0}')
                    .with_children(vec![
                        self.page_item("Welcome", Page::Welcome),
                        self.page_item("Hello World", Page::HelloWorld),
                    ])
                    .with_expanded(self.expanded_groups[0]),

                // Basic Components
                AccordionItem::new("Basic Components")
                    .with_icon('\u{1f4e6}')
                    .with_children(vec![
                        self.page_item("Button", Page::Button),
                        self.page_item("Text", Page::Text),
                        self.page_item("Input", Page::Input),
                        self.page_item("Checkbox", Page::Checkbox),
                        self.page_item("Radio", Page::Radio),
                    ])
                    .with_expanded(self.expanded_groups[1]),

                // Navigation Components (Plan 010)
                AccordionItem::new("Navigation Components")
                    .with_icon('\u{1f9ed}')
                    .with_children(vec![
                        self.page_item("Accordion", Page::Accordion),
                        self.page_item("Sidebar", Page::Sidebar),
                        self.page_item("Tabs", Page::Tabs),
                        self.page_item("Navigation Rail", Page::NavigationRail),
                    ])
                    .with_expanded(self.expanded_groups[2]),

                // Advanced Components
                AccordionItem::new("Advanced Components")
                    .with_icon('\u{1f680}')
                    .with_children(vec![
                        self.page_item("Slider", Page::Slider),
                        self.page_item("Progress", Page::Progress),
                        self.page_item("Select", Page::Select),
                        self.page_item("Table", Page::Table),
                        self.page_item("List", Page::List),
                    ])
                    .with_expanded(self.expanded_groups[3]),
            ])
            .allow_multiple(true)
            .on_toggle(|idx, expanded| Message::GroupToggled(idx, expanded))
            .build();

        // 创建顶部标题栏
        let header = View::col()
            .spacing(5)
            .padding(20)
            .child(View::text("Unified Component Gallery".to_string()))
            .child(View::text("展示所有 auto-ui 统一抽象组件".to_string()))
            .build();

        // 使用 Sidebar 组件创建左侧固定宽度的导航栏
        View::col()
            .child(header)
            .child(
                View::row()
                    .child(
                        View::sidebar(navigation_panel, 300.0)
                            .position(SidebarPosition::Left)
                            .build()
                    )
                    .child(self.current_content())
                    .build()
            )
            .build()
    }
}

impl GalleryApp {
    fn page_item(&self, label: &str, page: Page) -> View<Message> {
        View::button(label.to_string(), Message::PageSelected(page))
    }

    fn current_content(&self) -> View<Message> {
        match self.current_page {
            Page::Welcome => self.welcome_page(),
            Page::HelloWorld => self.hello_world_page(),

            // Basic Components
            Page::Button => self.button_page(),
            Page::Text => self.text_page(),
            Page::Input => self.input_page(),
            Page::Checkbox => self.checkbox_page(),
            Page::Radio => self.radio_page(),

            // Navigation Components
            Page::Accordion => self.accordion_page(),
            Page::Sidebar => self.sidebar_page(),
            Page::Tabs => self.tabs_page(),
            Page::NavigationRail => self.navigation_rail_page(),

            // Advanced Components
            Page::Slider => self.slider_page(),
            Page::Progress => self.progress_page(),
            Page::Select => self.select_page(),
            Page::Table => self.table_page(),
            Page::List => self.list_page(),

            // Additional pages
            Page::Settings => self.settings_page(),
            Page::About => self.about_page(),
        }
    }

    // ==================== Pages ====================

    fn welcome_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Welcome to Unified Gallery!".to_string()))
            .child(View::text("这是 auto-ui 统一抽象组件的展示应用。".to_string()))
            .child(View::text("".to_string()))
            .child(View::text("功能特性：".to_string()))
            .child(View::text("• 统一的 API 设计，支持 Iced 和 GPUI 后端".to_string()))
            .child(View::text("• 所有组件使用相同的声明式接口".to_string()))
            .child(View::text("• 内置样式系统和主题支持".to_string()))
            .child(View::text("• 导航组件（Accordion, Sidebar, Tabs, NavigationRail）".to_string()))
            .child(View::text("• 点击左侧分组展开/折叠".to_string()))
            .build()
    }

    fn hello_world_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Hello World".to_string()))
            .child(View::button("Click Me!", Message::ButtonClicked))
            .child(View::text("".to_string()))
            .child(View::text("最简单的示例：一个按钮 + 文本".to_string()))
            .build()
    }

    fn button_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Button 组件".to_string()))
            .child(View::text("基础按钮组件，支持点击事件".to_string()))
            .child(View::text("".to_string()))
            .child(View::button("主要按钮", Message::ButtonClicked))
            .child(View::button("次要按钮", Message::ButtonClicked))
            .child(View::button("警告按钮", Message::ButtonClicked))
            .build()
    }

    fn text_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Text 组件".to_string()))
            .child(View::text("文本显示组件".to_string()))
            .child(View::text("".to_string()))
            .child(View::text("普通文本 (20px)".to_string()))
            .child(View::text("大号文本 (32px)".to_string()))
            .child(View::text("小号文本 (12px)".to_string()))
            .build()
    }

    fn input_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Input 组件".to_string()))
            .child(View::text("文本输入组件".to_string()))
            .child(View::text("".to_string()))
            .child(View::input("请输入用户名...").build())
            .build()
    }

    fn checkbox_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Checkbox 组件".to_string()))
            .child(View::text("复选框组件".to_string()))
            .child(View::text("".to_string()))
            .child(View::checkbox(true, "记住密码").on_toggle(Message::CheckboxToggled))
            .child(View::checkbox(false, "同意条款").on_toggle(Message::CheckboxToggled))
            .build()
    }

    fn radio_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Radio 组件".to_string()))
            .child(View::text("单选按钮组".to_string()))
            .child(View::text("".to_string()))
            .child(View::radio(true, "选项 1").on_select(Message::RadioSelected))
            .child(View::radio(false, "选项 2").on_select(Message::RadioSelected))
            .child(View::radio(false, "选项 3").on_select(Message::RadioSelected))
            .build()
    }

    // ==================== Navigation Components ====================

    fn accordion_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Accordion 组件 (手风琴)".to_string()))
            .child(View::text("可展开/折叠的分组列表".to_string()))
            .child(View::text("支持多组同时展开".to_string()))
            .child(View::text("".to_string()))
            .child(
                View::accordion()
                    .items(vec![
                    AccordionItem::new("基础用法")
                        .with_children(vec![
                            View::text("• 点击标题展开/折叠".to_string()),
                            View::text("• 支持图标显示".to_string()),
                        ])
                        .with_expanded(true),
                    AccordionItem::new("高级特性")
                        .with_children(vec![
                            View::text("• 多组同时展开".to_string()),
                            View::text("• 状态管理".to_string()),
                        ])
                        .with_expanded(false),
                ])
                    .allow_multiple(true)
                    .build()
            )
            .build()
    }

    fn sidebar_page(&self) -> View<Message> {
        let sidebar_content = View::col()
            .spacing(10)
            .padding(10)
            .child(View::text("导航菜单".to_string()))
            .child(View::button("首页", Message::PageSelected(Page::Welcome)))
            .child(View::button("设置", Message::PageSelected(Page::Settings)))
            .child(View::button("关于", Message::PageSelected(Page::About)))
            .build();

        View::row()
            .child(
                View::sidebar(sidebar_content, 200.0)
                    .position(SidebarPosition::Left)
                    .build()
            )
            .child(
                View::col()
                    .spacing(15)
                    .padding(20)
                    .child(View::text("Sidebar 组件".to_string()))
                    .child(View::text("固定宽度侧边栏".to_string()))
                    .child(View::text("常用于应用主导航".to_string()))
                    .build()
            )
            .build()
    }

    fn tabs_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Tabs 组件".to_string()))
            .child(View::text("水平选项卡导航".to_string()))
            .child(View::text("支持多个标签页切换".to_string()))
            .child(View::text("".to_string()))
            .child(
                View::tabs(vec!["首页".to_string(), "配置".to_string(), "关于".to_string()])
                    .contents(vec![
                        View::col()
                            .spacing(10)
                            .child(View::text("首页内容".to_string()))
                            .build(),
                        View::col()
                            .spacing(10)
                            .child(View::text("配置页面".to_string()))
                            .build(),
                        View::col()
                            .spacing(10)
                            .child(View::text("关于页面".to_string()))
                            .build(),
                    ])
                    .selected(0)
                    .on_select(|idx| Message::TabChanged(idx))
                    .build()
            )
            .build()
    }

    fn navigation_rail_page(&self) -> View<Message> {
        View::row()
            .child(
                View::navigation_rail()
                    .items(vec![
                        NavigationRailItem::new('H', "Home"),
                        NavigationRailItem::new('S', "Settings").with_badge("3"),
                        NavigationRailItem::new('P', "Profile"),
                        NavigationRailItem::new('A', "About"),
                    ])
                    .selected(0)
                    .width(72.0)
                    .show_labels(true)
                    .on_select(|idx| Message::RailNavigate(idx))
                    .build()
            )
            .child(
                View::col()
                    .spacing(15)
                    .padding(20)
                    .child(View::text("NavigationRail 组件".to_string()))
                    .child(View::text("紧凑型垂直导航".to_string()))
                    .child(View::text("适用于移动端或紧凑界面".to_string()))
                    .child(View::text("支持徽章显示".to_string()))
                    .build()
            )
            .build()
    }

    // ==================== Advanced Components ====================

    fn slider_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Slider 组件".to_string()))
            .child(View::text("滑块输入组件".to_string()))
            .child(View::text("范围: 0-100, 当前值: 50".to_string()))
            .child(
                View::slider(0.0..=100.0, 50.0, |value| Message::SliderChanged(value))
                    .build()
            )
            .build()
    }

    fn progress_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Progress 组件".to_string()))
            .child(View::text("进度条显示组件".to_string()))
            .child(View::text("".to_string()))
            .child(View::text("25% 完成".to_string()))
            .child(View::progress_bar(0.25))
            .child(View::text("".to_string()))
            .child(View::text("50% 完成".to_string()))
            .child(View::progress_bar(0.50))
            .child(View::text("".to_string()))
            .child(View::text("75% 完成".to_string()))
            .child(View::progress_bar(0.75))
            .child(View::text("".to_string()))
            .child(View::text("100% 完成".to_string()))
            .child(View::progress_bar(1.0))
            .build()
    }

    fn select_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Select 组件".to_string()))
            .child(View::text("下拉选择组件".to_string()))
            .child(View::text("".to_string()))
            .child(
                View::select(vec![
                    "选项 1".to_string(),
                    "选项 2".to_string(),
                    "选项 3".to_string()
                ])
            )
            .build()
    }

    fn table_page(&self) -> View<Message> {
        let headers = vec![
            View::text("姓名".to_string()),
            View::text("年龄".to_string()),
            View::text("城市".to_string()),
        ];

        let rows = vec![
            vec![
                View::text("张三".to_string()),
                View::text("25".to_string()),
                View::text("北京".to_string()),
            ],
            vec![
                View::text("李四".to_string()),
                View::text("30".to_string()),
                View::text("上海".to_string()),
            ],
            vec![
                View::text("王五".to_string()),
                View::text("28".to_string()),
                View::text("广州".to_string()),
            ],
        ];

        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("Table 组件".to_string()))
            .child(View::text("表格展示组件".to_string()))
            .child(View::text("".to_string()))
            .child(View::table(headers, rows).build())
            .build()
    }

    fn list_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("List 组件".to_string()))
            .child(View::text("列表展示组件".to_string()))
            .child(View::text("".to_string()))
            .child(
                View::list(vec![
                    View::text("• 列表项 1".to_string()),
                    View::text("• 列表项 2".to_string()),
                    View::text("• 列表项 3".to_string()),
                    View::text("• 列表项 4".to_string()),
                ])
                .build()
            )
            .build()
    }

    fn settings_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("设置".to_string()))
            .child(View::text("应用设置页面".to_string()))
            .child(View::text("".to_string()))
            .child(View::text("这里是配置选项的示例页面。".to_string()))
            .build()
    }

    fn about_page(&self) -> View<Message> {
        View::col()
            .spacing(15)
            .padding(20)
            .child(View::text("关于".to_string()))
            .child(View::text("unified-gallery 示例应用".to_string()))
            .child(View::text("".to_string()))
            .child(View::text("展示了 auto-ui 的所有统一抽象组件。".to_string()))
            .child(View::text("支持 Iced 和 GPUI 后端。".to_string()))
            .build()
    }
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            expanded_groups: vec![true, false, false, false], // Getting Started expanded
            current_page: Page::Welcome,
        }
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        return auto_lang::ui::iced::run_app::<GalleryApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        return auto_lang::ui::gpui::run_app::<GalleryApp>("Unified Gallery");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled. Please enable either 'ui-iced' or 'ui-gpui' feature in Cargo.toml.".into())
    }
}
