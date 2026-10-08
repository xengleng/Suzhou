//! The one window: its state, what happens when you click things, and how
//! events from the backend land in conversations.

use std::time::{Duration, Instant};

use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Pixels, Point,
    ScrollHandle, Task, Window, actions, div, prelude::*, px,
};

use super::text_input::{InputEvent, TextInput};
use crate::{
    anim::{self, Tween},
    backend::{self, Backend, Event, Persona, Request},
    data,
    model::*,
    store::{self, Saved, Settings},
    theme::{self, ActiveTheme, FONT, ThemeMode, theme},
};

actions!(
    suzhou,
    [
        NewBot,
        FocusSearch,
        ToggleVoice,
        ToggleTheme,
        OpenSettings,
        OpenPlugins,
        ToggleComputer,
        ToggleDetails,
        ToggleSidebar,
        StopBot,
        Dismiss,
        Quit,
    ]
);

#[derive(Clone, Copy, Default)]
pub struct LaunchOptions {
    pub demo: bool,
    pub reset: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    SignIn,
    Authorizing,
    Tools,
    SettingUp,
    Meet,
    Main,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    NewBot,
    Bot(BotId),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Panel {
    Details,
    Routine(RoutineId),
    Computer,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Modal {
    Plugins,
    Settings,
    File { name: String, content: String },
    Takeover,
    Teach,
    ConfirmDelete(BotId),
    ConfirmReset,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Popover {
    BotMenu { bot: BotId, at: Point<Pixels> },
    Profile { at: Point<Pixels> },
    Plus { at: Point<Pixels> },
    MessageMore { msg: MsgId, at: Point<Pixels> },
    React { msg: MsgId, at: Point<Pixels> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuggestKind {
    Mention,
    Skill,
}

#[derive(Clone, Debug)]
pub struct Suggest {
    pub kind: SuggestKind,
    /// Byte offset of the `@` or `/`.
    pub start: usize,
    pub query: String,
    pub ix: usize,
}

#[derive(Clone, Debug)]
pub struct SuggestItem {
    pub label: String,
    pub detail: String,
    pub section: &'static str,
    pub icon: SuggestIcon,
    pub insert: String,
}

#[derive(Clone, Debug)]
pub enum SuggestIcon {
    Bot(usize, Shape),
    Path(&'static str),
    Glyph(&'static str, u32),
}

pub struct Toast {
    pub id: u64,
    pub bot: BotId,
    pub title: String,
    pub body: String,
    pub t: Tween,
    pub leaving: bool,
}

pub struct Inputs {
    pub composer: Entity<TextInput>,
    pub search: Entity<TextInput>,
    pub bot_name: Entity<TextInput>,
    pub plugin_search: Entity<TextInput>,
    pub routine_name: Entity<TextInput>,
    pub routine_instruction: Entity<TextInput>,
    pub profile_name: Entity<TextInput>,
    pub profile_title: Entity<TextInput>,
    pub profile_description: Entity<TextInput>,
    pub answer: Entity<TextInput>,
}

/// Something that slides or fades in and out: what is showing, and how far.
pub struct Presence<T> {
    pub target: Option<T>,
    pub shown: Option<T>,
    pub t: Tween,
}

impl<T: Clone + PartialEq> Presence<T> {
    fn new() -> Self {
        Presence {
            target: None,
            shown: None,
            t: Tween::new(0.0),
        }
    }

    pub fn open(&mut self, value: T, millis: u64) {
        let was_open = self.target.is_some();
        self.target = Some(value.clone());
        self.shown = Some(value);
        if !was_open {
            self.t.animate_to(1.0, millis, anim::ease_out_quint);
        }
    }

    pub fn close(&mut self, millis: u64) {
        if self.target.take().is_some() {
            self.t.animate_to(0.0, millis, anim::ease_out_cubic);
        }
    }

    pub fn is(&self, value: &T) -> bool {
        self.target.as_ref() == Some(value)
    }

    /// What to draw this frame, with its progress, if anything.
    pub fn visible(&mut self, window: &mut Window) -> Option<(T, f32)> {
        let t = self.t.tick(window);
        if self.target.is_none() && !self.t.is_animating() {
            self.shown = None;
        }
        self.shown.clone().map(|v| (v, t))
    }
}

pub struct AppView {
    pub focus: FocusHandle,
    pub stage: Stage,
    pub tools_picked: Vec<&'static str>,
    pub setup: Tween,
    pub profile: Profile,
    pub bots: Vec<Bot>,
    pub next_id: u64,
    pub page: Page,
    pub draft_color: usize,
    pub draft_shape: Shape,
    pub draft_bump: u32,
    pub draft_role: Option<(String, String)>,
    pub inputs: Inputs,
    pub editing_profile: bool,
    pub panel: Presence<Panel>,
    pub modal: Presence<Modal>,
    pub popover: Presence<Popover>,
    pub suggest: Option<Suggest>,
    pub plugins: Vec<Plugin>,
    pub plugin_category: usize,
    pub connecting: Vec<&'static str>,
    pub toasts: Vec<Toast>,
    pub voice: Option<Instant>,
    pub teach: Option<Instant>,
    pub settings: Settings,
    pub settings_section: usize,
    pub backend: Box<dyn Backend>,
    pub chat_scroll: ScrollHandle,
    pub stick_bottom: bool,
    pub hovered_msg: Option<MsgId>,
    pub reply_to: Option<(MsgId, String)>,
    pub attachments: Vec<String>,
    pub blink: (usize, Option<Instant>),
    pub sidebar: Tween,
    pub cursor: (Tween, Tween),
    pub cursor_bot: Option<BotId>,
    pub show_hidden: bool,
    pub persist: bool,
    pub bubble_max: f32,
    /// Window size this frame, for layouts that need pixel sizes.
    pub vp: (f32, f32),
    pub saved_theme: ThemeMode,
    pub search_cache: String,
    _tasks: Vec<Task<()>>,
}

fn input(
    cx: &mut Context<AppView>,
    placeholder: &str,
    build: impl FnOnce(TextInput) -> TextInput,
) -> Entity<TextInput> {
    let placeholder = placeholder.to_string();
    cx.new(|cx| build(TextInput::new(placeholder, cx)))
}

impl AppView {
    pub fn new(options: LaunchOptions, window: &mut Window, cx: &mut Context<Self>) -> Self {
        if options.reset {
            store::clear();
        }
        let saved = if options.demo || options.reset {
            None
        } else {
            store::load()
        };

        let inputs = Inputs {
            composer: input(cx, "Message", |i| i.multiline(9, true).sized(15., 24.)),
            search: input(cx, "Search", |i| i.sized(15., 22.)),
            bot_name: input(cx, "New Bot", |i| i.sized(16., 24.)),
            plugin_search: input(cx, "Search plugins", |i| i.sized(16., 24.)),
            routine_name: input(cx, "Routine name", |i| i.sized(15., 22.)),
            routine_instruction: input(cx, "What should happen each time it runs?", |i| {
                i.multiline(10, false).sized(15., 23.)
            }),
            profile_name: input(cx, "Name", |i| i.sized(15., 22.)),
            profile_title: input(cx, "Job title", |i| i.sized(15., 22.)),
            profile_description: input(cx, "What this Bot does, in a few sentences", |i| {
                i.multiline(8, false).sized(15., 22.)
            }),
            answer: input(cx, "Answer", |i| i.sized(15., 22.)),
        };

        let (backend, events) = backend::connect();
        let mut tasks = Vec::new();
        tasks.push(cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update(cx, |this, cx| this.on_backend(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        }));
        // Every few seconds one Bot blinks, so the roster feels alive.
        tasks.push(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(2300))
                    .await;
                let ok = this
                    .update(cx, |this, cx| {
                        this.blink = (
                            this.blink.0.wrapping_add(1),
                            Some(Instant::now() + Duration::from_millis(150)),
                        );
                        cx.notify();
                    })
                    .is_ok();
                if !ok {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(170))
                    .await;
                let _ = this.update(cx, |_, cx| cx.notify());
            }
        }));

        let mut view = AppView {
            focus: cx.focus_handle(),
            stage: Stage::SignIn,
            tools_picked: vec!["gmail", "gcal"],
            setup: Tween::new(0.0),
            profile: Profile {
                name: default_user_name(),
            },
            bots: Vec::new(),
            next_id: 100,
            page: Page::NewBot,
            draft_color: 6,
            draft_shape: Shape::Circle,
            draft_bump: 0,
            draft_role: None,
            inputs,
            editing_profile: false,
            panel: Presence::new(),
            modal: Presence::new(),
            popover: Presence::new(),
            suggest: None,
            plugins: data::plugins(),
            plugin_category: 0,
            connecting: Vec::new(),
            toasts: Vec::new(),
            voice: None,
            teach: None,
            settings: Settings::default(),
            settings_section: 0,
            backend,
            chat_scroll: ScrollHandle::new(),
            stick_bottom: true,
            hovered_msg: None,
            reply_to: None,
            attachments: Vec::new(),
            blink: (0, None),
            sidebar: Tween::new(1.0),
            cursor: (Tween::new(0.5), Tween::new(0.55)),
            cursor_bot: None,
            show_hidden: false,
            persist: !options.demo,
            bubble_max: 600.,
            vp: (1180., 780.),
            saved_theme: ThemeMode::System,
            search_cache: String::new(),
            _tasks: tasks,
        };

        if let Some(saved) = saved {
            view.apply_saved(saved);
        } else if options.demo {
            view.stage = Stage::Main;
            view.bots = data::demo_bots(&mut view.next_id);
            view.page = Page::Bot(view.bots[0].id);
        }
        let mode = view.theme_mode_from_saved();
        cx.global_mut::<ActiveTheme>().mode = mode;
        cx.global_mut::<ActiveTheme>().system_dark = theme::appearance_is_dark(window.appearance());
        cx.observe_window_appearance(window, |_, window, cx| {
            cx.global_mut::<ActiveTheme>().system_dark =
                theme::appearance_is_dark(window.appearance());
            cx.notify();
        })
        .detach();

        view.subscribe_inputs(window, cx);
        view.sync_composer_placeholder(cx);
        view
    }

    fn theme_mode_from_saved(&self) -> ThemeMode {
        std::env::var("SUZHOU_THEME")
            .ok()
            .and_then(|v| match v.as_str() {
                "dark" => Some(ThemeMode::Dark),
                "light" => Some(ThemeMode::Light),
                _ => None,
            })
            .unwrap_or(self.saved_theme)
    }

    fn apply_saved(&mut self, saved: Saved) {
        self.stage = if saved.signed_in {
            Stage::Main
        } else {
            Stage::SignIn
        };
        self.profile = saved.profile;
        if self.profile.name.is_empty() {
            self.profile.name = default_user_name();
        }
        self.bots = saved.bots;
        self.next_id = saved.next_id.max(100);
        self.saved_theme = saved.theme;
        self.settings = saved.settings;
        for plugin in &mut self.plugins {
            plugin.installed = saved.installed_plugins.iter().any(|id| id == plugin.id);
        }
        self.page = self
            .bots
            .iter()
            .find(|b| !b.hidden)
            .map(|b| Page::Bot(b.id))
            .unwrap_or(Page::NewBot);
    }

    pub fn save(&self, cx: &App) {
        if !self.persist {
            return;
        }
        let saved = Saved {
            signed_in: self.stage == Stage::Main,
            profile: self.profile.clone(),
            bots: self
                .bots
                .iter()
                .map(|b| {
                    let mut b = b.clone();
                    for m in &mut b.messages {
                        if let MessageKind::Bot { streaming, .. } = &mut m.kind {
                            *streaming = false;
                        }
                    }
                    b
                })
                .collect(),
            next_id: self.next_id,
            theme: cx.global::<ActiveTheme>().mode,
            installed_plugins: self
                .plugins
                .iter()
                .filter(|p| p.installed)
                .map(|p| p.id.to_string())
                .collect(),
            settings: self.settings.clone(),
        };
        store::save(&saved);
    }

    pub fn focus_initial(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match (self.stage, self.page) {
            (Stage::Main, Page::Bot(_)) => window.focus(&self.inputs.composer.focus_handle(cx)),
            (Stage::Main, Page::NewBot) => window.focus(&self.inputs.bot_name.focus_handle(cx)),
            _ => window.focus(&self.focus),
        }
    }

    fn subscribe_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let composer = self.inputs.composer.clone();
        cx.subscribe_in(
            &composer,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Changed => this.update_suggest(cx),
                InputEvent::Submit => this.send_message(window, cx),
                InputEvent::NavUp => this.move_suggest(-1, cx),
                InputEvent::NavDown => this.move_suggest(1, cx),
                InputEvent::Accept => this.accept_suggest(None, window, cx),
            },
        )
        .detach();

        let search = self.inputs.search.clone();
        cx.subscribe_in(
            &search,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Changed => cx.notify(),
                InputEvent::Submit => {
                    if let Some(id) = this.roster().first().copied() {
                        this.open_bot(id, window, cx);
                    }
                }
                _ => {}
            },
        )
        .detach();

        let bot_name = self.inputs.bot_name.clone();
        cx.subscribe_in(
            &bot_name,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Changed => cx.notify(),
                InputEvent::Submit => this.create_bot(window, cx),
                _ => {}
            },
        )
        .detach();

