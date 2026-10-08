//! Drives the Pi coding agent in RPC mode.
//!
//! Each Bot gets its own `pi --mode rpc` process, started on its first
//! message, with its own working folder under the data directory so its
//! files and sessions stay apart from the other Bots'. We write one JSON
//! command per line to its stdin and read one JSON event per line from its
//! stdout.
//!
//! The protocol mapping is written from Pi's RPC documentation and parses
//! defensively: unknown events are ignored, and a field that is missing just
//! means that part of the UI stays quiet. Settings:
//!
//! - `SUZHOU_PI_BIN`: the executable (default `pi`)
//! - `SUZHOU_PI_ARGS`: extra arguments, split on spaces (e.g. `--provider anthropic`)

use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use async_channel::Sender;
use serde_json::{Value, json};

use super::{Backend, Event, Request};
use crate::model::BotId;

struct Proc {
    child: Child,
    stdin: ChildStdin,
}

#[derive(Default)]
struct Shared {
    procs: HashMap<BotId, Proc>,
    /// Pi's `extension_ui_request` ids waiting on the user, oldest first.
    pending_ui: HashMap<BotId, VecDeque<(String, UiKind)>>,
}

#[derive(Clone, Copy, PartialEq)]
enum UiKind {
    Confirm,
    Input,
}

pub struct PiBackend {
    tx: Sender<Event>,
    shared: Arc<Mutex<Shared>>,
    bin: String,
    args: Vec<String>,
}

impl PiBackend {
    pub fn new(tx: Sender<Event>) -> Self {
        PiBackend {
            tx,
            shared: Arc::default(),
            bin: std::env::var("SUZHOU_PI_BIN").unwrap_or_else(|_| "pi".into()),
            args: std::env::var("SUZHOU_PI_ARGS")
                .map(|a| a.split_whitespace().map(String::from).collect())
                .unwrap_or_default(),
        }
    }

    fn workspace(bot: BotId) -> PathBuf {
        let dir = crate::store::data_dir().join("bots").join(bot.to_string());
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn ensure_process(&self, bot: BotId) -> Result<(), String> {
        let mut shared = self.shared.lock().unwrap();
        if let Some(proc) = shared.procs.get_mut(&bot) {
            if matches!(proc.child.try_wait(), Ok(None)) {
                return Ok(());
            }
            shared.procs.remove(&bot);
        }
        let mut child = Command::new(&self.bin)
            .arg("--mode")
            .arg("rpc")
            .args(&self.args)
            .current_dir(Self::workspace(bot))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                format!(
                    "Couldn't start `{}`: {err}. Is Pi installed and on PATH?",
                    self.bin
                )
            })?;
        let stdin = child.stdin.take().ok_or("Pi has no stdin")?;
        let stdout = child.stdout.take().ok_or("Pi has no stdout")?;
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    eprintln!("pi[{bot}]: {line}");
                }
            });
        }
        let tx = self.tx.clone();
        let shared_for_reader = self.shared.clone();
        thread::spawn(move || {
            let mut turn = Turn::default();
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let Ok(value) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                for event in translate(bot, &value, &mut turn, &shared_for_reader) {
                    if tx.send_blocking(event).is_err() {
                        return;
                    }
                }
            }
            let _ = tx.send_blocking(Event::Finished { bot });
        });
        shared.procs.insert(bot, Proc { child, stdin });
        Ok(())
    }

    fn write(&self, bot: BotId, command: Value) -> Result<(), String> {
        let mut shared = self.shared.lock().unwrap();
        let proc = shared
            .procs
            .get_mut(&bot)
            .ok_or("Pi is not running for this Bot")?;
        let mut line = command.to_string();
        line.push('\n');
        proc.stdin
            .write_all(line.as_bytes())
            .and_then(|_| proc.stdin.flush())
            .map_err(|err| format!("Lost the connection to Pi: {err}"))
    }

    fn prompt(&self, bot: BotId, message: String) {
        let result = self
            .ensure_process(bot)
            .and_then(|_| self.write(bot, json!({ "type": "prompt", "message": message })));
        if let Err(message) = result {
            let _ = self.tx.send_blocking(Event::Error { bot, message });
            let _ = self.tx.send_blocking(Event::Finished { bot });
        }
    }

    fn answer_ui(&self, bot: BotId, want: UiKind, value: Value) -> bool {
        let next = {
            let mut shared = self.shared.lock().unwrap();
            let queue = shared.pending_ui.entry(bot).or_default();
            let at = queue.iter().position(|(_, kind)| *kind == want);
            at.and_then(|at| queue.remove(at))
        };
        let Some((id, kind)) = next else {
            return false;
        };
        let mut response = json!({ "type": "extension_ui_response", "id": id });
        match kind {
            UiKind::Confirm => response["confirmed"] = value,
            UiKind::Input => response["value"] = value,
        }
        self.write(bot, response).is_ok()
    }
}

