// A pretend Bot, so every screen works before a real agent is connected.
// Same scripts as src/backend/mock.rs: words in the prompt steer it.
// "approve", "send", "publish" -> asks for approval; "which" or a short
// question -> asks back; a web address or "check"/"browse" -> uses the
// Agent Computer; "report"/"file" -> sends a file.

import 'dart:async';

import '../data.dart';
import 'backend.dart';

class MockBackend implements Backend {
  final _events = StreamController<BotEvent>.broadcast();
  final _generation = <int, int>{};

  @override
  String get name => 'Demo';

  @override
  Stream<BotEvent> get events => _events.stream;

  @override
  void dispose() => _events.close();

  void _run(int bot, Future<void> Function(_Script s) body) {
    final gen = (_generation[bot] ?? 0) + 1;
    _generation[bot] = gen;
    final s = _Script(bot, this, gen);
    () async {
      s.emit(Started(bot));
      await body(s);
      s.emit(Finished(bot));
    }();
  }

  @override
  void send(Request request) {
    switch (request) {
      case PromptRequest(:final bot, :final persona, :final text):
        final lower = text.toLowerCase();
        _run(bot, (s) async {
          if (!await s.pause(500)) return;
          final wantsApproval = ['approve', 'send', 'publish', 'buy', 'email'].any(lower.contains);
          final wantsQuestion = lower.contains('which') || (lower.trimRight().endsWith('?') && lower.length < 40);
          final site = _siteIn(text);
          final wantsComputer = site != null || ['site', 'browse', 'check', 'computer', 'web', 'look up'].any(lower.contains);
          final wantsFile = ['report', 'file', 'summary', 'write up'].any(lower.contains);

          await s.say('On it. ${_ack(lower)}');
          if (!await s.pause(350)) return;
          if (wantsQuestion) {
            s.emit(Question(bot, 'Quick check before I start: should I cover just this week, or the whole month?', 'This week is fine'));
            return;
          }
          if (wantsComputer) {
            if (!await s.browse(site ?? 'news.ycombinator.com')) return;
          } else {
            if (!await s.step('Reading your instructions', 700)) return;
            if (!await s.step('Checking connected apps', 900)) return;
          }
          if (wantsApproval) {
            s.emit(
              ApprovalNeeded(bot, 'Send the drafted message', '${persona.name} drafted a reply about “${_topic(text)}”. Nothing is sent until you approve.'),
            );
            return;
          }
          await s.say(_answer(text, persona.name));
          if (wantsFile) {
            await s.pause(300);
            final content =
                '# ${_topic(text)}\n\nPrepared by ${persona.name}.\n\n## What I did\n\n- Read the request\n- Checked the sources I can reach\n\n## Findings\n\nDemo content from the built-in mock backend. Connect Pi to get real work done.\n';
            s.emit(FileSent(bot, '${isoDate()}.md', '${(content.length / 1024).toStringAsFixed(1)} KB', content));
          }
        });
      case AnswerRequest(:final bot, :final text):
        _run(bot, (s) async {
          if (!await s.pause(500)) return;
          await s.say('Got it: ${text.replaceAll(RegExp(r'\.$'), '')}. Starting now.');
          if (!await s.step('Collecting sources', 900)) return;
          if (!await s.step('Comparing with last time', 900)) return;
          await s.say("Done. Nothing unusual this time. I'll flag anything that changes.");
        });
      case ApprovalRequest(:final bot, :final approved):
        _run(bot, (s) async {
          if (!await s.pause(400)) return;
          if (approved) {
            if (!await s.step('Sending', 1000)) return;
            await s.say("Sent. I'll watch for replies and tell you when one needs you.");
          } else {
            await s.say("Okay, I won't send it. The draft stays where it is if you change your mind.");
          }
        });
      case HandBackRequest(:final bot):
        _run(bot, (s) async {
          if (!await s.pause(400)) return;
          s.computer('https://accounts.example.com', 'Signed in', 'Picking up where you left off', 0.5, 0.5);
          await s.say("Thanks, I'm signed in now. Picking up where I stopped.");
          await s.step('Continuing the task', 1200);
        });
      case TestRunRequest(:final bot, :final routine, :final persona, :final instruction):
        _run(bot, (s) async {
          if (!await s.pause(400)) return;
          await s.say('Test run of “${_topic(instruction)}”. This does the real work.');
          if (!await s.step('Loading the saved skill', 800)) return;
          if (!await s.step('Running the steps', 1400)) return;
          await s.say('Test run finished. ${persona.name} will run it on schedule from now on.');
          s.emit(RoutineRun(bot, routine, ok: true));
        });
      case AbortRequest(:final bot):
        _generation[bot] = (_generation[bot] ?? 0) + 1;
        _events.add(Finished(bot));
      case ForgetRequest(:final bot):
        _generation[bot] = (_generation[bot] ?? 0) + 1;
    }
  }
}

