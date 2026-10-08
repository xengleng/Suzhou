// The conversation: header, messages and the composer.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../data.dart';
import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/computer_screen.dart';
import '../widgets/kit.dart';
import '../widgets/rich.dart';
import 'computer.dart';
import 'details.dart';
import 'file_viewer.dart';
import 'plugins.dart';
import 'routine.dart';

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key, required this.bot});
  final Bot bot;

  static Route<void> route(Bot bot) => MaterialPageRoute(builder: (_) => ChatScreen(bot: bot));

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  late final AppState _state = AppScope.read(context);
  final _input = TextEditingController();
  final _focus = FocusNode();
  final _answer = TextEditingController();
  final _attachments = <String>[];
  String? _replyTo;
  DateTime? _recording;
  Timer? _ticker;

  Bot get bot => widget.bot;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _state.opened(bot));
    _input.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _state.closed(bot);
    _ticker?.cancel();
    _input.dispose();
    _answer.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _send() {
    if (_input.text.trim().isEmpty && _attachments.isEmpty) return;
    HapticFeedback.lightImpact();
    _state.send(bot, _input.text, attachments: [..._attachments], replyTo: _replyTo);
    _input.clear();
    setState(() {
      _attachments.clear();
      _replyTo = null;
    });
  }

  void _toggleVoice() {
    if (_recording == null) {
      HapticFeedback.mediumImpact();
      setState(() => _recording = DateTime.now());
      _ticker = Timer.periodic(const Duration(seconds: 1), (_) => setState(() {}));
    } else {
      final secs = DateTime.now().difference(_recording!).inMilliseconds;
      _ticker?.cancel();
      setState(() => _recording = null);
      if (secs > 800) {
        final sep = _input.text.isEmpty || _input.text.endsWith(' ') ? '' : ' ';
        _input.text = '${_input.text}${sep}Can you check today\'s numbers and tell me what changed since yesterday?';
        _input.selection = TextSelection.collapsed(offset: _input.text.length);
      }
    }
  }

  void _cancelVoice() {
    _ticker?.cancel();
    setState(() => _recording = null);
  }

  // ----- @ and / suggestions --------------------------------------------------

  ({String kind, int start, String query})? get _token {
    final sel = _input.selection;
    final text = _input.text;
    final cursor = sel.isValid ? sel.baseOffset : text.length;
    if (cursor < 0 || cursor > text.length) return null;
    final before = text.substring(0, cursor);
    final start = before.lastIndexOf(RegExp(r'\s')) + 1;
    final token = before.substring(start);
    if (token.startsWith('@')) return (kind: '@', start: start, query: token.substring(1).toLowerCase());
    if (token.startsWith('/') && before.substring(0, start).trim().isEmpty) return (kind: '/', start: start, query: token.substring(1).toLowerCase());
    return null;
  }

  List<({String label, String detail, Widget lead, String insert})> _suggestions() {
    final t = _token;
    if (t == null) return const [];
    bool match(String s) => t.query.isEmpty || s.toLowerCase().contains(t.query);
    final p = context.p;
    final items = <({String label, String detail, Widget lead, String insert})>[];
    if (t.kind == '@') {
      for (final b in _state.bots.where((b) => b.id != bot.id && match(b.name))) {
        items.add((label: b.name, detail: b.title.isEmpty ? 'Bot' : b.title, lead: BotFace(shape: b.shape, color: b.color, size: 26), insert: '@${b.name} '));
      }
      for (final r in bot.routines.where((r) => match(r.name))) {
        items.add((label: r.name, detail: r.schedules.firstOrNull ?? '', lead: Icon(Icons.schedule_rounded, color: p.muted), insert: '@${r.name} '));
      }
      for (final pl in _state.pluginList.where((x) => x.installed && match(x.name))) {
        items.add((label: pl.name, detail: 'Connector', lead: GlyphTile(pl.glyph, pl.color, size: 26), insert: '@${pl.name} '));
      }
    } else {
      final skills = [...bot.skills, ...defaultSkills().where((d) => !bot.skills.any((s) => s.name == d.name))];
      for (final s in skills.where((s) => match(s.name))) {
        items.add((label: '/${s.name}', detail: s.description, lead: Icon(Icons.menu_book_rounded, color: p.muted), insert: '/${s.name} '));
      }
      if (match('teach')) {
        items.add((label: 'Teach a task', detail: 'Show the Bot how, once', lead: Icon(Icons.auto_awesome_outlined, color: p.muted), insert: ''));
      }
    }
    return items.take(7).toList();
  }

  void _pick(String insert) {
    final t = _token!;
    final cursor = _input.selection.baseOffset;
    if (insert.isEmpty) {
      _input.text = _input.text.replaceRange(t.start, cursor, '');
      showTeachSheet(context, bot);
      return;
    }
    _input.text = _input.text.replaceRange(t.start, cursor, insert);
    _input.selection = TextSelection.collapsed(offset: t.start + insert.length);
  }

  // ----- Build -------------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    AppScope.of(context); // rebuild on every state change
    final p = context.p;
    final working = bot.status == BotStatus.working;
    final items = bot.messages.reversed.toList();
    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: InkWell(
          borderRadius: BorderRadius.circular(10),
          onTap: () => Navigator.of(context).push(DetailsScreen.route(bot)),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 4),
            child: Row(
              children: [
                Hero(
                  tag: 'face-${bot.id}',
                  child: BotFace(shape: bot.shape, color: bot.color, size: 30, working: working, hop: 3),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(bot.name, maxLines: 1, overflow: TextOverflow.ellipsis),
                      AnimatedSwitcher(
                        duration: const Duration(milliseconds: 250),
                        child: Text(
                          working ? 'working…' : (bot.title.isEmpty ? 'Bot' : bot.title),
                          key: ValueKey(working),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 13, fontWeight: FontWeight.w400, color: working ? p.accent : p.muted),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        actions: [
          Stack(
            alignment: Alignment.center,
            children: [
              IconButton(
                tooltip: 'Agent Computer',
                icon: const Icon(Icons.desktop_windows_outlined),
                onPressed: () => Navigator.of(context).push(AgentComputerPage.route(bot)),
              ),
              if (bot.computer.active && working) Positioned(top: 12, right: 10, child: CircleAvatar(radius: 3.5, backgroundColor: p.online)),
            ],
          ),
          IconButton(tooltip: 'Details', icon: const Icon(Icons.more_horiz_rounded), onPressed: () => Navigator.of(context).push(DetailsScreen.route(bot))),
        ],
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(1),
          child: Divider(height: 1, color: p.border),
        ),
      ),
      body: Column(
        children: [
          Expanded(
            child: GestureDetector(
              onTap: () => FocusScope.of(context).unfocus(),
              child: ListView.builder(
                reverse: true,
                padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
                keyboardDismissBehavior: ScrollViewKeyboardDismissBehavior.onDrag,
                itemCount: items.length + 1,
                itemBuilder: (context, i) {
                  if (i == 0) {
                    return AnimatedSize(
                      duration: const Duration(milliseconds: 250),
                      child: working
                          ? Padding(
                              padding: const EdgeInsets.only(top: 10, left: 4),
                              child: Align(
                                alignment: Alignment.centerLeft,
                                child: BotFace(shape: bot.shape, color: bot.color, size: 30, working: true, hop: 6),
                              ),
                            )
                          : const SizedBox(width: double.infinity),
                    );
                  }
                  final m = items[i - 1];
                  final older = i < items.length ? items[i] : null;
                  final sameSide = older != null && older.kind == m.kind && const {MsgKind.bot, MsgKind.user, MsgKind.activity}.contains(m.kind);
                  return Padding(
                    padding: EdgeInsets.only(top: older == null ? 0 : (sameSide ? 5 : 14)),
                    child: Arrive(key: ValueKey(m.id), enabled: m.fresh, fromRight: m.kind == MsgKind.user, child: _message(m)),
                  );
                },
              ),
            ),
          ),
          _composer(),
        ],
      ),
    );
  }

  Widget _message(Message m) {
    final p = context.p;
    final maxW = MediaQuery.sizeOf(context).width * 0.8;
    switch (m.kind) {
      case MsgKind.stamp:
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(8),
            child: Text(m.text, style: TextStyle(fontSize: 13, color: p.muted)),
          ),
        );
      case MsgKind.event:
        return Center(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 16),
            child: InkWell(
              onTap: m.routine == null
                  ? null
                  : () {
                      final r = bot.routines.where((r) => r.id == m.routine).firstOrNull;
                      if (r != null) Navigator.of(context).push(RoutineScreen.route(bot, r));
                    },
              child: Text.rich(
                TextSpan(
                  style: TextStyle(fontSize: 13, color: p.muted),
                  children: [
                    TextSpan(text: m.text),
                    if (m.icon != null || m.link != null) const TextSpan(text: '  '),
                    if (m.icon != null)
                      WidgetSpan(
                        alignment: PlaceholderAlignment.middle,
                        child: Icon(iconFor(m.icon), size: 14, color: p.muted),
                      ),
                    if (m.link != null)
                      TextSpan(
                        text: ' ${m.link}',
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                  ],
                ),
                textAlign: TextAlign.center,
              ),
            ),
          ),
        );
      case MsgKind.activity:
        return Padding(
          padding: const EdgeInsets.only(left: 6),
          child: Row(
            children: [
              AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                transitionBuilder: (c, a) => ScaleTransition(scale: a, child: c),
                child: m.done ? Icon(Icons.check_rounded, key: const ValueKey(1), size: 16, color: p.online) : const SmallSpinner(key: ValueKey(0)),
              ),
              const SizedBox(width: 8),
              Expanded(child: Text.rich(inlineSpan(context, m.text, TextStyle(fontSize: 13.5, color: p.muted)))),
            ],
          ),
        );
      case MsgKind.user:
        return _bubbleRow(
          m,
          mine: true,
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              if (m.replyTo != null)
                Container(
                  margin: const EdgeInsets.only(bottom: 6),
                  padding: const EdgeInsets.only(left: 10),
                  decoration: BoxDecoration(
                    border: Border(left: BorderSide(color: p.borderStrong, width: 2)),
                  ),
                  child: Text(
                    m.replyTo!,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: 13, color: p.muted),
                  ),
                ),
              for (final a in m.attachments) Padding(padding: const EdgeInsets.only(bottom: 6), child: _AttachmentChip(a)),
              if (m.text.isNotEmpty)
                Container(
                  constraints: BoxConstraints(maxWidth: maxW),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 11),
                  decoration: BoxDecoration(color: p.userBubble, borderRadius: BorderRadius.circular(22)),
                  child: Markdown(m.text, color: p.userText),
                ),
            ],
          ),
        );
      case MsgKind.bot:
        return _bubbleRow(
          m,
          mine: false,
          Container(
            constraints: BoxConstraints(maxWidth: maxW),
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 11),
            decoration: BoxDecoration(color: p.botBubble, borderRadius: BorderRadius.circular(22)),
            child: AnimatedSize(
              duration: const Duration(milliseconds: 120),
              alignment: Alignment.topLeft,
              child: m.text.isEmpty && m.streaming ? const TypingDots() : Markdown(m.text),
            ),
          ),
        );
      case MsgKind.file:
        return Align(
          alignment: Alignment.centerLeft,
          child: Material(
            color: p.surface,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(14),
              side: BorderSide(color: p.border),
            ),
            child: InkWell(
              borderRadius: BorderRadius.circular(14),
              onTap: () => Navigator.of(context).push(FileViewer.route(m.fileName ?? 'file', m.text)),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(12, 10, 6, 10),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 42,
                      height: 42,
                      alignment: Alignment.center,
                      decoration: BoxDecoration(color: p.accent.withValues(alpha: 0.12), borderRadius: BorderRadius.circular(9)),
                      child: Text(
                        'M↓',
                        style: TextStyle(fontFamily: kMono, fontSize: 12, fontWeight: FontWeight.w600, color: p.accent),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(m.fileName ?? '', style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w500)),
                        Text(m.fileSize ?? '', style: TextStyle(fontSize: 13, color: p.muted)),
                      ],
                    ),
                    const SizedBox(width: 18),
                    IconButton(
                      icon: Icon(Icons.download_rounded, color: p.muted),
                      onPressed: () {
                        Clipboard.setData(ClipboardData(text: m.text));
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Copied ${m.fileName} to the clipboard')));
                      },
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      case MsgKind.question:
        final open = m.answer == null && bot.messages.lastWhere((x) => x.kind == MsgKind.question && x.answer == null, orElse: () => m) == m;
        return Align(
          alignment: Alignment.centerLeft,
          child: Container(
            constraints: BoxConstraints(maxWidth: maxW),
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
            decoration: BoxDecoration(color: p.botBubble, borderRadius: BorderRadius.circular(22)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Markdown(m.text),
                const SizedBox(height: 10),
                AnimatedSwitcher(
                  duration: const Duration(milliseconds: 250),
                  child: m.answer != null
                      ? Row(
                          key: const ValueKey('a'),
                          children: [
                            Icon(Icons.check_rounded, size: 16, color: p.online),
                            const SizedBox(width: 6),
                            Flexible(
                              child: Text(m.answer!, style: TextStyle(color: p.muted)),
                            ),
                          ],
                        )
                      : open
                      ? TextField(
                          key: const ValueKey('q'),
                          controller: _answer,
                          textInputAction: TextInputAction.send,
                          onSubmitted: (v) {
                            _state.answer(bot, m, v);
                            _answer.clear();
                          },
                          decoration: InputDecoration(
                            hintText: m.placeholder,
                            hintStyle: TextStyle(color: p.faint),
                            filled: true,
                            fillColor: p.bg,
                            isDense: true,
                            prefixIcon: Padding(
                              padding: const EdgeInsets.all(10),
                              child: Container(
                                width: 22,
                                alignment: Alignment.center,
                                decoration: BoxDecoration(color: p.field, borderRadius: BorderRadius.circular(5)),
                                child: Text('A', style: TextStyle(fontSize: 12, color: p.muted)),
                              ),
                            ),
                            suffixIcon: IconButton(
                              icon: const Icon(Icons.check_rounded),
                              onPressed: () {
                                _state.answer(bot, m, _answer.text);
                                _answer.clear();
                              },
                            ),
                            enabledBorder: OutlineInputBorder(
                              borderRadius: BorderRadius.circular(12),
                              borderSide: BorderSide(color: p.borderStrong),
                            ),
                            focusedBorder: OutlineInputBorder(
                              borderRadius: BorderRadius.circular(12),
                              borderSide: BorderSide(color: p.accent),
                            ),
                          ),
                        )
                      : Text(
                          'Waiting for an answer',
                          key: const ValueKey('w'),
                          style: TextStyle(color: p.faint),
                        ),
                ),
              ],
            ),
          ),
        );
      case MsgKind.approval:
        return Align(
          alignment: Alignment.centerLeft,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            constraints: BoxConstraints(maxWidth: maxW),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: p.surface,
              borderRadius: BorderRadius.circular(18),
              border: Border.all(color: m.approval == ApprovalState.pending ? p.warning.withValues(alpha: 0.6) : p.border),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(Icons.verified_user_outlined, size: 16, color: p.warning),
                    const SizedBox(width: 6),
                    Text('Needs your approval', style: TextStyle(fontSize: 13, color: p.muted)),
                  ],
                ),
                const SizedBox(height: 8),
                Text(m.text, style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w600)),
                const SizedBox(height: 6),
                Text(m.detail ?? '', style: TextStyle(fontSize: 15, height: 1.45, color: p.muted)),
                const SizedBox(height: 12),
                AnimatedSwitcher(
                  duration: const Duration(milliseconds: 250),
                  child: switch (m.approval) {
                    ApprovalState.pending => Row(
                      key: const ValueKey('p'),
                      children: [
                        SecondaryButton('Deny', onPressed: () => _state.decide(bot, m, false)),
                        const SizedBox(width: 8),
                        PrimaryButton(
                          'Approve',
                          onPressed: () {
                            HapticFeedback.mediumImpact();
                            _state.decide(bot, m, true);
                          },
                        ),
                      ],
                    ),
                    ApprovalState.approved => Row(
                      key: const ValueKey('a'),
                      children: [
                        Icon(Icons.check_rounded, size: 16, color: p.online),
                        const SizedBox(width: 6),
                        Text('Approved', style: TextStyle(color: p.muted)),
                      ],
                    ),
                    ApprovalState.denied => Row(
                      key: const ValueKey('d'),
                      children: [
                        Icon(Icons.close_rounded, size: 16, color: p.danger),
                        const SizedBox(width: 6),
                        Text('Denied', style: TextStyle(color: p.muted)),
                      ],
                    ),
                  },
                ),
              ],
            ),
          ),
        );
      case MsgKind.snapshot:
        final state = ComputerState()
          ..url = 'https://${m.link}'
          ..title = m.text;
        return Align(
          alignment: Alignment.centerLeft,
          child: GestureDetector(
            onTap: () => Navigator.of(context).push(AgentComputerPage.route(bot)),
            child: Container(
              width: maxW,
              clipBehavior: Clip.antiAlias,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(18),
                border: Border.all(color: p.border),
              ),
              child: ComputerScreen(state: state, live: false),
            ),
          ),
        );
    }
  }

  Widget _bubbleRow(Message m, Widget bubble, {required bool mine}) {
    final p = context.p;
    return GestureDetector(
      onLongPress: () {
        HapticFeedback.selectionClick();
        _messageActions(m);
      },
      child: Column(
        crossAxisAlignment: mine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
        children: [
          bubble,
          if (m.reactions.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Wrap(
                spacing: 4,
                children: [
                  for (final r in m.reactions)
                    TweenAnimationBuilder<double>(
                      key: ValueKey(r),
                      tween: Tween(begin: 0, end: 1),
                      duration: const Duration(milliseconds: 380),
                      curve: Curves.elasticOut,
                      builder: (context, s, child) => Transform.scale(scale: s, child: child),
                      child: GestureDetector(
                        onTap: () => _state.react(m, r),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                          decoration: BoxDecoration(
                            color: p.field,
                            borderRadius: BorderRadius.circular(12),
                            border: Border.all(color: p.border),
                          ),
                          child: Icon(_reactionIcon(r), size: 14, color: r == Reaction.heart ? p.danger : p.text),
                        ),
                      ),
                    ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  void _messageActions(Message m) {
    final p = context.p;
    showModalBottomSheet(
      context: context,
      builder: (sheet) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                for (final r in Reaction.values)
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 6),
                    child: IconButton.filledTonal(
                      iconSize: 24,
                      style: IconButton.styleFrom(backgroundColor: m.reactions.contains(r) ? p.accent.withValues(alpha: 0.2) : p.field),
                      icon: Icon(_reactionIcon(r), color: r == Reaction.heart ? p.danger : p.text),
                      onPressed: () {
                        Navigator.pop(sheet);
                        _state.react(m, r);
                      },
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 8),
            SheetRow(
              Icons.reply_rounded,
              'Reply',
              onTap: () {
                Navigator.pop(sheet);
                setState(() => _replyTo = m.text.split('\n').first);
                _focus.requestFocus();
              },
            ),
            SheetRow(
              Icons.copy_rounded,
              'Copy text',
              onTap: () {
                Navigator.pop(sheet);
                Clipboard.setData(ClipboardData(text: m.text));
              },
            ),
            SheetRow(
              Icons.menu_book_rounded,
              'Save the process as skill',
              onTap: () {
                Navigator.pop(sheet);
                _state.saveAsSkill(bot, m);
              },
            ),
            SheetRow(
              Icons.delete_outline_rounded,
              'Delete',
              danger: true,
              onTap: () {
                Navigator.pop(sheet);
                _state.deleteMessage(bot, m);
              },
            ),
            const SizedBox(height: 8),
          ],
        ),
      ),
    );
  }

  void _plusSheet() {
    FocusScope.of(context).unfocus();
    showModalBottomSheet(
      context: context,
      builder: (sheet) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
              child: Row(
                children: [
                  for (final (icon, label, name) in [
                    (Icons.insert_drive_file_outlined, 'Choose File', 'notes-${_attachments.length + 1}.pdf'),
                    (Icons.photo_camera_outlined, 'Take Photo', 'photo-${_attachments.length + 1}.jpg'),
                    (Icons.image_outlined, 'Attach Image', 'image-${_attachments.length + 1}.png'),
                  ])
                    Expanded(
                      child: InkWell(
                        borderRadius: BorderRadius.circular(16),
                        onTap: () {
                          Navigator.pop(sheet);
                          setState(() => _attachments.add(name));
                        },
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 12),
                          child: Column(
                            children: [
                              CircleAvatar(
                                radius: 26,
                                backgroundColor: context.p.field,
                                child: Icon(icon, color: context.p.text),
                              ),
                              const SizedBox(height: 8),
                              Text(label, style: const TextStyle(fontSize: 13)),
                            ],
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
            const Divider(),
            SheetRow(
              Icons.auto_awesome_outlined,
              'Teach a task',
              onTap: () {
                Navigator.pop(sheet);
                showTeachSheet(context, bot);
              },
            ),
            SheetRow(
              Icons.desktop_windows_outlined,
              'Agent Computer',
              onTap: () {
                Navigator.pop(sheet);
                Navigator.of(context).push(AgentComputerPage.route(bot));
              },
            ),
            SheetRow(
              Icons.alternate_email_rounded,
              'Mention a Bot or app',
              onTap: () {
                Navigator.pop(sheet);
                _input.text = '${_input.text}${_input.text.isEmpty || _input.text.endsWith(' ') ? '' : ' '}@';
                _input.selection = TextSelection.collapsed(offset: _input.text.length);
                _focus.requestFocus();
              },
            ),
            SheetRow(
              Icons.menu_book_rounded,
              'Use a skill',
              onTap: () {
                Navigator.pop(sheet);
                _input.text = '/';
                _input.selection = const TextSelection.collapsed(offset: 1);
                _focus.requestFocus();
              },
            ),
            SheetRow(
              Icons.power_outlined,
              'Plugins',
              onTap: () {
                Navigator.pop(sheet);
                Navigator.of(context).push(PluginsScreen.route());
              },
            ),
            const SizedBox(height: 8),
          ],
        ),
      ),
    );
  }

  Widget _composer() {
    final p = context.p;
    final working = bot.status == BotStatus.working;
    final empty = _input.text.trim().isEmpty && _attachments.isEmpty;
    final suggestions = _suggestions();

    Widget round(Widget icon, Color bg, VoidCallback onTap, String label, {Key? key}) => Semantics(
      key: key,
      button: true,
      label: label,
      child: Material(
        color: bg,
        shape: const CircleBorder(),
        child: InkWell(
          customBorder: const CircleBorder(),
          onTap: onTap,
          child: SizedBox(width: 40, height: 40, child: Center(child: icon)),
        ),
      ),
    );

    final trailing = _recording != null
        ? const SizedBox.shrink()
        : Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              AnimatedSwitcher(
                duration: const Duration(milliseconds: 220),
                transitionBuilder: (c, a) => ScaleTransition(
                  scale: CurvedAnimation(parent: a, curve: Curves.easeOutBack),
                  child: c,
                ),
                child: empty && working
                    ? round(
                        Container(
                          width: 13,
                          height: 13,
                          decoration: BoxDecoration(color: p.onPrimary, borderRadius: BorderRadius.circular(3)),
                        ),
                        p.primary,
                        () => _state.stop(bot),
                        'Stop',
                        key: const ValueKey('stop'),
                      )
                    : empty
                    ? round(Icon(Icons.mic_none_rounded, color: p.onPrimary), p.primary, _toggleVoice, 'Voice input', key: const ValueKey('mic'))
                    : Row(
                        key: const ValueKey('send'),
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          round(Icon(Icons.mic_none_rounded, color: p.text), p.field, _toggleVoice, 'Voice input'),
                          const SizedBox(width: 6),
                          round(Icon(Icons.arrow_upward_rounded, color: p.onPrimary), p.primary, _send, 'Send'),
                        ],
                      ),
              ),
            ],
          );

    final secs = _recording == null ? 0 : DateTime.now().difference(_recording!).inSeconds;

    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(10, 4, 10, 10),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            AnimatedSize(
              duration: const Duration(milliseconds: 220),
              curve: Curves.easeOutQuint,
              child: suggestions.isEmpty
                  ? const SizedBox(width: double.infinity)
                  : Container(
                      margin: const EdgeInsets.only(bottom: 8),
                      constraints: const BoxConstraints(maxHeight: 280),
                      decoration: BoxDecoration(
                        color: p.surface,
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: p.border),
                        boxShadow: const [BoxShadow(color: Color(0x1F000000), blurRadius: 24, offset: Offset(0, 8))],
                      ),
                      child: ListView(
                        shrinkWrap: true,
                        padding: const EdgeInsets.symmetric(vertical: 6),
                        children: [
                          for (final s in suggestions)
                            ListTile(
                              dense: true,
                              leading: SizedBox(width: 28, child: Center(child: s.lead)),
                              title: Text(s.label, style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w500)),
                              subtitle: Text(s.detail, maxLines: 1, overflow: TextOverflow.ellipsis),
                              onTap: () => _pick(s.insert),
                            ),
                        ],
                      ),
                    ),
            ),
            Container(
              padding: const EdgeInsets.all(6),
              decoration: BoxDecoration(
                color: p.surface,
                borderRadius: BorderRadius.circular(28),
                border: Border.all(color: p.borderStrong),
                boxShadow: const [BoxShadow(color: Color(0x0F000000), blurRadius: 10, offset: Offset(0, 2))],
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (_replyTo != null)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(10, 2, 0, 4),
                      child: Row(
                        children: [
                          Icon(Icons.reply_rounded, size: 16, color: p.muted),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              'Replying to “$_replyTo”',
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(fontSize: 13, color: p.muted),
                            ),
                          ),
                          IconButton(
                            visualDensity: VisualDensity.compact,
                            icon: const Icon(Icons.close_rounded, size: 18),
                            onPressed: () => setState(() => _replyTo = null),
                          ),
                        ],
                      ),
                    ),
                  if (_attachments.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(6, 2, 6, 6),
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: Wrap(
                          spacing: 6,
                          runSpacing: 6,
                          children: [
                            for (final a in _attachments)
                              GestureDetector(onTap: () => setState(() => _attachments.remove(a)), child: _AttachmentChip(a, removable: true)),
                          ],
                        ),
                      ),
                    ),
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      round(Icon(Icons.add_rounded, color: p.text), p.field, _plusSheet, 'Add'),
                      const SizedBox(width: 8),
                      Expanded(
                        child: AnimatedSwitcher(
                          duration: const Duration(milliseconds: 250),
                          child: _recording != null
                              ? _VoiceBar(key: const ValueKey('voice'), seconds: secs, onCancel: _cancelVoice, onDone: _toggleVoice)
                              : TextField(
                                  key: const ValueKey('text'),
                                  controller: _input,
                                  focusNode: _focus,
                                  minLines: 1,
                                  maxLines: 6,
                                  textCapitalization: TextCapitalization.sentences,
                                  style: const TextStyle(fontSize: 16, height: 1.4),
                                  decoration: InputDecoration(
                                    hintText: 'Message ${bot.name}',
                                    hintStyle: TextStyle(color: p.faint),
                                    border: InputBorder.none,
                                    isDense: true,
                                    contentPadding: const EdgeInsets.symmetric(vertical: 10),
                                  ),
                                ),
                        ),
                      ),
                      const SizedBox(width: 6),
                      trailing,
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

