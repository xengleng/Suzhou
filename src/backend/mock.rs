//! A pretend Bot, so the whole interface works before a real agent is wired in.
//!
//! Each request runs on its own thread and plays a small script with
//! human-ish pauses: an acknowledgement, a few steps (sometimes on the Agent
//! Computer), then a streamed answer. Some words in the prompt steer the
//! script: "approve", "send" or "publish" asks for approval; "?" or "which"
//! asks a question back; "site", "browse", "check" or "computer" drives the
//! computer; "report" or "file" sends a file.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use async_channel::Sender;

use super::{Backend, Event, Request};
use crate::model::BotId;

pub struct MockBackend {
    tx: Sender<Event>,
    /// Bumped on abort so a running script notices and stops.
    generations: Arc<Mutex<HashMap<BotId, Arc<AtomicU64>>>>,
}

impl MockBackend {
    pub fn new(tx: Sender<Event>) -> Self {
        MockBackend {
            tx,
            generations: Arc::default(),
        }
    }

    fn generation(&self, bot: BotId) -> Arc<AtomicU64> {
        self.generations
            .lock()
            .unwrap()
            .entry(bot)
            .or_default()
            .clone()
    }

    fn run(&self, bot: BotId, body: impl FnOnce(&Script) + Send + 'static) {
        let counter = self.generation(bot);
        let generation = counter.fetch_add(1, Ordering::SeqCst) + 1;
        let script = Script {
            bot,
            tx: self.tx.clone(),
            counter,
            generation,
        };
        thread::spawn(move || {
            script.emit(Event::Started { bot });
            body(&script);
            script.emit(Event::Finished { bot });
        });
    }
}

struct Script {
    bot: BotId,
    tx: Sender<Event>,
    counter: Arc<AtomicU64>,
    generation: u64,
}

impl Script {
    fn alive(&self) -> bool {
        self.counter.load(Ordering::SeqCst) == self.generation
    }

    fn emit(&self, event: Event) {
        if self.alive() {
            let _ = self.tx.send_blocking(event);
        }
    }

    fn pause(&self, millis: u64) -> bool {
        let mut left = millis;
        while left > 0 {
            if !self.alive() {
                return false;
            }
            let step = left.min(40);
            thread::sleep(Duration::from_millis(step));
            left -= step;
        }
        self.alive()
    }

    /// Streams text a few characters at a time, like a model would.
    fn say(&self, text: &str) {
        let mut chunk = String::new();
        for (i, ch) in text.chars().enumerate() {
            chunk.push(ch);
            if chunk.len() >= 3 + (i % 5) || ch == '\n' {
                self.emit(Event::Delta {
                    bot: self.bot,
                    text: std::mem::take(&mut chunk),
                });
                if !self.pause(if ch == '.' { 90 } else { 22 }) {
                    return;
                }
            }
        }
        if !chunk.is_empty() {
            self.emit(Event::Delta {
                bot: self.bot,
                text: chunk,
            });
        }
    }

    fn step(&self, label: &str, millis: u64) -> bool {
        self.emit(Event::Activity {
            bot: self.bot,
            label: label.into(),
            done: false,
        });
        let ok = self.pause(millis);
        self.emit(Event::Activity {
            bot: self.bot,
            label: label.into(),
            done: true,
        });
        ok
    }

    fn computer(
        &self,
        url: &str,
        title: &str,
        status: &str,
        cursor: (f32, f32),
        typing: Option<&str>,
    ) {
        self.emit(Event::Computer {
            bot: self.bot,
            url: url.into(),
            title: title.into(),
            status: status.into(),
            cursor,
            typing: typing.map(Into::into),
        });
    }

    fn browse(&self, site: &str) -> bool {
        let url = format!("https://{site}");
        self.computer(&url, site, &format!("Opening {site}"), (0.42, 0.08), None);
        if !self.step(&format!("Opening {site}"), 1100) {
            return false;
        }
        self.computer(&url, site, "Reading the page", (0.35, 0.45), None);
        if !self.pause(900) {
            return false;
        }
        self.computer(&url, site, "Scrolling", (0.6, 0.7), None);
        if !self.pause(800) {
            return false;
        }
        self.computer(
            &url,
            site,
            "Clicking \u{201c}Sign in\u{201d}",
            (0.82, 0.12),
            None,
        );
        if !self.pause(800) {
            return false;
        }
        self.computer(
            &url,
            site,
            "Typing in search",
            (0.3, 0.3),
            Some("latest posts"),
        );
        self.step("Checked what the page shows", 900)
    }
}

fn topic(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().take(8).collect();
    let mut line = words.join(" ");
    if text.split_whitespace().count() > 8 {
        line.push('\u{2026}');
    }
    line
}

fn site_in(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '-'))
        .map(|w| {
            w.trim_start_matches("https://")
                .trim_start_matches("http://")
        })
        .find(|w| {
            w.contains('.')
                && !w.ends_with('.')
                && w.rsplit('.')
                    .next()
                    .is_some_and(|tld| (2..=6).contains(&tld.len()))
        })
        .map(|w| w.split('/').next().unwrap_or(w).to_string())
}