impl Drop for PiBackend {
    fn drop(&mut self) {
        for (_, mut proc) in self.shared.lock().unwrap().procs.drain() {
            let _ = proc.child.kill();
        }
    }
}

impl Backend for PiBackend {
    fn name(&self) -> &'static str {
        "Pi"
    }

    fn send(&self, request: Request) {
        match request {
            Request::Prompt {
                bot,
                persona,
                first,
                text,
            } => {
                let message = if first {
                    format!("{}\n\n---\n\n{text}", persona.preamble())
                } else {
                    text
                };
                self.prompt(bot, message);
            }
            Request::Answer { bot, text, .. } => {
                if !self.answer_ui(bot, UiKind::Input, json!(text)) {
                    self.prompt(bot, text);
                }
            }
            Request::Approval { bot, approved, .. } => {
                if !self.answer_ui(bot, UiKind::Confirm, json!(approved)) {
                    let text = if approved {
                        "Approved. Go ahead."
                    } else {
                        "Denied. Don't do that."
                    };
                    self.prompt(bot, text.into());
                }
            }
            Request::HandBack { bot } => {
                self.prompt(
                    bot,
                    "I've finished on the computer and handed control back. Please continue."
                        .into(),
                );
            }
            Request::TestRun {
                bot,
                persona,
                instruction,
                ..
            } => {
                self.prompt(
                    bot,
                    format!(
                        "{}\n\n---\n\nRoutine test run. {instruction}",
                        persona.preamble()
                    ),
                );
            }
            Request::Abort { bot } => {
                let _ = self.write(bot, json!({ "type": "abort" }));
            }
            Request::Forget { bot } => {
                let mut shared = self.shared.lock().unwrap();
                if let Some(mut proc) = shared.procs.remove(&bot) {
                    let _ = proc.child.kill();
                }
                shared.pending_ui.remove(&bot);
            }
        }
    }
}

#[derive(Default)]
struct Turn {
    /// Whether text arrived as deltas, so `message_end` doesn't repeat it.
    streamed: bool,
}

fn str_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn tool_label(name: &str, args: &Value) -> String {
    let detail = ["command", "path", "file_path", "url", "query", "pattern"]
        .iter()
        .find_map(|key| str_at(args, key))
        .map(|s| {
            s.lines()
                .next()
                .unwrap_or(s)
                .chars()
                .take(80)
                .collect::<String>()
        });
    match (name, detail) {
        ("bash", Some(cmd)) => format!("Ran `{cmd}`"),
        ("read", Some(path)) => format!("Read {path}"),
        ("write", Some(path)) => format!("Wrote {path}"),
        ("edit", Some(path)) => format!("Edited {path}"),
        (name, Some(detail)) => format!("{name}: {detail}"),
        (name, None) => format!("Used {name}"),
    }
}