IconData _reactionIcon(Reaction r) => switch (r) {
  Reaction.thumbsUp => Icons.thumb_up_alt_outlined,
  Reaction.heart => Icons.favorite_border_rounded,
  Reaction.check => Icons.check_rounded,
  Reaction.party => Icons.celebration_outlined,
};

class _AttachmentChip extends StatelessWidget {
  const _AttachmentChip(this.name, {this.removable = false});
  final String name;
  final bool removable;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
    decoration: BoxDecoration(color: context.p.field, borderRadius: BorderRadius.circular(9)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.attach_file_rounded, size: 14, color: context.p.muted),
        const SizedBox(width: 4),
        Text(name, style: const TextStyle(fontSize: 13)),
        if (removable) ...[const SizedBox(width: 4), Icon(Icons.close_rounded, size: 14, color: context.p.muted)],
      ],
    ),
  );
}

/// Recording: a pulsing dot, the time, and a live waveform.
class _VoiceBar extends StatefulWidget {
  const _VoiceBar({super.key, required this.seconds, required this.onCancel, required this.onDone});
  final int seconds;
  final VoidCallback onCancel, onDone;

  @override
  State<_VoiceBar> createState() => _VoiceBarState();
}

class _VoiceBarState extends State<_VoiceBar> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 900))..repeat();

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    return SizedBox(
      height: 40,
      child: Row(
        children: [
          FadeTransition(
            opacity: Tween(begin: 0.3, end: 1.0).animate(_c),
            child: CircleAvatar(radius: 5, backgroundColor: p.danger),
          ),
          const SizedBox(width: 8),
          Text(
            '${widget.seconds ~/ 60}:${(widget.seconds % 60).toString().padLeft(2, '0')}',
            style: TextStyle(fontFamily: kMono, color: p.muted),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: AnimatedBuilder(
              animation: _c,
              builder: (context, _) => Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  for (var i = 0; i < 22; i++)
                    Container(
                      margin: const EdgeInsets.symmetric(horizontal: 1.5),
                      width: 3,
                      height: 4 + 18 * (0.25 + 0.75 * ((i * 37) % 11) / 11) * (0.5 + 0.5 * _wave(_c.value, i)),
                      decoration: BoxDecoration(color: p.text, borderRadius: BorderRadius.circular(2)),
                    ),
                ],
              ),
            ),
          ),
          IconButton(icon: const Icon(Icons.close_rounded), onPressed: widget.onCancel),
          Material(
            color: p.primary,
            shape: const CircleBorder(),
            child: InkWell(
              customBorder: const CircleBorder(),
              onTap: widget.onDone,
              child: SizedBox(width: 40, height: 40, child: Icon(Icons.check_rounded, color: p.onPrimary)),
            ),
          ),
        ],
      ),
    );
  }

  double _wave(double t, int i) {
    final x = (t + i * 0.13) % 1.0;
    return (x < 0.5 ? x * 2 : (1 - x) * 2);
  }
}