class _Script {
  _Script(this.bot, this.owner, this.gen);
  final int bot;
  final MockBackend owner;
  final int gen;

  bool get alive => owner._generation[bot] == gen;

  void emit(BotEvent e) {
    if (alive && !owner._events.isClosed) owner._events.add(e);
  }

  Future<bool> pause(int ms) async {
    await Future<void>.delayed(Duration(milliseconds: ms));
    return alive;
  }

  Future<void> say(String text) async {
    final chunk = StringBuffer();
    var i = 0;
    for (final ch in text.split('')) {
      chunk.write(ch);
      if (chunk.length >= 3 + (i % 5) || ch == '\n') {
        emit(Delta(bot, chunk.toString()));
        chunk.clear();
        if (!await pause(ch == '.' ? 90 : 22)) return;
      }
      i++;
    }
    if (chunk.isNotEmpty) emit(Delta(bot, chunk.toString()));
  }

  Future<bool> step(String label, int ms) async {
    emit(Activity(bot, label, done: false));
    final ok = await pause(ms);
    emit(Activity(bot, label, done: true));
    return ok;
  }

  void computer(String url, String title, String status, double x, double y, [String? typing]) =>
      emit(ComputerUpdate(bot, url: url, title: title, status: status, x: x, y: y, typing: typing));

  Future<bool> browse(String site) async {
    final url = 'https://$site';
    computer(url, site, 'Opening $site', 0.42, 0.08);
    if (!await step('Opening $site', 1100)) return false;
    computer(url, site, 'Reading the page', 0.35, 0.45);
    if (!await pause(900)) return false;
    computer(url, site, 'Scrolling', 0.6, 0.7);
    if (!await pause(800)) return false;
    computer(url, site, 'Clicking “Sign in”', 0.82, 0.12);
    if (!await pause(800)) return false;
    computer(url, site, 'Typing in search', 0.3, 0.3, 'latest posts');
    return step('Checked what the page shows', 900);
  }
}

String _topic(String text) {
  final words = text.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).toList();
  final line = words.take(8).join(' ');
  return words.length > 8 ? '$line…' : line;
}

String? _siteIn(String text) {
  for (var w in text.split(RegExp(r'\s+'))) {
    w = w.replaceAll(RegExp(r'^[^\w]+|[^\w/.-]+$'), '').replaceFirst(RegExp(r'^https?://'), '');
    final tld = w.split('.').last;
    if (w.contains('.') && !w.endsWith('.') && tld.length >= 2 && tld.length <= 6) return w.split('/').first;
  }
  return null;
}

String _ack(String lower) {
  if (lower.contains('review') || lower.contains('check')) return "I'll check which sources I can actually reach first.";
  if (lower.contains('plan') || lower.contains('week')) return "I'll look at what's already on your plate before I plan anything.";
  if (lower.contains('email') || lower.contains('inbox')) return "I'll go through the inbox and draft, not send.";
  return "I'll take a look and report back here.";
}

String _answer(String text, String name) =>
    "Here's where things stand on “${_topic(text)}”.\n\nWhat I checked: your instructions, the apps $name can reach, and the pages above.\n\nWhat I could not check: anything behind a sign-in I don't have yet. I won't guess those numbers.\n\nNext: tell me if you want this as a routine, or save the steps as a skill.";