impl Backend for MockBackend {
    fn name(&self) -> &'static str {
        "Demo"
    }

    fn send(&self, request: Request) {
        match request {
            Request::Prompt {
                bot, persona, text, ..
            } => {
                let lower = text.to_lowercase();
                self.run(bot, move |s| {
                    if !s.pause(500) {
                        return;
                    }
                    let wants_approval = ["approve", "send", "publish", "buy", "email"]
                        .iter()
                        .any(|w| lower.contains(w));
                    let wants_question = lower.contains("which") || lower.trim_end().ends_with('?') && lower.len() < 40;
                    let site = site_in(&text);
                    let wants_computer = site.is_some()
                        || ["site", "browse", "check", "computer", "web", "look up"].iter().any(|w| lower.contains(w));
                    let wants_file = ["report", "file", "summary", "write up", "write-up"].iter().any(|w| lower.contains(w));

                    s.say(&format!("On it. {}", ack(&lower)));
                    if !s.pause(350) {
                        return;
                    }
                    if wants_question {
                        s.emit(Event::Question {
                            bot,
                            prompt: "Quick check before I start: should I cover just this week, or the whole month?".into(),
                            placeholder: "This week is fine".into(),
                        });
                        return;
                    }
                    if wants_computer {
                        let site = site.unwrap_or_else(|| "news.ycombinator.com".into());
                        if !s.browse(&site) {
                            return;
                        }
                    } else {
                        for (label, ms) in [("Reading your instructions", 700), ("Checking connected apps", 900)] {
                            if !s.step(label, ms) {
                                return;
                            }
                        }
                    }
                    if wants_approval {
                        s.emit(Event::ApprovalNeeded {
                            bot,
                            action: "Send the drafted message".into(),
                            detail: format!(
                                "{} drafted a reply about \u{201c}{}\u{201d}. Nothing is sent until you approve.",
                                persona.name,
                                topic(&text)
                            ),
                        });
                        return;
                    }
                    s.say(&answer(&text, &persona.name));
                    if wants_file {
                        s.pause(300);
                        let name = format!("{}.md", chrono::Local::now().format("%Y-%m-%d"));
                        let content = format!(
                            "# {}\n\nPrepared by {} on {}.\n\n## What I did\n\n- Read the request: {}\n- Checked the sources I can reach.\n\n## Findings\n\nThis is demo content from the built-in mock backend. Connect Pi to get real work done.\n",
                            topic(&text),
                            persona.name,
                            chrono::Local::now().format("%-d %b %Y"),
                            text.lines().next().unwrap_or_default()
                        );
                        let size = format!("{:.1} KB", content.len() as f32 / 1024.0);
                        s.emit(Event::File { bot, name, size, content });
                    }
                });
            }
            Request::Answer { bot, text, .. } => {
                self.run(bot, move |s| {
                    if !s.pause(500) {
                        return;
                    }
                    s.say(&format!(
                        "Got it: {}. Starting now.",
                        text.trim_end_matches('.')
                    ));
                    for (label, ms) in [
                        ("Collecting sources", 900),
                        ("Comparing with last time", 900),
                    ] {
                        if !s.step(label, ms) {
                            return;
                        }
                    }
                    s.say("Done. Nothing unusual this time. I'll flag anything that changes.");
                });
            }
            Request::Approval { bot, approved, .. } => {
                self.run(bot, move |s| {
                    if !s.pause(400) {
                        return;
                    }
                    if approved {
                        if !s.step("Sending", 1000) {
                            return;
                        }
                        s.say("Sent. I'll watch for replies and tell you when one needs you.");
                    } else {
                        s.say("Okay, I won't send it. The draft stays where it is if you change your mind.");
                    }
                });
            }
            Request::HandBack { bot } => {
                self.run(bot, move |s| {
                    if !s.pause(400) {
                        return;
                    }
                    s.computer(
                        "https://accounts.example.com",
                        "Signed in",
                        "Picking up where you left off",
                        (0.5, 0.5),
                        None,
                    );
                    s.say("Thanks, I'm signed in now. Picking up where I stopped.");
                    s.step("Continuing the task", 1200);
                });
            }
            Request::TestRun {
                bot,
                routine,
                persona,
                instruction,
            } => {
                self.run(bot, move |s| {
                    if !s.pause(400) {
                        return;
                    }
                    s.say(&format!(
                        "Test run of \u{201c}{}\u{201d}. This does the real work.",
                        topic(&instruction)
                    ));
                    for (label, ms) in [
                        ("Loading the saved skill", 800),
                        ("Running the steps", 1400),
                    ] {
                        if !s.step(label, ms) {
                            return;
                        }
                    }
                    s.say(&format!(
                        "Test run finished. {} will run it on schedule from now on.",
                        persona.name
                    ));
                    s.emit(Event::RoutineRun {
                        bot,
                        routine,
                        ok: true,
                    });
                });
            }
            Request::Abort { bot } => {
                self.generation(bot).fetch_add(1, Ordering::SeqCst);
                let _ = self.tx.send_blocking(Event::Finished { bot });
            }
            Request::Forget { bot } => {
                self.generation(bot).fetch_add(1, Ordering::SeqCst);
                self.generations.lock().unwrap().remove(&bot);
            }
        }
    }
}

fn ack(lower: &str) -> &'static str {
    if lower.contains("review") || lower.contains("check") {
        "I'll check which sources I can actually reach first."
    } else if lower.contains("plan") || lower.contains("week") {
        "I'll look at what's already on your plate before I plan anything."
    } else if lower.contains("email") || lower.contains("inbox") {
        "I'll go through the inbox and draft, not send."
    } else {
        "I'll take a look and report back here."
    }
}

fn answer(text: &str, name: &str) -> String {
    format!(
        "Here's where things stand on \u{201c}{}\u{201d}.\n\nWhat I checked: your instructions, the apps {name} can reach, and the pages above.\n\nWhat I could not check: anything behind a sign-in I don't have yet. I won't guess those numbers.\n\nNext: tell me if you want this as a routine, or save the steps as a skill with `Save as skill`.",
        topic(text)
    )
}
