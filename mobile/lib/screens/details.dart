// Bot details: profile, routines, skills, files and housekeeping.

import 'package:flutter/material.dart';

import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/kit.dart';
import 'chat.dart';
import 'computer.dart';
import 'file_viewer.dart';
import 'home.dart';
import 'routine.dart';

class DetailsScreen extends StatefulWidget {
  const DetailsScreen({super.key, required this.bot, this.editing = false});
  final Bot bot;
  final bool editing;

  static Route<void> route(Bot bot, {bool editing = false}) => MaterialPageRoute(
    builder: (_) => DetailsScreen(bot: bot, editing: editing),
  );

  @override
  State<DetailsScreen> createState() => _DetailsScreenState();
}

class _DetailsScreenState extends State<DetailsScreen> {
  late bool _editing = widget.editing;
  late final _name = TextEditingController(text: widget.bot.name);
  late final _title = TextEditingController(text: widget.bot.title);
  late final _description = TextEditingController(text: widget.bot.description);

  Bot get bot => widget.bot;

  @override
  void dispose() {
    _name.dispose();
    _title.dispose();
    _description.dispose();
    super.dispose();
  }

  void _commit() {
    final state = AppScope.read(context);
    if (_name.text.trim().isNotEmpty) bot.name = _name.text.trim();
    bot.title = _title.text.trim();
    bot.description = _description.text.trim();
    state.save();
  }

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final files = bot.messages.where((m) => m.kind == MsgKind.file).toList();
    return Scaffold(
      appBar: AppBar(
        title: const Text('Details'),
        actions: [
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 200),
            child: _editing
                ? TextButton(
                    key: const ValueKey('done'),
                    onPressed: () {
                      _commit();
                      setState(() => _editing = false);
                    },
                    child: const Text('Done', style: TextStyle(fontWeight: FontWeight.w600)),
                  )
                : TextButton(key: const ValueKey('edit'), onPressed: () => setState(() => _editing = true), child: const Text('Edit')),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 40),
        children: [
          Center(
            child: Hero(
              tag: 'face-${bot.id}',
              child: BotFace(shape: bot.shape, color: bot.color, size: 84),
            ),
          ),
          const SizedBox(height: 12),
          AnimatedCrossFade(
            duration: const Duration(milliseconds: 300),
            sizeCurve: Curves.easeOutQuint,
            crossFadeState: _editing ? CrossFadeState.showSecond : CrossFadeState.showFirst,
            firstChild: Column(
              children: [
                Text(bot.name, style: const TextStyle(fontSize: 22, fontWeight: FontWeight.w600)),
                const SizedBox(height: 4),
                Text(bot.title.isEmpty ? 'No job title yet' : bot.title, style: TextStyle(fontSize: 15, color: p.muted)),
                if (bot.description.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 10),
                    child: Text(
                      bot.description,
                      textAlign: TextAlign.center,
                      style: TextStyle(fontSize: 14, height: 1.5, color: p.muted),
                    ),
                  ),
                const SizedBox(height: 14),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    SecondaryButton('Message', onPressed: () => Navigator.pop(context)),
                    const SizedBox(width: 8),
                    SecondaryButton('Computer', onPressed: () => Navigator.of(context).push(AgentComputerPage.route(bot))),
                  ],
                ),
              ],
            ),
            secondChild: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const SectionLabel('Name'),
                Field(controller: _name),
                const SectionLabel('Job title'),
                Field(controller: _title, hint: 'Job title'),
                const SectionLabel('Description'),
                Field(controller: _description, hint: 'What this Bot does, in a few sentences', maxLines: 6),
                const SectionLabel('Colour'),
                Wrap(
                  spacing: 6,
                  runSpacing: 6,
                  children: [
                    for (var i = 0; i < botColors.length; i++)
                      GestureDetector(
                        onTap: () {
                          bot.color = i;
                          state.save();
                        },
                        child: AnimatedContainer(
                          duration: const Duration(milliseconds: 200),
                          width: 34,
                          height: 34,
                          padding: const EdgeInsets.all(3),
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            border: Border.all(color: i == bot.color ? p.text : Colors.transparent, width: 2),
                          ),
                          child: CircleAvatar(backgroundColor: botColor(i)),
                        ),
                      ),
                  ],
                ),
                const SectionLabel('Shape'),
                Wrap(
                  spacing: 4,
                  runSpacing: 4,
                  children: [
                    for (final s in Shape.values)
                      GestureDetector(
                        onTap: () {
                          bot.shape = s;
                          state.save();
                        },
                        child: AnimatedContainer(
                          duration: const Duration(milliseconds: 200),
                          width: 44,
                          height: 44,
                          alignment: Alignment.center,
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(10),
                            border: Border.all(color: s == bot.shape ? p.accent : Colors.transparent, width: 2),
                          ),
                          child: CustomPaint(size: const Size.square(28), painter: FacePainter(s, botColor(bot.color))),
                        ),
                      ),
                  ],
                ),
              ],
            ),
          ),
          SectionLabel('Routines · ${bot.routines.length}'),
          for (final r in bot.routines)
            ListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 6),
              leading: Icon(Icons.schedule_rounded, color: r.active ? p.text : p.faint),
              title: Text(r.name),
              subtitle: Text(r.active ? (r.schedules.firstOrNull ?? '') : 'Paused'),
              trailing: Icon(Icons.chevron_right_rounded, color: p.faint),
              onTap: () => Navigator.of(context).push(RoutineScreen.route(bot, r)),
            ),
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 6),
            leading: Icon(Icons.add_rounded, color: p.muted),
            title: Text('New routine', style: TextStyle(color: p.muted)),
            onTap: () {
              final r = state.newRoutine(bot);
              Navigator.of(context).push(RoutineScreen.route(bot, r));
            },
          ),
          SectionLabel('Skills · ${bot.skills.length}'),
          for (final s in bot.skills)
            ListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 6),
              leading: Icon(Icons.menu_book_rounded, color: p.muted),
              title: Text('/${s.name}'),
              subtitle: Text(s.description, maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 6),
            leading: Icon(Icons.auto_awesome_outlined, color: p.muted),
            title: Text('Teach a task', style: TextStyle(color: p.muted)),
            onTap: () => showTeachSheet(context, bot),
          ),
          SectionLabel('Files · ${files.length}'),
          for (final f in files)
            ListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 6),
              leading: Icon(Icons.description_outlined, color: p.muted),
              title: Text(f.fileName ?? ''),
              trailing: Text(f.fileSize ?? '', style: TextStyle(color: p.faint)),
              onTap: () => Navigator.of(context).push(FileViewer.route(f.fileName ?? 'file', f.text)),
            ),
          const SizedBox(height: 24),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            alignment: WrapAlignment.center,
            children: [
              SecondaryButton(bot.pinned ? 'Unpin' : 'Pin', onPressed: () => state.togglePin(bot)),
              SecondaryButton(
                'Duplicate',
                onPressed: () {
                  final copy = state.duplicate(bot);
                  Navigator.of(context).popUntil((r) => r.isFirst);
                  Navigator.of(context).push(ChatScreen.route(copy));
                },
              ),
              SecondaryButton(bot.hidden ? 'Unhide' : 'Hide', onPressed: () => state.toggleHidden(bot)),
              SecondaryButton(
                'Delete',
                danger: true,
                onPressed: () async {
                  final nav = Navigator.of(context);
                  if (await confirmDelete(context, bot)) nav.popUntil((r) => r.isFirst);
                },
              ),
            ],
          ),
        ],
      ),
    );
  }
}
