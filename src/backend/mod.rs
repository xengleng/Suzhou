//! The seam between the interface and whatever does the work.
//!
//! The UI only ever sends [`Request`]s and reacts to [`Event`]s. Two
//! backends implement that today:
//!
//! - [`mock`]: a scripted Bot that streams replies, uses the Agent Computer,
//!   asks questions and requests approvals, so every screen can be exercised
//!   without a model.
//! - [`pi`]: drives the Pi coding agent (`pi --mode rpc`) over its JSON-lines
//!   protocol, one process per Bot.
//!
//! Choose with `SUZHOU_BACKEND=mock|pi` (default: mock).

pub mod mock;
pub mod pi;

use crate::model::{BotId, MsgId, RoutineId};

/// Who a Bot is, sent along with its first prompt so the agent can play the role.
#[derive(Clone, Debug)]
pub struct Persona {
    pub name: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub enum Request {
    Prompt {
        bot: BotId,
        persona: Persona,
        /// True the first time this Bot's session hears from us.
        first: bool,
        text: String,
    },
    /// Stop whatever the Bot is doing.
    Abort { bot: BotId },
    /// The answer to a question the Bot asked. `question` is the message it
    /// answers, for backends that track more than one open question.
    #[allow(dead_code)]
    Answer {
        bot: BotId,
        question: MsgId,
        text: String,
    },
    #[allow(dead_code)]
    Approval {
        bot: BotId,
        request: MsgId,
        approved: bool,
    },
    /// You took the Agent Computer and gave it back.
    HandBack { bot: BotId },
    TestRun {
        bot: BotId,
        routine: RoutineId,
        persona: Persona,
        instruction: String,
    },
    /// The Bot is gone; free its resources.
    Forget { bot: BotId },
}

#[derive(Clone, Debug)]
pub enum Event {
    /// The Bot started working on a turn.
    Started {
        bot: BotId,
    },
    /// More reply text. A new bubble starts after any non-text event.
    Delta {
        bot: BotId,
        text: String,
    },
    /// A tool call or step: "Opening flaviocopes.com".
    Activity {
        bot: BotId,
        label: String,
        done: bool,
    },
    Question {
        bot: BotId,
        prompt: String,
        placeholder: String,
    },
    ApprovalNeeded {
        bot: BotId,
        action: String,
        detail: String,
    },
    File {
        bot: BotId,
        name: String,
        size: String,
        content: String,
    },
    /// A grey line in the conversation.
    Note {
        bot: BotId,
        text: String,
    },
    /// What the Agent Computer is showing.
    Computer {
        bot: BotId,
        url: String,
        title: String,
        status: String,
        cursor: (f32, f32),
        typing: Option<String>,
    },
    RoutineRun {
        bot: BotId,
        routine: RoutineId,
        ok: bool,
    },
    Finished {
        bot: BotId,
    },
    Error {
        bot: BotId,
        message: String,
    },
}

impl Event {
    pub fn bot(&self) -> BotId {
        match self {
            Event::Started { bot }
            | Event::Delta { bot, .. }
            | Event::Activity { bot, .. }
            | Event::Question { bot, .. }
            | Event::ApprovalNeeded { bot, .. }
            | Event::File { bot, .. }
            | Event::Note { bot, .. }
            | Event::Computer { bot, .. }
            | Event::RoutineRun { bot, .. }
            | Event::Finished { bot }
            | Event::Error { bot, .. } => *bot,
        }
    }
}

pub trait Backend {
    fn send(&self, request: Request);
    fn name(&self) -> &'static str;
}

pub type Events = async_channel::Receiver<Event>;

/// Picks the backend from the environment.
pub fn connect() -> (Box<dyn Backend>, Events) {
    let (tx, rx) = async_channel::unbounded();
    let choice = std::env::var("SUZHOU_BACKEND").unwrap_or_default();
    let backend: Box<dyn Backend> = match choice.as_str() {
        "pi" => Box::new(pi::PiBackend::new(tx)),
        _ => Box::new(mock::MockBackend::new(tx)),
    };
    (backend, rx)
}

impl Persona {
    /// The preamble sent before a Bot's first message.
    pub fn preamble(&self) -> String {
        let mut out = format!("You are {}", self.name);
        if !self.title.is_empty() {
            out.push_str(&format!(", {}", self.title));
        }
        out.push_str(
            ". You are one of the user's always-on Bots: a teammate they give real work to.",
        );
        if !self.description.is_empty() {
            out.push_str(&format!("\n\nYour job: {}", self.description));
        }
        out.push_str(
            "\n\nReply in short, plain chat messages. Ask before anything that sends, publishes, \
             deletes or spends money.",
        );
        out
    }
}
