// What the app knows about: Bots, their conversations, routines and skills.
// Mirrors src/model.rs in the desktop app.

enum Shape { circle, blob, square, pill, triangle, hexagon, cloud, drop }

enum BotStatus { idle, working, needsYou }

enum ApprovalState { pending, approved, denied }

enum RunStatus { running, succeeded, failed }

enum Reaction { thumbsUp, heart, check, party }

class Skill {
  Skill(this.name, this.description);
  String name;
  String description;

  Map<String, dynamic> toJson() => {'name': name, 'description': description};
  factory Skill.fromJson(Map<String, dynamic> j) => Skill(j['name'], j['description']);
}

class RunRecord {
  RunRecord(this.when, this.status);
  String when;
  RunStatus status;

  Map<String, dynamic> toJson() => {'when': when, 'status': status.name};
  factory RunRecord.fromJson(Map<String, dynamic> j) => RunRecord(j['when'], RunStatus.values.byName(j['status']));
}

class Routine {
  Routine({required this.id, required this.name, this.instruction = '', List<String>? schedules, this.active = true, List<RunRecord>? history})
    : schedules = schedules ?? ['Every day at 9:00 AM'],
      history = history ?? [];
  final int id;
  String name;
  String instruction;
  List<String> schedules;
  bool active;
  List<RunRecord> history;

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'instruction': instruction,
    'schedules': schedules,
    'active': active,
    'history': history.map((h) => h.toJson()).toList(),
  };
  factory Routine.fromJson(Map<String, dynamic> j) => Routine(
    id: j['id'],
    name: j['name'],
    instruction: j['instruction'] ?? '',
    schedules: List<String>.from(j['schedules'] ?? const []),
    active: j['active'] ?? true,
    history: [for (final h in (j['history'] as List? ?? const [])) RunRecord.fromJson(h)],
  );
}

/// One item in a conversation. `kind` says which of the fields matter.
enum MsgKind { user, bot, event, stamp, activity, file, question, approval, snapshot }

class Message {
  Message({
    required this.id,
    required this.kind,
    this.text = '',
    this.time = '',
    this.streaming = false,
    this.done = false,
    this.icon,
    this.link,
    this.routine,
    this.fileName,
    this.fileSize,
    this.placeholder,
    this.answer,
    this.detail,
    this.approval = ApprovalState.pending,
    List<String>? attachments,
    List<Reaction>? reactions,
    this.replyTo,
    this.fresh = false,
  }) : attachments = attachments ?? [],
       reactions = reactions ?? [];

  final int id;
  final MsgKind kind;

  /// Body text; for files the content, for approvals the action.
  String text;
  String time;
  bool streaming;
  bool done;
  String? icon;
  String? link;
  int? routine;
  String? fileName;
  String? fileSize;
  String? placeholder;
  String? answer;
  String? detail;
  ApprovalState approval;
  List<String> attachments;
  List<Reaction> reactions;
  String? replyTo;

  /// Plays the arrival animation once.
  bool fresh;

  Map<String, dynamic> toJson() => {
    'id': id,
    'kind': kind.name,
    'text': text,
    'time': time,
    'done': done,
    'icon': icon,
    'link': link,
    'routine': routine,
    'fileName': fileName,
    'fileSize': fileSize,
    'placeholder': placeholder,
    'answer': answer,
    'detail': detail,
    'approval': approval.name,
    'attachments': attachments,
    'reactions': reactions.map((r) => r.name).toList(),
    'replyTo': replyTo,
  };

  factory Message.fromJson(Map<String, dynamic> j) => Message(
    id: j['id'],
    kind: MsgKind.values.byName(j['kind']),
    text: j['text'] ?? '',
    time: j['time'] ?? '',
    done: j['done'] ?? false,
    icon: j['icon'],
    link: j['link'],
    routine: j['routine'],
    fileName: j['fileName'],
    fileSize: j['fileSize'],
    placeholder: j['placeholder'],
    answer: j['answer'],
    detail: j['detail'],
    approval: ApprovalState.values.byName(j['approval'] ?? 'pending'),
    attachments: List<String>.from(j['attachments'] ?? const []),
    reactions: [for (final r in (j['reactions'] as List? ?? const [])) Reaction.values.byName(r)],
    replyTo: j['replyTo'],
  );
}

class ComputerState {
  String url = 'about:blank';
  String title = 'New Tab';
  String status = 'Idle';
  double x = 0.5, y = 0.55;
  String? typing;
  bool active = false;
}

class Bot {
  Bot({required this.id, required this.name, required this.color, required this.shape, this.title = '', this.description = ''});

  final int id;
  String name;
  String title;
  String description;
  int color;
  Shape shape;
  bool pinned = false;
  bool hidden = false;
  bool unread = false;
  BotStatus status = BotStatus.idle;
  List<Message> messages = [];
  List<Routine> routines = [];
  List<Skill> skills = [];
  String lastActivity = '';
  ComputerState computer = ComputerState();

  /// Whether the backend has been told who this Bot is yet.
  bool primed = false;

  String get preview {
    for (final m in messages.reversed) {
      switch (m.kind) {
        case MsgKind.bot when m.text.isNotEmpty:
          return m.text.replaceAll('\n', ' ');
        case MsgKind.user:
          return 'You: ${m.text.replaceAll('\n', ' ')}';
        case MsgKind.question:
          return m.text;
        case MsgKind.approval:
          return 'Needs approval: ${m.text}';
        case MsgKind.file:
          return 'Sent ${m.fileName}';
        default:
      }
    }
    return '';
  }

  String get lastTime {
    for (final m in messages.reversed) {
      if (m.time.isNotEmpty) return m.time;
    }
    return lastActivity;
  }

  Message? find(int id) {
    for (final m in messages) {
      if (m.id == id) return m;
    }
    return null;
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'title': title,
    'description': description,
    'color': color,
    'shape': shape.name,
    'pinned': pinned,
    'hidden': hidden,
    'unread': unread,
    'lastActivity': lastActivity,
    'messages': messages.map((m) => m.toJson()).toList(),
    'routines': routines.map((r) => r.toJson()).toList(),
    'skills': skills.map((s) => s.toJson()).toList(),
  };

  factory Bot.fromJson(Map<String, dynamic> j) {
    final b = Bot(
      id: j['id'],
      name: j['name'],
      color: j['color'],
      shape: Shape.values.byName(j['shape']),
      title: j['title'] ?? '',
      description: j['description'] ?? '',
    );
    b.pinned = j['pinned'] ?? false;
    b.hidden = j['hidden'] ?? false;
    b.unread = j['unread'] ?? false;
    b.lastActivity = j['lastActivity'] ?? '';
    b.messages = [for (final m in (j['messages'] as List? ?? const [])) Message.fromJson(m)];
    b.routines = [for (final r in (j['routines'] as List? ?? const [])) Routine.fromJson(r)];
    b.skills = [for (final s in (j['skills'] as List? ?? const [])) Skill.fromJson(s)];
    return b;
  }
}

class Plugin {
  Plugin(this.id, this.name, this.description, this.categories, this.glyph, this.color, {this.featured = false, this.team = false});
  final String id, name, description, glyph;
  final List<String> categories;
  final int color;
  final bool featured, team;
  bool installed = false;
}
