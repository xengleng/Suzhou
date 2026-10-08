// All app state and behaviour. Screens read from it and call its methods;
// backend events land here. Mirrors src/ui/app.rs in the desktop app.

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'backend/backend.dart';
import 'backend/mock_backend.dart';
import 'backend/pi_backend.dart';
import 'data.dart';
import 'models.dart';

enum Stage { signIn, authorizing, tools, settingUp, meet, main }

class Toast {
  Toast(this.id, this.bot, this.title, this.body);
  final int id, bot;
  final String title, body;
}

class Settings {
  bool notifications = true;
  bool sounds = false;
  bool onlyWhenNeeded = false;
  bool autoReview = false;
  bool askSending = true;
  bool askPurchases = true;
  bool askPublishing = true;

  Map<String, dynamic> toJson() => {
    'notifications': notifications,
    'sounds': sounds,
    'onlyWhenNeeded': onlyWhenNeeded,
    'autoReview': autoReview,
    'askSending': askSending,
    'askPurchases': askPurchases,
    'askPublishing': askPublishing,
  };

  void load(Map<String, dynamic> j) {
    notifications = j['notifications'] ?? true;
    sounds = j['sounds'] ?? false;
    onlyWhenNeeded = j['onlyWhenNeeded'] ?? false;
    autoReview = j['autoReview'] ?? false;
    askSending = j['askSending'] ?? true;
    askPurchases = j['askPurchases'] ?? true;
    askPublishing = j['askPublishing'] ?? true;
  }
}

class AppState extends ChangeNotifier {
  AppState({this.demo = false, this.persist = true});

  final bool demo;
  final bool persist;

  Stage stage = Stage.signIn;
  String userName = 'You';
  List<Bot> bots = [];
  int _nextId = 100;
  List<Plugin> pluginList = plugins();
  final connecting = <String>{};
  List<String> toolsPicked = ['gmail', 'gcal'];
  Settings settings = Settings();
  ThemeMode themeMode = ThemeMode.system;
  String piBridge = '';
  bool showHidden = false;

  /// The chat on screen right now, if any.
  int? visibleBot;

  final toasts = ValueNotifier<List<Toast>>([]);

  late Backend backend;
  StreamSubscription<BotEvent>? _sub;
  Timer? _saveTimer;

  int nextId() => ++_nextId;

  // ----- Start-up and saving ------------------------------------------------

  Future<void> load() async {
    if (demo) {
      stage = Stage.main;
      bots = demoBots(nextId);
    } else if (persist) {
      final prefs = await SharedPreferences.getInstance();
      final raw = prefs.getString('suzhou.state');
      if (raw != null) {
        try {
          _apply(jsonDecode(raw) as Map<String, dynamic>);
        } catch (e) {
          debugPrint('suzhou: ignoring unreadable saved state: $e');
        }
      }
    }
    _connect();
  }

  void _apply(Map<String, dynamic> j) {
    stage = (j['signedIn'] ?? false) ? Stage.main : Stage.signIn;
    userName = j['userName'] ?? 'You';
    _nextId = j['nextId'] ?? 100;
    bots = [for (final b in (j['bots'] as List? ?? const [])) Bot.fromJson(b)];
    themeMode = ThemeMode.values.byName(j['theme'] ?? 'system');
    piBridge = j['piBridge'] ?? '';
    settings.load((j['settings'] as Map?)?.cast<String, dynamic>() ?? const {});
    final installed = Set<String>.from(j['plugins'] ?? const []);
    for (final p in pluginList) {
      p.installed = installed.contains(p.id);
    }
  }

  void _connect() {
    _sub?.cancel();
    backend = piBridge.isNotEmpty ? PiBackend(piBridge) : MockBackend();
    _sub = backend.events.listen(_onEvent);
  }

  void setPiBridge(String url) {
    piBridge = url.trim();
    backend.dispose();
    _connect();
    for (final b in bots) {
      b.primed = false;
    }
    save();
  }