        let plugin_search = self.inputs.plugin_search.clone();
        cx.subscribe(&plugin_search, |_, _, _: &InputEvent, cx| cx.notify())
            .detach();

        let answer = self.inputs.answer.clone();
        cx.subscribe_in(
            &answer,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if *event == InputEvent::Submit {
                    this.answer_question(None, window, cx);
                }
            },
        )
        .detach();

        for (entity, field) in [
            (self.inputs.routine_name.clone(), Field::RoutineName),
            (
                self.inputs.routine_instruction.clone(),
                Field::RoutineInstruction,
            ),
            (self.inputs.profile_name.clone(), Field::ProfileName),
            (self.inputs.profile_title.clone(), Field::ProfileTitle),
            (
                self.inputs.profile_description.clone(),
                Field::ProfileDescription,
            ),
        ] {
            cx.subscribe(&entity, move |this, input, event: &InputEvent, cx| {
                if *event == InputEvent::Changed {
                    let text = input.read(cx).text().to_string();
                    this.write_field(field, text, cx);
                }
            })
            .detach();
        }
    }

    fn write_field(&mut self, field: Field, text: String, cx: &mut Context<Self>) {
        let routine = match self.panel.target {
            Some(Panel::Routine(id)) => Some(id),
            _ => None,
        };
        let Some(bot) = self.current_bot_mut() else {
            return;
        };
        match field {
            Field::RoutineName => {
                if let Some(r) = routine.and_then(|id| bot.routines.iter_mut().find(|r| r.id == id))
                {
                    r.name = text;
                }
            }
            Field::RoutineInstruction => {
                if let Some(r) = routine.and_then(|id| bot.routines.iter_mut().find(|r| r.id == id))
                {
                    r.instruction = text;
                }
            }
            Field::ProfileName => {
                if !text.trim().is_empty() {
                    bot.name = text;
                }
            }
            Field::ProfileTitle => bot.title = text,
            Field::ProfileDescription => bot.description = text,
        }
        self.sync_composer_placeholder(cx);
        self.save(cx);
        cx.notify();
    }

    // ----- Lookups -----------------------------------------------------------

    pub fn bot(&self, id: BotId) -> Option<&Bot> {
        self.bots.iter().find(|b| b.id == id)
    }

    pub fn bot_mut(&mut self, id: BotId) -> Option<&mut Bot> {
        self.bots.iter_mut().find(|b| b.id == id)
    }

    pub fn current_bot(&self) -> Option<&Bot> {
        match self.page {
            Page::Bot(id) => self.bot(id),
            Page::NewBot => None,
        }
    }

    pub fn current_bot_mut(&mut self) -> Option<&mut Bot> {
        match self.page {
            Page::Bot(id) => self.bot_mut(id),
            Page::NewBot => None,
        }
    }

    /// The Bots in the sidebar, in order: pinned first, then by recency.
    pub fn roster(&self) -> Vec<BotId> {
        let query = self.inputs_search_text();
        let mut list: Vec<&Bot> = self
            .bots
            .iter()
            .filter(|b| !b.hidden || self.show_hidden)
            .filter(|b| {
                query.is_empty()
                    || b.name.to_lowercase().contains(&query)
                    || b.title.to_lowercase().contains(&query)
                    || b.preview().to_lowercase().contains(&query)
            })
            .collect();
        list.sort_by_key(|b| !b.pinned);
        list.into_iter().map(|b| b.id).collect()
    }

    fn inputs_search_text(&self) -> String {
        self.search_cache.to_lowercase()
    }

    fn alloc_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn is_visible(&self, bot: BotId) -> bool {
        self.stage == Stage::Main && self.page == Page::Bot(bot)
    }

    pub fn eyes_closed(&self, index: usize) -> bool {
        let n = self.bots.len().max(1);
        self.blink.0 % n == index % n && self.blink.1.is_some_and(|until| Instant::now() < until)
    }

    fn persona(bot: &Bot) -> Persona {
        Persona {
            name: bot.name.clone(),
            title: bot.title.clone(),
            description: bot.description.clone(),
        }
    }

    fn push(&mut self, bot: BotId, kind: MessageKind) -> MsgId {
        let id = self.alloc_id();
        let time = match kind {
            MessageKind::Stamp(_) | MessageKind::Event { .. } | MessageKind::Activity { .. } => {
                String::new()
            }
            _ => data::now_time(),
        };
        if let Some(bot) = self.bot_mut(bot) {
            if let Some(MessageKind::Bot { streaming, .. }) =
                bot.messages.last_mut().map(|m| &mut m.kind)
            {
                *streaming = false;
            }
            bot.messages.push(Message {
                id,
                kind,
                time,
                reactions: Vec::new(),
                reply_to: None,
                fresh: true,
            });
        }
        id
    }

    pub fn sync_composer_placeholder(&mut self, cx: &mut Context<Self>) {
        let name = self
            .current_bot()
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "Bot".into());
        self.inputs.composer.update(cx, |input, cx| {
            input.set_placeholder(format!("Message {name}"), cx)
        });
    }

    // ----- Navigation --------------------------------------------------------

    pub fn open_bot(&mut self, id: BotId, window: &mut Window, cx: &mut Context<Self>) {
        if self.page != Page::Bot(id) {
            self.page = Page::Bot(id);
            self.stick_bottom = true;
            self.chat_scroll.scroll_to_bottom();
            self.reply_to = None;
            self.attachments.clear();
            self.suggest = None;
            self.editing_profile = false;
            if matches!(self.panel.target, Some(Panel::Routine(_))) {
                self.panel.close(200);
            }
            self.inputs.composer.update(cx, |i, cx| i.set_text("", cx));
        }
        if let Some(bot) = self.bot_mut(id) {
            bot.unread = false;
            let (x, y) = bot.computer.cursor;
            self.cursor.0.set(x);
            self.cursor.1.set(y);
            self.cursor_bot = Some(id);
        }
        self.toasts.retain(|t| t.bot != id);
        self.sync_composer_placeholder(cx);
        self.sync_panel_inputs(cx);
        window.focus(&self.inputs.composer.focus_handle(cx));
        cx.notify();
    }

    pub fn start_new_bot(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.stage != Stage::Main {
            return;
        }
        self.page = Page::NewBot;
        self.draft_color = (self.bots.len() * 3 + 6) % theme::BOT_COLORS.len();
        self.draft_shape = Shape::ALL[self.bots.len() % Shape::ALL.len()];
        self.draft_role = None;
        self.draft_bump += 1;
        self.panel.close(200);
        self.inputs.bot_name.update(cx, |i, cx| i.set_text("", cx));
        window.focus(&self.inputs.bot_name.focus_handle(cx));
        cx.notify();
    }

    pub fn create_bot(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.inputs.bot_name.read(cx).text().trim().to_string();
        if name.is_empty() {
            return;
        }
        let id = self.alloc_id();
        let mut bot = Bot::new(id, name.clone(), self.draft_color, self.draft_shape);
        if let Some((title, description)) = self.draft_role.take() {
            bot.title = title;
            bot.description = description;
        }
        bot.last_activity = data::now_time();
        self.bots.insert(0, bot);
        self.push(id, MessageKind::Stamp(data::today_stamp()));
        self.push(
            id,
            MessageKind::Event {
                text: format!("{name} joined your team"),
                icon: None,
                link: None,
                routine: None,
            },
        );
        self.open_bot(id, window, cx);
        self.save(cx);
    }

    pub fn use_suggestion(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let s = &data::SUGGESTIONS[ix];
        self.draft_color = s.color;
        self.draft_shape = s.shape;
        self.draft_bump += 1;
        self.draft_role = Some((s.title.into(), s.description.into()));
        self.inputs
            .bot_name
            .update(cx, |i, cx| i.set_text(s.name, cx));
        window.focus(&self.inputs.bot_name.focus_handle(cx));
        cx.notify();
    }

    // ----- Sending -----------------------------------------------------------

    pub fn send_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = window;
        let text = self.inputs.composer.read(cx).text().trim().to_string();
        let Page::Bot(bot_id) = self.page else { return };
        if text.is_empty() && self.attachments.is_empty() {
            return;
        }
        let attachments = std::mem::take(&mut self.attachments);
        let reply = self.reply_to.take();
        let id = self.push(
            bot_id,
            MessageKind::User {
                text: text.clone(),
                attachments: attachments.clone(),
            },
        );
        if let Some((_, quote)) = &reply
            && let Some(m) = self.bot_mut(bot_id).and_then(|b| b.find_message_mut(id))
        {
            m.reply_to = Some(quote.clone());
        }
        self.inputs.composer.update(cx, |i, cx| i.set_text("", cx));
        self.suggest = None;
        self.stick_bottom = true;
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let first = !bot.primed;
        bot.primed = true;
        bot.status = BotStatus::Working;
        let persona = Self::persona(bot);
        let mut prompt = text;
        if let Some((_, quote)) = reply {
            prompt = format!("> {}\n\n{prompt}", quote.replace('\n', "\n> "));
        }
        for file in attachments {
            prompt.push_str(&format!("\n\n[Attached: {file}]"));
        }
        self.backend.send(Request::Prompt {
            bot: bot_id,
            persona,
            first,
            text: prompt,
        });
        self.save(cx);
        cx.notify();
    }

    pub fn stop_bot(&mut self, cx: &mut Context<Self>) {
        if let Page::Bot(id) = self.page
            && self.bot(id).is_some_and(|b| b.status == BotStatus::Working)
        {
            self.backend.send(Request::Abort { bot: id });
            self.push(
                id,
                MessageKind::Event {
                    text: "Stopped".into(),
                    icon: Some("icons/stop.svg".into()),
                    link: None,
                    routine: None,
                },
            );
            cx.notify();
        }
    }

    pub fn answer_question(
        &mut self,
        question: Option<MsgId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.inputs.answer.read(cx).text().trim().to_string();
        let Page::Bot(bot_id) = self.page else { return };
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let target = question.or_else(|| {
            bot.messages
                .iter()
                .rev()
                .find(|m| matches!(m.kind, MessageKind::Question { answer: None, .. }))
                .map(|m| m.id)
        });
        let Some(qid) = target else { return };
        let Some(msg) = bot.find_message_mut(qid) else {
            return;
        };
        let MessageKind::Question {
            answer,
            placeholder,
            ..
        } = &mut msg.kind
        else {
            return;
        };
        let text = if text.is_empty() {
            placeholder.clone()
        } else {
            text
        };
        *answer = Some(text.clone());
        bot.status = BotStatus::Working;
        self.inputs.answer.update(cx, |i, cx| i.set_text("", cx));
        self.backend.send(Request::Answer {
            bot: bot_id,
            question: qid,
            text,
        });
        window.focus(&self.inputs.composer.focus_handle(cx));
        self.stick_bottom = true;
        self.save(cx);
        cx.notify();
    }

    pub fn decide(&mut self, msg: MsgId, approved: bool, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let Some(m) = bot.find_message_mut(msg) else {
            return;
        };
        if let MessageKind::Approval { state, .. } = &mut m.kind {
            if *state != ApprovalState::Pending {
                return;
            }
            *state = if approved {
                ApprovalState::Approved
            } else {
                ApprovalState::Denied
            };
        }
        bot.status = BotStatus::Working;
        self.backend.send(Request::Approval {
            bot: bot_id,
            request: msg,
            approved,
        });
        self.stick_bottom = true;
        self.save(cx);
        cx.notify();
    }

    // ----- Composer suggestions (@ and /) -----------------------------------

    fn update_suggest(&mut self, cx: &mut Context<Self>) {
        let (text, cursor) = {
            let input = self.inputs.composer.read(cx);
            (input.text().to_string(), input.cursor())
        };
        let before = &text[..cursor.min(text.len())];
        let token_start = before
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        let token = &before[token_start..];
        let next = if let Some(q) = token.strip_prefix('@') {
            Some(Suggest {
                kind: SuggestKind::Mention,
                start: token_start,
                query: q.to_lowercase(),
                ix: 0,
            })
        } else if let Some(q) = token.strip_prefix('/') {
            (token_start == 0 || before[..token_start].trim().is_empty()).then(|| Suggest {
                kind: SuggestKind::Skill,
                start: token_start,
                query: q.to_lowercase(),
                ix: 0,
            })
        } else {
            None
        };
        let next = next.filter(|s| !self.suggest_items(s).is_empty());
        if let (Some(old), Some(new)) = (&self.suggest, &next)
            && old.kind == new.kind
            && old.start == new.start
            && old.query == new.query
        {
            return;
        }
        self.suggest = next;
        let intercept = self.suggest.is_some();
        self.inputs
            .composer
            .update(cx, |i, _| i.intercept_nav = intercept);
        cx.notify();
    }

    pub fn suggest_items(&self, s: &Suggest) -> Vec<SuggestItem> {
        let q = &s.query;
        let matches = |label: &str| q.is_empty() || label.to_lowercase().contains(q.as_str());
        let mut items = Vec::new();
        match s.kind {
            SuggestKind::Mention => {
                for bot in self
                    .bots
                    .iter()
                    .filter(|b| Some(b.id) != self.current_bot().map(|c| c.id))
                {
                    if matches(&bot.name) {
                        items.push(SuggestItem {
                            label: bot.name.clone(),
                            detail: if bot.title.is_empty() {
                                "Bot".into()
                            } else {
                                bot.title.clone()
                            },
                            section: "Bots",
                            icon: SuggestIcon::Bot(bot.color, bot.shape),
                            insert: format!("@{} ", bot.name),
                        });
                    }
                }
                if let Some(bot) = self.current_bot() {
                    for r in &bot.routines {
                        if matches(&r.name) {
                            items.push(SuggestItem {
                                label: r.name.clone(),
                                detail: r.schedules.first().cloned().unwrap_or_default(),
                                section: "Routines",
                                icon: SuggestIcon::Path("icons/clock.svg"),
                                insert: format!("@{} ", r.name),
                            });
                        }
                    }
                }
                for p in self.plugins.iter().filter(|p| p.installed) {
                    if matches(p.name) {
                        items.push(SuggestItem {
                            label: p.name.into(),
                            detail: "Connector".into(),
                            section: "Connectors",
                            icon: SuggestIcon::Glyph(p.glyph, p.color),
                            insert: format!("@{} ", p.name),
                        });
                    }
                }
            }
            SuggestKind::Skill => {
                let mut skills: Vec<Skill> = self
                    .current_bot()
                    .map(|b| b.skills.clone())
                    .unwrap_or_default();
                for s in data::default_skills() {
                    if !skills.iter().any(|k| k.name == s.name) {
                        skills.push(s);
                    }
                }
                for skill in skills {
                    if matches(&skill.name) {
                        items.push(SuggestItem {
                            label: format!("/{}", skill.name),
                            detail: skill.description.clone(),
                            section: "Skills",
                            icon: SuggestIcon::Path("icons/book.svg"),
                            insert: format!("/{} ", skill.name),
                        });
                    }
                }
                if matches("teach") {
                    items.push(SuggestItem {
                        label: "Teach a task".into(),
                        detail: "Show the Bot how, once; it saves a skill".into(),
                        section: "Actions",
                        icon: SuggestIcon::Path("icons/sparkles.svg"),
                        insert: String::new(),
                    });
                }
            }
        }
        items.truncate(8);
        items
    }

    fn move_suggest(&mut self, delta: i32, cx: &mut Context<Self>) {
        let Some(s) = self.suggest.clone() else {
            return;
        };
        let n = self.suggest_items(&s).len() as i32;
        if n == 0 {
            return;
        }
        if let Some(s) = self.suggest.as_mut() {
            s.ix = ((s.ix as i32 + delta).rem_euclid(n)) as usize;
        }
        cx.notify();
    }

    pub fn accept_suggest(
        &mut self,
        ix: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(s) = self.suggest.take() else { return };
        let items = self.suggest_items(&s);
        let Some(item) = items.get(ix.unwrap_or(s.ix)).cloned() else {
            return;
        };
        self.inputs
            .composer
            .update(cx, |i, _| i.intercept_nav = false);
        let cursor = self.inputs.composer.read(cx).cursor();
        if item.insert.is_empty() {
            self.inputs
                .composer
                .update(cx, |i, cx| i.replace_range(s.start..cursor, "", cx));
            self.suggest = None;
            self.open_modal(Modal::Teach, cx);
            return;
        }
        self.inputs.composer.update(cx, |i, cx| {
            i.replace_range(s.start..cursor, &item.insert, cx)
        });
        self.suggest = None;
        window.focus(&self.inputs.composer.focus_handle(cx));
        cx.notify();
    }

    // ----- Overlays ----------------------------------------------------------

    pub fn open_modal(&mut self, modal: Modal, cx: &mut Context<Self>) {
        if modal == Modal::Teach {
            self.teach = Some(Instant::now());
        }
        self.popover.close(120);
        self.modal.open(modal, 320);
        cx.notify();
    }

    pub fn close_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.teach = None;
        self.modal.close(200);
        if self.stage == Stage::Main && matches!(self.page, Page::Bot(_)) {
            window.focus(&self.inputs.composer.focus_handle(cx));
        }
        cx.notify();
    }

    pub fn open_popover(&mut self, popover: Popover, cx: &mut Context<Self>) {
        if self.popover.is(&popover) {
            self.popover.close(140);
        } else {
            self.popover.close(0);
            self.popover.open(popover, 220);
        }
        cx.notify();
    }

    pub fn close_popover(&mut self, cx: &mut Context<Self>) {
        self.popover.close(140);
        cx.notify();
    }

    pub fn toggle_panel(&mut self, panel: Panel, cx: &mut Context<Self>) {
        if self.panel.is(&panel) {
            self.panel.close(260);
        } else {
            self.panel.open(panel, 380);
            self.sync_panel_inputs(cx);
        }
        cx.notify();
    }

    pub fn open_panel(&mut self, panel: Panel, cx: &mut Context<Self>) {
        self.panel.open(panel, 380);
        self.sync_panel_inputs(cx);
        cx.notify();
    }

    /// Loads the open panel's fields into its text inputs.
    fn sync_panel_inputs(&mut self, cx: &mut Context<Self>) {
        let Some(bot) = self.current_bot().cloned() else {
            return;
        };
        match self.panel.target {
            Some(Panel::Routine(id)) => {
                if let Some(r) = bot.routines.iter().find(|r| r.id == id) {
                    let (name, instruction) = (r.name.clone(), r.instruction.clone());
                    self.inputs
                        .routine_name
                        .update(cx, |i, cx| i.set_text(name, cx));
                    self.inputs
                        .routine_instruction
                        .update(cx, |i, cx| i.set_text(instruction, cx));
                }
            }
            Some(Panel::Details) => {
                self.inputs
                    .profile_name
                    .update(cx, |i, cx| i.set_text(bot.name.clone(), cx));
                self.inputs
                    .profile_title
                    .update(cx, |i, cx| i.set_text(bot.title.clone(), cx));
                self.inputs
                    .profile_description
                    .update(cx, |i, cx| i.set_text(bot.description.clone(), cx));
            }
            _ => {}
        }
    }

    pub fn toast(&mut self, bot: BotId, title: String, body: String, cx: &mut Context<Self>) {
        if !self.settings.notifications {
            return;
        }
        let id = self.alloc_id();
        let mut t = Tween::new(0.0);
        t.animate_to(1.0, 420, anim::ease_out_back);
        self.toasts.retain(|t| t.bot != bot);
        self.toasts.push(Toast {
            id,
            bot,
            title,
            body,
            t,
            leaving: false,
        });
        self.toasts.truncate(4);
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(5200))
                .await;
            let _ = this.update(cx, |this, cx| this.dismiss_toast(id, cx));
        })
        .detach();
        cx.notify();
    }

    pub fn dismiss_toast(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id && !t.leaving) {
            toast.leaving = true;
            toast.t.animate_to(0.0, 240, anim::ease_out_cubic);
            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(260))
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.toasts.retain(|t| t.id != id);
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    // ----- Bot management ----------------------------------------------------

    pub fn toggle_pin(&mut self, id: BotId, cx: &mut Context<Self>) {
        if let Some(b) = self.bot_mut(id) {
            b.pinned = !b.pinned;
        }
        self.close_popover(cx);
        self.save(cx);
    }

    pub fn toggle_hidden(&mut self, id: BotId, window: &mut Window, cx: &mut Context<Self>) {
        let mut now_hidden = false;
        if let Some(b) = self.bot_mut(id) {
            b.hidden = !b.hidden;
            now_hidden = b.hidden;
        }
        self.close_popover(cx);
        if now_hidden && self.page == Page::Bot(id) && !self.show_hidden {
            match self.roster().first().copied() {
                Some(next) => self.open_bot(next, window, cx),
                None => self.page = Page::NewBot,
            }
        }
        self.save(cx);
        cx.notify();
    }

    pub fn duplicate_bot(&mut self, id: BotId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.bot(id).cloned() else {
            return;
        };
        let new_id = self.alloc_id();
        let mut copy = Bot::new(
            new_id,
            format!("{} copy", source.name),
            source.color,
            source.shape,
        );
        copy.title = source.title.clone();
        copy.description = source.description.clone();
        copy.skills = source.skills.clone();
        for r in &source.routines {
            let rid = self.alloc_id();
            copy.routines.push(Routine {
                id: rid,
                history: Vec::new(),
                ..r.clone()
            });
        }
        let at = self
            .bots
            .iter()
            .position(|b| b.id == id)
            .map(|i| i + 1)
            .unwrap_or(0);
        self.bots.insert(at, copy);
        self.push(new_id, MessageKind::Stamp(data::today_stamp()));
        self.push(
            new_id,
            MessageKind::Event {
                text: format!("Duplicated from {}", source.name),
                icon: Some("icons/copy.svg".into()),
                link: None,
                routine: None,
            },
        );
        self.close_popover(cx);
        self.open_bot(new_id, window, cx);
        self.save(cx);
    }

    pub fn delete_bot(&mut self, id: BotId, window: &mut Window, cx: &mut Context<Self>) {
        self.backend.send(Request::Forget { bot: id });
        self.bots.retain(|b| b.id != id);
        self.toasts.retain(|t| t.bot != id);
        self.close_modal(window, cx);
        self.panel.close(200);
        if self.page == Page::Bot(id) {
            match self.roster().first().copied() {
                Some(next) => self.open_bot(next, window, cx),
                None => self.start_new_bot(window, cx),
            }
        }
        self.save(cx);
        cx.notify();
    }

    pub fn new_routine(&mut self, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        let rid = self.alloc_id();
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        if bot.routines.len() >= 50 {
            return;
        }
        let name = format!("Routine {}", bot.routines.len() + 1);
        bot.routines.push(Routine {
            id: rid,
            name: name.clone(),
            instruction: String::new(),
            schedules: vec!["Every day at 9:00 AM".into()],
            active: true,
            history: Vec::new(),
        });
        self.push(
            bot_id,
            MessageKind::Event {
                text: "Created routine".into(),
                icon: Some("icons/clock.svg".into()),
                link: Some(name),
                routine: Some(rid),
            },
        );
        self.open_panel(Panel::Routine(rid), cx);
        self.save(cx);
    }

    pub fn routine_mut(&mut self, rid: RoutineId) -> Option<&mut Routine> {
        self.current_bot_mut()?
            .routines
            .iter_mut()
            .find(|r| r.id == rid)
    }

    pub fn test_run(&mut self, rid: RoutineId, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let persona = Self::persona(bot);
        let Some(r) = bot.routines.iter_mut().find(|r| r.id == rid) else {
            return;
        };
        if r.history
            .first()
            .is_some_and(|h| h.status == RunStatus::Running)
        {
            return;
        }
        r.history.insert(
            0,
            RunRecord {
                when: data::now_stamp_full(),
                status: RunStatus::Running,
            },
        );
        r.history.truncate(20);
        let instruction = if r.instruction.trim().is_empty() {
            r.name.clone()
        } else {
            r.instruction.clone()
        };
        let name = r.name.clone();
        bot.status = BotStatus::Working;
        self.push(
            bot_id,
            MessageKind::Event {
                text: "Test run".into(),
                icon: Some("icons/play.svg".into()),
                link: Some(name),
                routine: Some(rid),
            },
        );
        self.backend.send(Request::TestRun {
            bot: bot_id,
            routine: rid,
            persona,
            instruction,
        });
        self.stick_bottom = true;
        cx.notify();
    }

    pub fn delete_routine(&mut self, rid: RoutineId, cx: &mut Context<Self>) {
        if let Some(bot) = self.current_bot_mut() {
            bot.routines.retain(|r| r.id != rid);
        }
        self.panel.close(240);
        self.save(cx);
        cx.notify();
    }

    pub fn install_plugin(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if self.connecting.contains(&id) {
            return;
        }
        self.connecting.push(id);
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.connecting.retain(|c| *c != id);
                if let Some(p) = this.plugins.iter_mut().find(|p| p.id == id) {
                    p.installed = true;
                }
                this.save(cx);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn uninstall_plugin(&mut self, id: &'static str, cx: &mut Context<Self>) {
        if let Some(p) = self.plugins.iter_mut().find(|p| p.id == id) {
            p.installed = false;
        }
        self.save(cx);
        cx.notify();
    }

    pub fn finish_teach(&mut self, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else {
            self.teach = None;
            return;
        };
        let elapsed = self.teach.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let name = format!("taught-task-{}", bot.skills.len() + 1);
        bot.skills.push(Skill {
            name: name.clone(),
            description: format!("Shown by demonstration ({}s)", elapsed.max(1)),
        });
        self.teach = None;
        self.modal.close(200);
        self.push(
            bot_id,
            MessageKind::Event {
                text: "Saved skill".into(),
                icon: Some("icons/book.svg".into()),
                link: Some(name),
                routine: None,
            },
        );
        self.save(cx);
        cx.notify();
    }

    pub fn save_as_skill(&mut self, msg: MsgId, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        let Some(bot) = self.bot_mut(bot_id) else {
            return;
        };
        let Some(text) = bot
            .messages
            .iter()
            .find(|m| m.id == msg)
            .and_then(|m| match &m.kind {
                MessageKind::User { text, .. } | MessageKind::Bot { text, .. } => {
                    Some(text.clone())
                }
                _ => None,
            })
        else {
            return;
        };
        let name = text
            .split_whitespace()
            .take(3)
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let name = if name.is_empty() {
            "saved-process".into()
        } else {
            name
        };
        bot.skills.push(Skill {
            name: name.clone(),
            description: text
                .lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(90)
                .collect(),
        });
        self.push(
            bot_id,
            MessageKind::Event {
                text: "Saved the process as skill".into(),
                icon: Some("icons/book.svg".into()),
                link: Some(name),
                routine: None,
            },
        );
        self.close_popover(cx);
        self.save(cx);
    }

    pub fn toggle_reaction(&mut self, msg: MsgId, reaction: Reaction, cx: &mut Context<Self>) {
        if let Some(m) = self.current_bot_mut().and_then(|b| b.find_message_mut(msg)) {
            if let Some(at) = m.reactions.iter().position(|r| *r == reaction) {
                m.reactions.remove(at);
            } else {
                m.reactions.push(reaction);
            }
        }
        self.close_popover(cx);
        self.save(cx);
    }

    pub fn delete_message(&mut self, msg: MsgId, cx: &mut Context<Self>) {
        if let Some(bot) = self.current_bot_mut() {
            bot.messages.retain(|m| m.id != msg);
        }
        self.close_popover(cx);
        self.save(cx);
    }

    pub fn hand_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        self.close_modal(window, cx);
        let name = self.bot(bot_id).map(|b| b.name.clone()).unwrap_or_default();
        self.push(
            bot_id,
            MessageKind::Event {
                text: format!("You handed the computer back to {name}"),
                icon: Some("icons/hand.svg".into()),
                link: None,
                routine: None,
            },
        );
        if let Some(b) = self.bot_mut(bot_id) {
            b.status = BotStatus::Working;
        }
        self.backend.send(Request::HandBack { bot: bot_id });
        self.stick_bottom = true;
        cx.notify();
    }

    pub fn take_over(&mut self, cx: &mut Context<Self>) {
        let Page::Bot(bot_id) = self.page else { return };
        if self
            .bot(bot_id)
            .is_some_and(|b| b.status == BotStatus::Working)
        {
            self.backend.send(Request::Abort { bot: bot_id });
        }
        let name = self.bot(bot_id).map(|b| b.name.clone()).unwrap_or_default();
        self.push(
            bot_id,
            MessageKind::Event {
                text: format!("You took over {name}'s computer"),
                icon: Some("icons/hand.svg".into()),
                link: None,
                routine: None,
            },
        );
        self.open_modal(Modal::Takeover, cx);
    }

    pub fn toggle_voice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.page == Page::NewBot || self.stage != Stage::Main {
            return;
        }
        if let Some(started) = self.voice.take() {
            let secs = started.elapsed().as_secs_f32();
            let heard = if secs < 1.0 {
                String::new()
            } else {
                "Can you check today's numbers and tell me what changed since yesterday?"
                    .to_string()
            };
            if !heard.is_empty() {
                self.inputs.composer.update(cx, |i, cx| {
                    let mut text = i.text().to_string();
                    if !text.is_empty() && !text.ends_with(' ') {
                        text.push(' ');
                    }
                    text.push_str(&heard);
                    i.set_text(text, cx);
                });
            }
            window.focus(&self.inputs.composer.focus_handle(cx));
        } else {
            self.voice = Some(Instant::now());
        }
        cx.notify();
    }

    pub fn cancel_voice(&mut self, cx: &mut Context<Self>) {
        self.voice = None;
        cx.notify();
    }

    pub fn download_file(&mut self, name: &str, content: &str, cx: &mut Context<Self>) {
        let dir = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join("Downloads"))
            .unwrap_or_else(|_| store::data_dir());
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        let result = std::fs::write(&path, content);
        if let Page::Bot(bot_id) = self.page {
            let text = match result {
                Ok(()) => format!("Saved {name} to {}", dir.display()),
                Err(err) => format!("Couldn't save {name}: {err}"),
            };
            self.push(
                bot_id,
                MessageKind::Event {
                    text,
                    icon: Some("icons/download.svg".into()),
                    link: None,
                    routine: None,
                },
            );
        }
        cx.notify();
    }

    pub fn sign_out(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_popover(cx);
        self.modal.close(0);
        self.panel.close(0);
        self.stage = Stage::SignIn;
        self.save(cx);
        window.focus(&self.focus);
        cx.notify();
    }

    pub fn reset_computer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for bot in &mut self.bots {
            bot.computer = ComputerState::default();
        }
        self.close_modal(window, cx);
    }

    // ----- Onboarding --------------------------------------------------------

    pub fn go(&mut self, stage: Stage, window: &mut Window, cx: &mut Context<Self>) {
        self.stage = stage;
        match stage {
            Stage::Authorizing => {
                cx.spawn(async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(2200))
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        if this.stage == Stage::Authorizing {
                            this.stage = Stage::Tools;
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            Stage::SettingUp => {
                for id in self.tools_picked.clone() {
                    if let Some(p) = self.plugins.iter_mut().find(|p| p.id == id) {
                        p.installed = true;
                    }
                }
                self.setup.set(0.0);
                self.setup.animate_to(1.0, 3400, anim::ease_in_out_cubic);
                cx.spawn(async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(3600))
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        if this.stage == Stage::SettingUp {
                            this.stage = Stage::Meet;
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            Stage::Main => {
                if self.bots.is_empty() {
                    self.start_new_bot(window, cx);
                } else {
                    self.focus_initial(window, cx);
                }
                self.save(cx);
            }
            _ => {}
        }
        cx.notify();
    }

    // ----- Backend events ----------------------------------------------------

    fn on_backend(&mut self, event: Event, cx: &mut Context<Self>) {
        let bot_id = event.bot();
        if self.bot(bot_id).is_none() {
            return;
        }
        let visible = self.is_visible(bot_id);
        let name = self.bot(bot_id).map(|b| b.name.clone()).unwrap_or_default();
        match event {
            Event::Started { .. } => {
                if let Some(b) = self.bot_mut(bot_id) {
                    b.status = BotStatus::Working;
                }
            }
            Event::Delta { text, .. } => {
                let bot = self.bot_mut(bot_id).unwrap();
                match bot.messages.last_mut() {
                    Some(Message {
                        kind:
                            MessageKind::Bot {
                                text: existing,
                                streaming: true,
                            },
                        ..
                    }) => existing.push_str(&text),
                    _ => {
                        self.push(
                            bot_id,
                            MessageKind::Bot {
                                text,
                                streaming: true,
                            },
                        );
                    }
                }
            }
            Event::Activity { label, done, .. } => {
                let bot = self.bot_mut(bot_id).unwrap();
                let existing = bot.messages.iter_mut().rev().take(4).find(|m| {
                    matches!(&m.kind, MessageKind::Activity { label: l, done: false } if *l == label)
                });
                match existing {
                    Some(m) => m.kind = MessageKind::Activity { label, done },
                    None => {
                        self.push(bot_id, MessageKind::Activity { label, done });
                    }
                }
            }
            Event::Question {
                prompt,
                placeholder,
                ..
            } => {
                self.push(
                    bot_id,
                    MessageKind::Question {
                        prompt: prompt.clone(),
                        placeholder,
                        answer: None,
                    },
                );
                if let Some(b) = self.bot_mut(bot_id) {
                    b.status = BotStatus::NeedsYou;
                }
                if !visible {
                    self.toast(bot_id, format!("{name} has a question"), prompt, cx);
                }
            }
            Event::ApprovalNeeded { action, detail, .. } => {
                self.push(
                    bot_id,
                    MessageKind::Approval {
                        action: action.clone(),
                        detail,
                        state: ApprovalState::Pending,
                    },
                );
                if let Some(b) = self.bot_mut(bot_id) {
                    b.status = BotStatus::NeedsYou;
                }
                if !visible {
                    self.toast(bot_id, format!("{name} needs approval"), action, cx);
                }
            }
            Event::File {
                name: file,
                size,
                content,
                ..
            } => {
                self.push(
                    bot_id,
                    MessageKind::File {
                        name: file,
                        size,
                        content,
                    },
                );
            }
            Event::Note { text, .. } => {
                self.push(
                    bot_id,
                    MessageKind::Event {
                        text,
                        icon: Some("icons/info.svg".into()),
                        link: None,
                        routine: None,
                    },
                );
            }
            Event::Computer {
                url,
                title,
                status,
                cursor,
                typing,
                ..
            } => {
                if let Some(b) = self.bot_mut(bot_id) {
                    b.computer = ComputerState {
                        url,
                        page_title: title,
                        status,
                        cursor,
                        typing,
                        active: true,
                    };
                }
                if self.cursor_bot == Some(bot_id) || visible {
                    self.cursor_bot = Some(bot_id);
                    self.cursor
                        .0
                        .animate_to(cursor.0, 750, anim::ease_in_out_cubic);
                    self.cursor
                        .1
                        .animate_to(cursor.1, 750, anim::ease_in_out_cubic);
                }
            }
            Event::RoutineRun { routine, ok, .. } => {
                if let Some(r) = self
                    .bot_mut(bot_id)
                    .and_then(|b| b.routines.iter_mut().find(|r| r.id == routine))
                    && let Some(h) = r
                        .history
                        .iter_mut()
                        .find(|h| h.status == RunStatus::Running)
                {
                    h.status = if ok {
                        RunStatus::Succeeded
                    } else {
                        RunStatus::Failed
                    };
                }
            }
            Event::Error { message, .. } => {
                if let Some(b) = self.bot_mut(bot_id)
                    && b.status == BotStatus::Working
                {
                    b.status = BotStatus::Idle;
                }
                self.push(
                    bot_id,
                    MessageKind::Event {
                        text: message,
                        icon: Some("icons/info.svg".into()),
                        link: None,
                        routine: None,
                    },
                );
            }
            Event::Finished { .. } => {
                let bot = self.bot_mut(bot_id).unwrap();
                if let Some(MessageKind::Bot { streaming, .. }) =
                    bot.messages.last_mut().map(|m| &mut m.kind)
                {
                    *streaming = false;
                }
                for m in bot.messages.iter_mut() {
                    if let MessageKind::Activity { done, .. } = &mut m.kind {
                        *done = true;
                    }
                }
                let waiting = bot.messages.iter().any(|m| {
                    matches!(
                        m.kind,
                        MessageKind::Question { answer: None, .. }
                            | MessageKind::Approval {
                                state: ApprovalState::Pending,
                                ..
                            }
                    )
                });
                let was_working = bot.status == BotStatus::Working;
                bot.status = if waiting {
                    BotStatus::NeedsYou
                } else {
                    BotStatus::Idle
                };
                bot.computer.status = "Idle".into();
                if !visible {
                    bot.unread = true;
                    if was_working && !waiting && !self.settings.only_when_needed {
                        let preview: String = self
                            .bot(bot_id)
                            .map(|b| b.preview())
                            .unwrap_or_default()
                            .chars()
                            .take(90)
                            .collect();
                        self.toast(bot_id, format!("{name} finished"), preview, cx);
                    }
                }
                self.save(cx);
            }
        }
        cx.notify();
    }

    // ----- Actions -----------------------------------------------------------

    fn on_new_bot(&mut self, _: &NewBot, window: &mut Window, cx: &mut Context<Self>) {
        self.start_new_bot(window, cx);
    }

    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        if self.stage == Stage::Main {
            if self.sidebar.target() < 0.5 {
                self.sidebar.animate_to(1.0, 320, anim::ease_out_quint);
            }
            window.focus(&self.inputs.search.focus_handle(cx));
        }
    }

    fn on_toggle_voice(&mut self, _: &ToggleVoice, window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_voice(window, cx);
    }

    fn on_toggle_theme(&mut self, _: &ToggleTheme, _: &mut Window, cx: &mut Context<Self>) {
        let dark = cx.global::<ActiveTheme>().is_dark();
        cx.global_mut::<ActiveTheme>().mode = if dark {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        self.save(cx);
        cx.notify();
    }

    fn on_open_settings(&mut self, _: &OpenSettings, _: &mut Window, cx: &mut Context<Self>) {
        if self.stage == Stage::Main {
            self.open_modal(Modal::Settings, cx);
        }
    }

    fn on_open_plugins(&mut self, _: &OpenPlugins, _: &mut Window, cx: &mut Context<Self>) {
        if self.stage == Stage::Main {
            self.open_modal(Modal::Plugins, cx);
        }
    }

    fn on_toggle_computer(&mut self, _: &ToggleComputer, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.page, Page::Bot(_)) && self.stage == Stage::Main {
            self.toggle_panel(Panel::Computer, cx);
        }
    }

    fn on_toggle_details(&mut self, _: &ToggleDetails, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.page, Page::Bot(_)) && self.stage == Stage::Main {
            self.toggle_panel(Panel::Details, cx);
        }
    }

    fn on_toggle_sidebar(&mut self, _: &ToggleSidebar, _: &mut Window, cx: &mut Context<Self>) {
        let target = if self.sidebar.target() > 0.5 {
            0.0
        } else {
            1.0
        };
        self.sidebar.animate_to(target, 340, anim::ease_out_quint);
        cx.notify();
    }

    fn on_stop(&mut self, _: &StopBot, _: &mut Window, cx: &mut Context<Self>) {
        self.stop_bot(cx);
    }

    fn on_dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.popover.target.is_some() {
            self.close_popover(cx);
        } else if self.suggest.is_some() {
            self.suggest = None;
            self.inputs
                .composer
                .update(cx, |i, _| i.intercept_nav = false);
            cx.notify();
        } else if self.voice.is_some() {
            self.cancel_voice(cx);
        } else if self.modal.target.is_some() {
            if self.modal.target == Some(Modal::Takeover) {
                self.hand_back(window, cx);
            } else {
                self.close_modal(window, cx);
            }
        } else if self.reply_to.is_some() {
            self.reply_to = None;
            cx.notify();
        } else if self.panel.target.is_some() {
            self.panel.close(260);
            cx.notify();
        } else if self.stage == Stage::Authorizing {
            self.go(Stage::SignIn, window, cx);
        }
    }
}