/// Teach a task: record a demonstration on the computer, save it as a skill.
Future<void> showTeachSheet(BuildContext context, Bot bot) {
  final started = DateTime.now();
  return showModalBottomSheet(
    context: context,
    isDismissible: false,
    builder: (sheet) => StatefulBuilder(
      builder: (context, setSheet) {
        Future<void>.delayed(const Duration(seconds: 1), () {
          if (sheet.mounted) setSheet(() {});
        });
        final secs = DateTime.now().difference(started).inSeconds;
        final p = context.p;
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.fromLTRB(24, 0, 24, 20),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                CircleAvatar(
                  radius: 32,
                  backgroundColor: p.danger.withValues(alpha: 0.12),
                  child: CircleAvatar(radius: 11, backgroundColor: p.danger),
                ),
                const SizedBox(height: 14),
                const Text('Teach a task', style: TextStyle(fontSize: 22, fontWeight: FontWeight.w600)),
                const SizedBox(height: 8),
                Text(
                  'Do the task once on the computer while ${bot.name} watches. It saves the steps as a skill it can repeat. Up to 10 minutes; the microphone is off.',
                  textAlign: TextAlign.center,
                  style: TextStyle(fontSize: 15, height: 1.5, color: p.muted),
                ),
                const SizedBox(height: 14),
                Text('${secs ~/ 60}:${(secs % 60).toString().padLeft(2, '0')} / 10:00', style: const TextStyle(fontFamily: kMono, fontSize: 26)),
                const SizedBox(height: 18),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    SecondaryButton('Cancel', onPressed: () => Navigator.pop(sheet)),
                    const SizedBox(width: 10),
                    PrimaryButton(
                      'Stop and save skill',
                      onPressed: () {
                        Navigator.pop(sheet);
                        AppScope.read(context).teach(bot, secs);
                      },
                    ),
                  ],
                ),
              ],
            ),
          ),
        );
      },
    ),
  );
}