  /// Saves soon, once, however many changes happen in between.
  void save() {
    notifyListeners();
    if (!persist || demo) return;
    _saveTimer?.cancel();
    _saveTimer = Timer(const Duration(milliseconds: 400), () async {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(
        'suzhou.state',
        jsonEncode({
          'signedIn': stage == Stage.main,
          'userName': userName,
          'nextId': _nextId,
          'bots': bots.map((b) => b.toJson()).toList(),
          'theme': themeMode.name,
          'piBridge': piBridge,
          'settings': settings.toJson(),
          'plugins': [
            for (final p in pluginList)
              if (p.installed) p.id,
          ],
        }),
      );
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    backend.dispose();
    super.dispose();
  }

  // ----- Lookups --------------------------------------------------------------

  Bot? bot(int id) {
    for (final b in bots) {
      if (b.id == id) return b;
    }
    return null;
  }

  List<Bot> roster(String query) {
    final q = query.toLowerCase();
    final list = bots
        .where((b) => !b.hidden || showHidden)
        .where((b) => q.isEmpty || b.name.toLowerCase().contains(q) || b.title.toLowerCase().contains(q) || b.preview.toLowerCase().contains(q))
        .toList();
    list.sort((a, b) => (b.pinned ? 1 : 0) - (a.pinned ? 1 : 0));
    return list;
  }

  int get hiddenCount => bots.where((b) => b.hidden).length;

  Persona _persona(Bot b) => Persona(b.name, b.title, b.description);

  Message push(int botId, Message m) {
    final b = bot(botId);
    if (b == null) return m;
    if (b.messages.isNotEmpty && b.messages.last.kind == MsgKind.bot) b.messages.last.streaming = false;
    m.fresh = true;
    if (m.time.isEmpty && const {MsgKind.user, MsgKind.bot, MsgKind.file, MsgKind.question, MsgKind.approval}.contains(m.kind)) {
      m.time = nowTime();
    }
    b.messages.add(m);
    return m;
  }

  Message _event(int botId, String text, {String? icon, String? link, int? routine}) =>
      push(botId, Message(id: nextId(), kind: MsgKind.event, text: text, icon: icon, link: link, routine: routine));

  // ----- Onboarding -----------------------------------------------------------

  void go(Stage s) {
    stage = s;
    switch (s) {
      case Stage.authorizing:
        Timer(const Duration(milliseconds: 2200), () {
          if (stage == Stage.authorizing) go(Stage.tools);
        });
      case Stage.settingUp:
        for (final p in pluginList) {
          if (toolsPicked.contains(p.id)) p.installed = true;
        }
        Timer(const Duration(milliseconds: 3600), () {
          if (stage == Stage.settingUp) go(Stage.meet);
        });
      default:
    }
    save();
  }

  void toggleTool(String id) {
    toolsPicked.contains(id) ? toolsPicked.remove(id) : toolsPicked.add(id);
    notifyListeners();
  }

  void signOut() {
    stage = Stage.signIn;
    save();
  }

  // ----- Bots -------------------------------------------------------------------

  Bot createBot(String name, int color, Shape shape, {String title = '', String description = ''}) {
    final b = Bot(id: nextId(), name: name, color: color, shape: shape, title: title, description: description);
    b.lastActivity = nowTime();
    bots.insert(0, b);
    push(b.id, Message(id: nextId(), kind: MsgKind.stamp, text: todayStamp()));
    _event(b.id, '$name joined your team');
    save();
    return b;
  }

  void togglePin(Bot b) {
    b.pinned = !b.pinned;
    save();
  }

  void toggleHidden(Bot b) {
    b.hidden = !b.hidden;
    save();
  }

  Bot duplicate(Bot src) {
    final copy = Bot(id: nextId(), name: '${src.name} copy', color: src.color, shape: src.shape, title: src.title, description: src.description);
    copy.skills = [for (final s in src.skills) Skill(s.name, s.description)];
    copy.routines = [
      for (final r in src.routines) Routine(id: nextId(), name: r.name, instruction: r.instruction, schedules: [...r.schedules], active: r.active),
    ];
    bots.insert(bots.indexOf(src) + 1, copy);
    push(copy.id, Message(id: nextId(), kind: MsgKind.stamp, text: todayStamp()));
    _event(copy.id, 'Duplicated from ${src.name}', icon: 'copy');
    save();
    return copy;
  }

  void deleteBot(Bot b) {
    backend.send(ForgetRequest(b.id));
    bots.remove(b);
    toasts.value = toasts.value.where((t) => t.bot != b.id).toList();
    save();
  }

  void opened(Bot b) {
    visibleBot = b.id;
    b.unread = false;
    toasts.value = toasts.value.where((t) => t.bot != b.id).toList();
    notifyListeners();
  }

  void closed(Bot b) {
    if (visibleBot == b.id) visibleBot = null;
    for (final m in b.messages) {
      m.fresh = false;
    }
  }

  // ----- Conversation -----------------------------------------------------------

  void send(Bot b, String text, {List<String> attachments = const [], String? replyTo}) {
    text = text.trim();
    if (text.isEmpty && attachments.isEmpty) return;
    final m = push(b.id, Message(id: nextId(), kind: MsgKind.user, text: text, attachments: [...attachments], replyTo: replyTo));
    m.replyTo = replyTo;
    final first = !b.primed;
    b.primed = true;
    b.status = BotStatus.working;
    var prompt = text;
    if (replyTo != null) prompt = '> $replyTo\n\n$prompt';
    for (final f in attachments) {
      prompt += '\n\n[Attached: $f]';
    }
    backend.send(PromptRequest(b.id, _persona(b), prompt, first: first));
    save();
  }

  void stop(Bot b) {
    if (b.status != BotStatus.working) return;
    backend.send(AbortRequest(b.id));
    _event(b.id, 'Stopped', icon: 'stop');
    save();
  }

  void answer(Bot b, Message q, String text) {
    if (q.answer != null) return;
    q.answer = text.trim().isEmpty ? (q.placeholder ?? '') : text.trim();
    b.status = BotStatus.working;
    backend.send(AnswerRequest(b.id, q.answer!));
    save();
  }

  void decide(Bot b, Message m, bool approved) {
    if (m.approval != ApprovalState.pending) return;
    m.approval = approved ? ApprovalState.approved : ApprovalState.denied;
    b.status = BotStatus.working;
    backend.send(ApprovalRequest(b.id, approved));
    save();
  }

  void react(Message m, Reaction r) {
    m.reactions.contains(r) ? m.reactions.remove(r) : m.reactions.add(r);
    save();
  }

  void deleteMessage(Bot b, Message m) {
    b.messages.remove(m);
    save();
  }

  void saveAsSkill(Bot b, Message m) {
    final words = m.text.split(RegExp(r'\s+')).map((w) => w.replaceAll(RegExp(r'[^\w]'), '').toLowerCase()).where((w) => w.isNotEmpty).take(3).join('-');
    final name = words.isEmpty ? 'saved-process' : words;
    final line = m.text.split('\n').first;
    b.skills.add(Skill(name, line.length > 90 ? line.substring(0, 90) : line));
    _event(b.id, 'Saved the process as skill', icon: 'book', link: name);
    save();
  }

  void teach(Bot b, int seconds) {
    final name = 'taught-task-${b.skills.length + 1}';
    b.skills.add(Skill(name, 'Shown by demonstration (${seconds < 1 ? 1 : seconds}s)'));
    _event(b.id, 'Saved skill', icon: 'book', link: name);
    save();
  }

  void takeOver(Bot b) {
    if (b.status == BotStatus.working) backend.send(AbortRequest(b.id));
    _event(b.id, "You took over ${b.name}'s computer", icon: 'hand');
    save();
  }

  void handBack(Bot b) {
    _event(b.id, 'You handed the computer back to ${b.name}', icon: 'hand');
    b.status = BotStatus.working;
    backend.send(HandBackRequest(b.id));
    save();
  }

  void moveComputerPointer(Bot b, double x, double y) {
    b.computer.x = x;
    b.computer.y = y;
    notifyListeners();
  }

  // ----- Routines -----------------------------------------------------------------

  Routine newRoutine(Bot b) {
    final r = Routine(id: nextId(), name: 'Routine ${b.routines.length + 1}');
    b.routines.add(r);
    _event(b.id, 'Created routine', icon: 'clock', link: r.name, routine: r.id);
    save();
    return r;
  }

  void deleteRoutine(Bot b, Routine r) {
    b.routines.remove(r);
    save();
  }

  void testRun(Bot b, Routine r) {
    if (r.history.isNotEmpty && r.history.first.status == RunStatus.running) return;
    r.history.insert(0, RunRecord(nowFull(), RunStatus.running));
    if (r.history.length > 20) r.history.removeLast();
    b.status = BotStatus.working;
    _event(b.id, 'Test run', icon: 'play', link: r.name, routine: r.id);
    backend.send(TestRunRequest(b.id, r.id, _persona(b), r.instruction.trim().isEmpty ? r.name : r.instruction));
    save();
  }

  // ----- Plugins ------------------------------------------------------------------

  void install(Plugin p) {
    if (connecting.contains(p.id)) return;
    connecting.add(p.id);
    notifyListeners();
    Timer(const Duration(milliseconds: 1400), () {
      connecting.remove(p.id);
      p.installed = true;
      save();
    });
  }

  void uninstall(Plugin p) {
    p.installed = false;
    save();
  }

  // ----- Notifications --------------------------------------------------------------

  void _toast(int botId, String title, String body) {
    if (!settings.notifications) return;
    final t = Toast(nextId(), botId, title, body);
    toasts.value = [...toasts.value.where((x) => x.bot != botId), t].take(3).toList();
    Timer(const Duration(milliseconds: 5200), () => dismissToast(t.id));
  }

  void dismissToast(int id) {
    toasts.value = toasts.value.where((t) => t.id != id).toList();
  }

  // ----- Backend events ---------------------------------------------------------------

  void _onEvent(BotEvent e) {
    final b = bot(e.bot);
    if (b == null) return;
    final visible = visibleBot == b.id;
    switch (e) {
      case Started():
        b.status = BotStatus.working;
      case Delta(:final text):
        final last = b.messages.isEmpty ? null : b.messages.last;
        if (last != null && last.kind == MsgKind.bot && last.streaming) {
          last.text += text;
        } else {
          push(b.id, Message(id: nextId(), kind: MsgKind.bot, text: text, streaming: true));
        }
      case Activity(:final label, :final done):
        final open = b.messages.reversed.take(4).where((m) => m.kind == MsgKind.activity && !m.done && m.text == label);
        if (open.isNotEmpty) {
          open.first.done = done;
        } else {
          push(b.id, Message(id: nextId(), kind: MsgKind.activity, text: label, done: done));
        }
      case Question(:final prompt, :final placeholder):
        push(b.id, Message(id: nextId(), kind: MsgKind.question, text: prompt, placeholder: placeholder));
        b.status = BotStatus.needsYou;
        if (!visible) _toast(b.id, '${b.name} has a question', prompt);
      case ApprovalNeeded(:final action, :final detail):
        push(b.id, Message(id: nextId(), kind: MsgKind.approval, text: action, detail: detail));
        b.status = BotStatus.needsYou;
        if (!visible) _toast(b.id, '${b.name} needs approval', action);
      case FileSent(:final name, :final size, :final content):
        push(b.id, Message(id: nextId(), kind: MsgKind.file, fileName: name, fileSize: size, text: content));
      case Note(:final text):
        _event(b.id, text, icon: 'info');
      case ComputerUpdate(:final url, :final title, :final status, :final x, :final y, :final typing):
        b.computer
          ..url = url
          ..title = title
          ..status = status
          ..x = x
          ..y = y
          ..typing = typing
          ..active = true;
      case RoutineRun(:final routine, :final ok):
        for (final r in b.routines.where((r) => r.id == routine)) {
          for (final h in r.history.where((h) => h.status == RunStatus.running)) {
            h.status = ok ? RunStatus.succeeded : RunStatus.failed;
          }
        }
      case Failed(:final message):
        if (b.status == BotStatus.working) b.status = BotStatus.idle;
        _event(b.id, message, icon: 'info');
      case Finished():
        if (b.messages.isNotEmpty) b.messages.last.streaming = false;
        for (final m in b.messages.where((m) => m.kind == MsgKind.activity)) {
          m.done = true;
        }
        final waiting = b.messages.any(
          (m) => (m.kind == MsgKind.question && m.answer == null) || (m.kind == MsgKind.approval && m.approval == ApprovalState.pending),
        );
        final wasWorking = b.status == BotStatus.working;
        b.status = waiting ? BotStatus.needsYou : BotStatus.idle;
        b.computer.status = 'Idle';
        if (!visible) {
          b.unread = true;
          if (wasWorking && !waiting && !settings.onlyWhenNeeded) {
            final p = b.preview;
            _toast(b.id, '${b.name} finished', p.length > 90 ? '${p.substring(0, 90)}…' : p);
          }
        }
        save();
        return;
    }
    notifyListeners();
  }
}

/// Makes the AppState reachable from any widget.
class AppScope extends InheritedNotifier<AppState> {
  const AppScope({super.key, required AppState state, required super.child}) : super(notifier: state);

  static AppState of(BuildContext context) => context.dependOnInheritedWidgetOfExactType<AppScope>()!.notifier!;
  static AppState read(BuildContext context) => context.getInheritedWidgetOfExactType<AppScope>()!.notifier!;
}