#[derive(Clone, Copy)]
enum Field {
    RoutineName,
    RoutineInstruction,
    ProfileName,
    ProfileTitle,
    ProfileDescription,
}

fn default_user_name() -> String {
    std::env::var("SUZHOU_USER")
        .or_else(|_| std::env::var("USER"))
        .ok()
        .filter(|n| !n.is_empty() && n != "root")
        .map(|n| {
            let mut c = n.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| "You".into())
}

impl Focusable for AppView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.search_cache = self.inputs.search.read(cx).text().to_string();
        let viewport = window.viewport_size();
        self.vp = (f32::from(viewport.width), f32::from(viewport.height));
        let t = theme(cx);
        let mut root = div()
            .id("root")
            .key_context("App")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::on_new_bot))
            .on_action(cx.listener(Self::on_focus_search))
            .on_action(cx.listener(Self::on_toggle_voice))
            .on_action(cx.listener(Self::on_toggle_theme))
            .on_action(cx.listener(Self::on_open_settings))
            .on_action(cx.listener(Self::on_open_plugins))
            .on_action(cx.listener(Self::on_toggle_computer))
            .on_action(cx.listener(Self::on_toggle_details))
            .on_action(cx.listener(Self::on_toggle_sidebar))
            .on_action(cx.listener(Self::on_stop))
            .on_action(cx.listener(Self::on_dismiss))
            .relative()
            .size_full()
            .flex()
            .overflow_hidden()
            .bg(t.bg)
            .font_family(FONT)
            .text_color(t.text)
            .text_size(px(15.));

        if self.stage == Stage::Main {
            root = root
                .child(self.render_sidebar(window, cx))
                .child(self.render_main(window, cx))
                .child(self.render_panel(window, cx));
        } else {
            root = root.child(self.render_onboarding(window, cx));
        }
        root.child(self.render_modal(window, cx))
            .child(self.render_popover(window, cx))
            .child(self.render_toasts(window, cx))
    }
}
