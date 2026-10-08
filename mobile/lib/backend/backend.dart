// The seam between the screens and whatever does the work.
//
// Screens send Requests and react to Events, exactly like the desktop app
// (src/backend/mod.rs). Two backends:
//
// - MockBackend: a scripted Bot so every screen can be tried offline.
// - PiBackend: Pi's RPC protocol over a WebSocket. A phone cannot run the
//   `pi` command itself, so `tool/pi_bridge.dart` runs next to Pi on a
//   computer and relays one `pi --mode rpc` process per Bot.

class Persona {
  const Persona(this.name, this.title, this.description);
  final String name, title, description;

  /// Sent before a Bot's first message so the agent can play the role.
  String get preamble {
    final b = StringBuffer('You are $name');
    if (title.isNotEmpty) b.write(', $title');
    b.write(". You are one of the user's always-on Bots: a teammate they give real work to.");
    if (description.isNotEmpty) b.write('\n\nYour job: $description');
    b.write('\n\nReply in short, plain chat messages. Ask before anything that sends, publishes, deletes or spends money.');
    return b.toString();
  }
}

sealed class Request {
  const Request(this.bot);
  final int bot;
}

class PromptRequest extends Request {
  const PromptRequest(super.bot, this.persona, this.text, {required this.first});
  final Persona persona;
  final String text;
  final bool first;
}

class AbortRequest extends Request {
  const AbortRequest(super.bot);
}

class AnswerRequest extends Request {
  const AnswerRequest(super.bot, this.text);
  final String text;
}

class ApprovalRequest extends Request {
  const ApprovalRequest(super.bot, this.approved);
  final bool approved;
}

class HandBackRequest extends Request {
  const HandBackRequest(super.bot);
}

class TestRunRequest extends Request {
  const TestRunRequest(super.bot, this.routine, this.persona, this.instruction);
  final int routine;
  final Persona persona;
  final String instruction;
}

class ForgetRequest extends Request {
  const ForgetRequest(super.bot);
}

sealed class BotEvent {
  const BotEvent(this.bot);
  final int bot;
}

class Started extends BotEvent {
  const Started(super.bot);
}

class Delta extends BotEvent {
  const Delta(super.bot, this.text);
  final String text;
}

class Activity extends BotEvent {
  const Activity(super.bot, this.label, {required this.done});
  final String label;
  final bool done;
}

class Question extends BotEvent {
  const Question(super.bot, this.prompt, this.placeholder);
  final String prompt, placeholder;
}

class ApprovalNeeded extends BotEvent {
  const ApprovalNeeded(super.bot, this.action, this.detail);
  final String action, detail;
}

class FileSent extends BotEvent {
  const FileSent(super.bot, this.name, this.size, this.content);
  final String name, size, content;
}

class Note extends BotEvent {
  const Note(super.bot, this.text);
  final String text;
}

class ComputerUpdate extends BotEvent {
  const ComputerUpdate(super.bot, {required this.url, required this.title, required this.status, required this.x, required this.y, this.typing});
  final String url, title, status;
  final double x, y;
  final String? typing;
}

class RoutineRun extends BotEvent {
  const RoutineRun(super.bot, this.routine, {required this.ok});
  final int routine;
  final bool ok;
}

class Finished extends BotEvent {
  const Finished(super.bot);
}

class Failed extends BotEvent {
  const Failed(super.bot, this.message);
  final String message;
}

abstract class Backend {
  String get name;
  Stream<BotEvent> get events;
  void send(Request request);
  void dispose();
}