/// Turns one line of Pi output into zero or more UI events.
fn translate(
    bot: BotId,
    value: &Value,
    turn: &mut Turn,
    shared: &Arc<Mutex<Shared>>,
) -> Vec<Event> {
    let kind = str_at(value, "type").unwrap_or_default();
    match kind {
        "agent_start" => {
            turn.streamed = false;
            vec![Event::Started { bot }]
        }
        "agent_end" => vec![Event::Finished { bot }],
        "message_start" => {
            turn.streamed = false;
            vec![]
        }
        "message_update" => {
            let Some(inner) = value.get("assistantMessageEvent") else {
                return vec![];
            };
            match str_at(inner, "type") {
                Some("text_delta") => {
                    turn.streamed = true;
                    str_at(inner, "delta")
                        .map(|delta| {
                            vec![Event::Delta {
                                bot,
                                text: delta.into(),
                            }]
                        })
                        .unwrap_or_default()
                }
                _ => vec![],
            }
        }
        "message_end" => {
            let message = value.get("message");
            let is_assistant = message.and_then(|m| str_at(m, "role")) == Some("assistant");
            if !is_assistant || turn.streamed {
                return vec![];
            }
            let text: String = message
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
                .map(|parts| {
                    parts
                        .iter()
                        .filter(|p| str_at(p, "type") == Some("text"))
                        .filter_map(|p| str_at(p, "text"))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                })
                .unwrap_or_default();
            if text.is_empty() {
                vec![]
            } else {
                vec![Event::Delta { bot, text }]
            }
        }
        "tool_execution_start" | "tool_execution_end" => {
            let name = str_at(value, "toolName").unwrap_or("tool");
            let args = value.get("args").cloned().unwrap_or(Value::Null);
            let mut events = vec![Event::Activity {
                bot,
                label: tool_label(name, &args),
                done: kind == "tool_execution_end",
            }];
            if kind == "tool_execution_end"
                && value.get("isError").and_then(Value::as_bool) == Some(true)
            {
                events.push(Event::Note {
                    bot,
                    text: format!("{name} failed"),
                });
            }
            events
        }
        "extension_ui_request" => {
            let id = str_at(value, "id").unwrap_or_default().to_string();
            let title = str_at(value, "title")
                .or_else(|| str_at(value, "message"))
                .unwrap_or("Pi needs you");
            let detail = str_at(value, "message").unwrap_or_default();
            match str_at(value, "method") {
                Some("confirm") => {
                    shared
                        .lock()
                        .unwrap()
                        .pending_ui
                        .entry(bot)
                        .or_default()
                        .push_back((id, UiKind::Confirm));
                    vec![Event::ApprovalNeeded {
                        bot,
                        action: title.into(),
                        detail: detail.into(),
                    }]
                }
                Some("input") | Some("editor") | Some("select") => {
                    shared
                        .lock()
                        .unwrap()
                        .pending_ui
                        .entry(bot)
                        .or_default()
                        .push_back((id, UiKind::Input));
                    let options = value
                        .get("options")
                        .and_then(Value::as_array)
                        .map(|o| {
                            o.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(" / ")
                        })
                        .unwrap_or_default();
                    vec![Event::Question {
                        bot,
                        prompt: title.into(),
                        placeholder: if options.is_empty() {
                            str_at(value, "placeholder").unwrap_or("Answer").into()
                        } else {
                            options
                        },
                    }]
                }
                Some("notify") => vec![Event::Note {
                    bot,
                    text: title.into(),
                }],
                _ => vec![],
            }
        }
        "response" => {
            if value.get("success").and_then(Value::as_bool) == Some(false) {
                let message = str_at(value, "error")
                    .unwrap_or("Pi refused the request")
                    .to_string();
                vec![Event::Error { bot, message }, Event::Finished { bot }]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(lines: &[Value]) -> Vec<Event> {
        let shared = Arc::default();
        let mut turn = Turn::default();
        lines
            .iter()
            .flat_map(|l| translate(7, l, &mut turn, &shared))
            .collect()
    }

    #[test]
    fn streams_text_and_tools() {
        let events = run(&[
            json!({"type": "agent_start"}),
            json!({"type": "message_start", "message": {"role": "assistant"}}),
            json!({"type": "message_update", "assistantMessageEvent": {"type": "text_delta", "delta": "Hi"}}),
            json!({"type": "message_end", "message": {"role": "assistant", "content": [{"type": "text", "text": "Hi"}]}}),
            json!({"type": "tool_execution_start", "toolName": "bash", "args": {"command": "ls -la"}}),
            json!({"type": "tool_execution_end", "toolName": "bash", "args": {"command": "ls -la"}, "isError": false}),
            json!({"type": "agent_end"}),
        ]);
        assert!(matches!(events[0], Event::Started { bot: 7 }));
        assert!(matches!(&events[1], Event::Delta { text, .. } if text == "Hi"));
        assert!(
            matches!(&events[2], Event::Activity { label, done: false, .. } if label == "Ran `ls -la`")
        );
        assert!(matches!(&events[3], Event::Activity { done: true, .. }));
        assert!(matches!(events[4], Event::Finished { bot: 7 }));
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn unstreamed_messages_still_show() {
        let events = run(&[
            json!({"type": "message_end", "message": {"role": "assistant", "content": [{"type": "text", "text": "Whole reply"}]}}),
        ]);
        assert!(matches!(&events[0], Event::Delta { text, .. } if text == "Whole reply"));
    }

    #[test]
    fn confirm_becomes_approval() {
        let events = run(&[
            json!({"type": "extension_ui_request", "id": "u1", "method": "confirm", "title": "Delete file?", "message": "rm notes.md"}),
        ]);
        assert!(
            matches!(&events[0], Event::ApprovalNeeded { action, .. } if action == "Delete file?")
        );
    }

    #[test]
    fn failed_command_reports_error() {
        let events = run(&[
            json!({"type": "response", "command": "prompt", "success": false, "error": "no model"}),
        ]);
        assert!(matches!(&events[0], Event::Error { message, .. } if message == "no model"));
    }
}
