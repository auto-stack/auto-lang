// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// 014 内存哨兵:记账分配器(存活字节 + 大块分配点回溯;超限冻结时
// 自动落盘分配报告到 %TEMP%/auto-term-mem-report.txt)。
#[global_allocator]
static GUARD_ALLOC: auto_lang::ui::mem_guard::GuardAlloc = auto_lang::ui::mem_guard::GuardAlloc;

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Debug)]
pub struct App {
    pub name: String,
    pub role: String,
    pub bio: String,
    pub avatar_url: String,
    pub status: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            name: "Jane Cooper".to_string(),
            role: "Full Stack Developer".to_string(),
            bio: "Passionate about building great user experiences. Open source contributor and coffee enthusiast.".to_string(),
            avatar_url: "https://cn.cravatar.com/avatar/e1e7ba949ade0936e071132d2edd3b3c.png".to_string(),
            status: "online".to_string(),
        }
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = ();

    fn on(&mut self, msg: Self::Msg) {
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("p-6 items-center").child(View::col().style("bg-card rounded-lg shadow-lg border border-border w-96 items-center gap-4 pb-6 overflow-hidden").child(View::col().style("w-full h-20 bg-gradient-to-r from-blue-500 to-purple-600 rounded-t-lg").build()).child(View::col().style("-mt-10 items-center").child(View::image_styled(format!("{}", self.avatar_url), "w-20 h-20 rounded-full border-4 border-border shadow-md")).build()).child(View::col().style("gap-1 items-center").child(View::text_styled(format!("{}", self.name), "text-xl font-bold text-foreground")).child(View::row().style("gap-2 items-center").child(View::col().style("w-3 h-3 rounded-full bg-green-400").build()).child(View::text_styled("Active".to_string(), "text-sm text-muted-foreground")).build()).build()).child(View::row().child(View::text_styled(format!("{}", self.role), "px-3 py-1 bg-secondary text-secondary-foreground text-sm rounded-full font-medium")).build()).child(View::text_styled(format!("{}", self.bio), "text-muted-foreground text-sm text-center px-6 leading-relaxed")).child(View::row().style("gap-3").child(View::button("Follow").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90").on_click(|_| ()).build()).child(View::button("Message").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 px-4 py-2 bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/80").on_click(|_| ()).build()).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("name".to_string(), auto_lang::ui::auto_val::Value::str(&self.name));
        m.insert("role".to_string(), auto_lang::ui::auto_val::Value::str(&self.role));
        m.insert("bio".to_string(), auto_lang::ui::auto_val::Value::str(&self.bio));
        m.insert("avatar_url".to_string(), auto_lang::ui::auto_val::Value::str(&self.avatar_url));
        m.insert("status".to_string(), auto_lang::ui::auto_val::Value::str(&self.status));
        m
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        // Plan 020 T-05：孵化参数在册 → native 协议 client 臂（返回即走）；
        // 无标记 → 独立窗（下行 iced_entry 现行行为零变化）。
        let __autodesk_args: Vec<String> = std::env::args().collect();
        let __has_client = __autodesk_args.iter().any(|a| a.starts_with("--autodesk-client="));
        let __has_incubate = __autodesk_args.iter().any(|a| a == "--autodesk-incubate");
        if __has_client || __has_incubate {
            let mut __pipe: Option<String> = None;
            let mut __broker = auto_lang::ui::desktop_protocol::broker::BROKER_PIPE.to_string();
            let mut __render: Option<String> = None;
            let mut __app_name = "profile-card".to_string();
            for __a in &__autodesk_args {
                if let Some(v) = __a.strip_prefix("--autodesk-client=") {
                    __pipe = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--autodesk-broker=") {
                    __broker = v.to_string();
                } else if let Some(v) = __a.strip_prefix("--autodesk-render=") {
                    __render = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--app386=") {
                    __app_name = v.to_string();
                }
            }
            if let Some(arg) = __render.as_deref() {
                if auto_lang::ui::desktop_protocol::coverage::RenderMode::parse(arg).is_none() {
                    eprintln!("[autodesk-client] 未知 --autodesk-render={arg}（auto|queue|independent），回退 auto");
                }
            }
            let __mode = auto_lang::ui::desktop_protocol::coverage::RenderMode::resolve(
                __render.as_deref(),
                None,
            );
            // PLAN-026 T-06 翻转：组件先行构造（auto 裁决 = 覆盖扫描制
            // ——queue 优先 + NotCovered 降级 independent 留痕）。
            let __component = App::default();
            let (__frame_mode, __downgraded, __log) =
                auto_lang::ui::desktop_protocol::client_entry::resolve_native_frame_mode(
                    __mode,
                    "App",
                    &__component.view(),
                );
            if let Some(__l) = &__log {
                eprintln!("[autodesk-client] {__l}");
            }
            let __target = match __pipe {
                Some(p) => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Direct(p),
                None => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Broker {
                    broker_pipe: __broker,
                },
            };
            let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {
                app_name: __app_name.clone(),
                title: __app_name,
                width: 480.0,
                height: 320.0,
                frame_mode: __frame_mode,
                auto_downgraded: __downgraded,
            };
            return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                __component,
                __opts,
                __target,
            )
            .map_err(Into::into);
        }
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        // Plan 020 §5.5：GPUI 臂不接桌面孵化客户端——参数在册报错退出留痕
        //（v1 限 iced；防静默直跑开窗与孵化预期背离）。
        if std::env::args().any(|a| a == "--autodesk-incubate" || a.starts_with("--autodesk-client=")) {
            return Err("native GPUI 臂不接桌面孵化客户端（Plan 020 v1 限 iced）".into());
        }
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("profile-card");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
