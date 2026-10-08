//! What the app knows about: Bots, their conversations, routines and skills.

use serde::{Deserialize, Serialize};

pub type BotId = u64;
pub type MsgId = u64;
pub type RoutineId = u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Shape {
    #[default]
    Circle,
    Blob,
    Square,
    Pill,
    Triangle,
    Hexagon,
    Cloud,
    Drop,
}

impl Shape {
    pub const ALL: [Shape; 8] = [
        Shape::Circle,
        Shape::Blob,
        Shape::Square,
        Shape::Pill,
        Shape::Triangle,
        Shape::Hexagon,
        Shape::Cloud,
        Shape::Drop,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Shape::Circle => "circle",
            Shape::Blob => "blob",
            Shape::Square => "square",
            Shape::Pill => "pill",
            Shape::Triangle => "triangle",
            Shape::Hexagon => "hexagon",
            Shape::Cloud => "cloud",
            Shape::Drop => "drop",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum BotStatus {
    #[default]
    Idle,
    /// Thinking, using tools or the computer.
    Working,
    /// Waiting on an answer or an approval from you.
    NeedsYou,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bot {
    pub id: BotId,
    pub name: String,
    pub title: String,
    pub description: String,
    pub color: usize,
    pub shape: Shape,
    pub pinned: bool,
    pub hidden: bool,
    #[serde(skip)]
    pub status: BotStatus,
    pub unread: bool,
    pub messages: Vec<Message>,
    pub routines: Vec<Routine>,
    pub skills: Vec<Skill>,
    /// Shown in the roster when the conversation is empty.
    pub last_activity: String,
    #[serde(skip)]
    pub computer: ComputerState,
    /// Whether the backend session has been told who this Bot is yet.
    #[serde(skip)]
    pub primed: bool,
}

impl Bot {
    pub fn new(id: BotId, name: impl Into<String>, color: usize, shape: Shape) -> Self {
        Bot {
            id,
            name: name.into(),
            title: String::new(),
            description: String::new(),
            color,
            shape,
            pinned: false,
            hidden: false,
            status: BotStatus::Idle,
            unread: false,
            messages: Vec::new(),
            routines: Vec::new(),
            skills: Vec::new(),
            last_activity: String::new(),
            computer: ComputerState::default(),
            primed: false,
        }
    }

    /// The line under the name in the roster.
    pub fn preview(&self) -> String {
        self.messages
            .iter()
            .rev()
            .find_map(|m| match &m.kind {
                MessageKind::Bot { text, .. } if !text.is_empty() => Some(text.clone()),
                MessageKind::User { text, .. } => Some(format!("You: {text}")),
                MessageKind::Question { prompt, .. } => Some(prompt.clone()),
                MessageKind::Approval { action, .. } => Some(format!("Needs approval: {action}")),
                MessageKind::File { name, .. } => Some(format!("Sent {name}")),
                _ => None,
            })
            .map(|s| s.replace('\n', " "))
            .unwrap_or_default()
    }

    pub fn last_time(&self) -> String {
        self.messages
            .iter()
            .rev()
            .find(|m| !m.time.is_empty())
            .map(|m| m.time.clone())
            .unwrap_or_else(|| self.last_activity.clone())
    }

    pub fn find_message_mut(&mut self, id: MsgId) -> Option<&mut Message> {
        self.messages.iter_mut().find(|m| m.id == id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: MsgId,
    pub kind: MessageKind,
    /// Display time, e.g. "10:02 AM".
    pub time: String,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub reply_to: Option<String>,
    /// Plays the arrival animation the first time it is drawn.
    #[serde(skip)]
    pub fresh: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Reaction {
    ThumbsUp,
    Heart,
    Check,
    Party,
}

impl Reaction {
    pub const ALL: [Reaction; 4] = [
        Reaction::ThumbsUp,
        Reaction::Heart,
        Reaction::Check,
        Reaction::Party,
    ];

    pub fn icon(self) -> &'static str {
        match self {
            Reaction::ThumbsUp => "icons/thumbs-up.svg",
            Reaction::Heart => "icons/heart.svg",
            Reaction::Check => "icons/check.svg",
            Reaction::Party => "icons/party.svg",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ApprovalState {
    Pending,
    Approved,
    Denied,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageKind {
    User {
        text: String,
        #[serde(default)]
        attachments: Vec<String>,
    },
    Bot {
        text: String,
        streaming: bool,
    },
    /// A centred grey line: "Renamed to Blog Pulse", "Created routine ...".
    Event {
        text: String,
        icon: Option<String>,
        /// Clickable label after the icon, e.g. a routine's name.
        link: Option<String>,
        routine: Option<RoutineId>,
    },
    /// A centred date line: "Today 10:01 AM".
    Stamp(String),
    /// What the Bot is doing right now: "Opening flaviocopes.com".
    Activity {
        label: String,
        done: bool,
    },
    File {
        name: String,
        size: String,
        content: String,
    },
    Question {
        prompt: String,
        placeholder: String,
        answer: Option<String>,
    },
    Approval {
        action: String,
        detail: String,
        state: ApprovalState,
    },
    /// A picture of the Agent Computer's screen at one moment.
    Snapshot {
        title: String,
        url: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Routine {
    pub id: RoutineId,
    pub name: String,
    pub instruction: String,
    pub schedules: Vec<String>,
    pub active: bool,
    pub history: Vec<RunRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub when: String,
    pub status: RunStatus,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
}

/// The Agent Computer as the side panel draws it.
#[derive(Clone, Debug)]
pub struct ComputerState {
    pub url: String,
    pub page_title: String,
    pub status: String,
    /// Where the pointer is heading, as fractions of the screen.
    pub cursor: (f32, f32),
    pub typing: Option<String>,
    pub active: bool,
}

impl Default for ComputerState {
    fn default() -> Self {
        ComputerState {
            url: "about:blank".into(),
            page_title: "New Tab".into(),
            status: "Idle".into(),
            cursor: (0.5, 0.55),
            typing: None,
            active: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub categories: &'static [&'static str],
    pub glyph: &'static str,
    pub color: u32,
    pub featured: bool,
    pub team: bool,
    pub installed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Profile {
    pub name: String,
}
