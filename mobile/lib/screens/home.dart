// The roster: your Bots, pinned first, with their faces and status.

import 'package:flutter/material.dart';

import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/kit.dart';
import 'chat.dart';
import 'details.dart';
import 'new_bot.dart';
import 'plugins.dart';
import 'settings.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});
  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _search = TextEditingController();

  @override
  void initState() {
    super.initState();
    // A brand-new account goes straight to making its first Bot.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && AppScope.read(context).bots.isEmpty) {
        Navigator.of(context).push(NewBotScreen.route(first: true));
      }
    });
  }

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final list = state.roster(_search.text);
    return Scaffold(
      body: SafeArea(
        bottom: false,
        child: CustomScrollView(
          slivers: [
            SliverAppBar(
              pinned: true,
              expandedHeight: 104,
              backgroundColor: p.bg,
              leading: Padding(
                padding: const EdgeInsets.only(left: 12),
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () => Navigator.of(context).push(SettingsScreen.route()),
                  child: Semantics(
                    button: true,
                    label: 'Settings',
                    child: Center(child: UserAvatar(state.userName, size: 32)),
                  ),
                ),
              ),
              actions: [
                IconButton(tooltip: 'Plugins', icon: const Icon(Icons.power_outlined), onPressed: () => Navigator.of(context).push(PluginsScreen.route())),
                IconButton(
                  tooltip: 'New Bot',
                  icon: const Icon(Icons.add_rounded, size: 28),
                  onPressed: () => Navigator.of(context).push(NewBotScreen.route()),
                ),
                const SizedBox(width: 6),
              ],
              flexibleSpace: FlexibleSpaceBar(
                titlePadding: const EdgeInsetsDirectional.only(start: 20, bottom: 12),
                title: Text(
                  'Bots',
                  style: TextStyle(fontWeight: FontWeight.w600, color: p.text),
                ),
              ),
            ),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 4, 16, 10),
                child: TextField(
                  controller: _search,
                  onChanged: (_) => setState(() {}),
                  decoration: InputDecoration(
                    hintText: 'Search',
                    hintStyle: TextStyle(color: p.muted),
                    prefixIcon: Icon(Icons.search_rounded, color: p.muted),
                    filled: true,
                    fillColor: p.field,
                    isDense: true,
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide.none),
                  ),
                ),
              ),
            ),
            if (state.bots.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: Column(
                  children: [
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 10),
                      child: _Row(
                        leading: const BotFace(shape: Shape.circle, color: 6, size: 46),
                        title: 'Create your first Bot',
                        onTap: () => Navigator.of(context).push(NewBotScreen.route(first: true)),
                      ),
                    ),
                    Expanded(
                      child: Center(
                        child: Text('No chats yet', style: TextStyle(fontSize: 17, color: p.muted)),
                      ),
                    ),
                  ],
                ),
              )
            else if (list.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: Center(
                  child: Text('No matches', style: TextStyle(color: p.muted)),
                ),
              )
            else
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 10),
                sliver: SliverList.builder(
                  itemCount: list.length,
                  itemBuilder: (context, i) => Arrive(
                    delay: Duration(milliseconds: 40 * i),
                    child: BotTile(bot: list[i]),
                  ),
                ),
              ),
            if (state.hiddenCount > 0)
              SliverToBoxAdapter(
                child: Center(
                  child: TextButton(
                    onPressed: () {
                      state.showHidden = !state.showHidden;
                      state.save();
                    },
                    child: Text(state.showHidden ? 'Hide hidden Bots' : '${state.hiddenCount} hidden · Show', style: TextStyle(color: p.muted)),
                  ),
                ),
              ),
            const SliverToBoxAdapter(child: SizedBox(height: 40)),
          ],
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row({required this.leading, required this.title, this.onTap});
  final Widget leading;
  final String title;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Material(
    color: context.p.selected,
    borderRadius: BorderRadius.circular(16),
    child: InkWell(
      borderRadius: BorderRadius.circular(16),
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            leading,
            const SizedBox(width: 14),
            Expanded(
              child: Text(
                title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w500),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

class BotTile extends StatelessWidget {
  const BotTile({super.key, required this.bot});
  final Bot bot;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    final working = bot.status == BotStatus.working;
    String preview = bot.preview;
    if (working) {
      final open = bot.messages.reversed.where((m) => m.kind == MsgKind.activity && !m.done);
      preview = open.isNotEmpty ? '${open.first.text}…' : 'Working…';
    }
    return Opacity(
      opacity: bot.hidden ? 0.55 : 1,
      child: InkWell(
        borderRadius: BorderRadius.circular(16),
        onTap: () => Navigator.of(context).push(ChatScreen.route(bot)),
        onLongPress: () => showBotActions(context, bot),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 12),
          child: Row(
            children: [
              Hero(
                tag: 'face-${bot.id}',
                child: SizedBox(
                  width: 50,
                  height: 50,
                  child: Stack(
                    clipBehavior: Clip.none,
                    children: [
                      BotFace(shape: bot.shape, color: bot.color, size: 50, working: working),
                      Positioned(
                        right: -1,
                        bottom: 1,
                        child: StatusDot(status: bot.status, ring: p.bg),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Text(
                            bot.name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 17, fontWeight: bot.unread ? FontWeight.w600 : FontWeight.w500),
                          ),
                        ),
                        if (bot.pinned)
                          Padding(
                            padding: const EdgeInsets.only(right: 4),
                            child: Icon(Icons.push_pin_outlined, size: 14, color: p.faint),
                          ),
                        Text(bot.lastTime, style: TextStyle(fontSize: 13, color: p.muted)),
                      ],
                    ),
                    const SizedBox(height: 3),
                    Row(
                      children: [
                        Expanded(
                          child: AnimatedSwitcher(
                            duration: const Duration(milliseconds: 250),
                            child: Text(
                              preview,
                              key: ValueKey(preview),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(fontSize: 15, color: bot.unread ? p.text : p.muted),
                            ),
                          ),
                        ),
                        AnimatedScale(
                          scale: bot.unread ? 1 : 0,
                          duration: const Duration(milliseconds: 250),
                          curve: Curves.easeOutBack,
                          child: Padding(
                            padding: const EdgeInsets.only(left: 8),
                            child: CircleAvatar(radius: 4.5, backgroundColor: p.accent),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Long-press menu for a Bot: profile, pin, duplicate, hide, delete.
Future<void> showBotActions(BuildContext context, Bot bot) {
  final state = AppScope.read(context);
  return showModalBottomSheet(
    context: context,
    builder: (sheet) => SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                BotFace(shape: bot.shape, color: bot.color, size: 28),
                const SizedBox(width: 10),
                Text(bot.name, style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w600)),
              ],
            ),
          ),
          SheetRow(
            Icons.person_outline,
            'Edit profile',
            onTap: () {
              Navigator.pop(sheet);
              Navigator.of(context).push(DetailsScreen.route(bot, editing: true));
            },
          ),
          SheetRow(
            Icons.push_pin_outlined,
            bot.pinned ? 'Unpin' : 'Pin to top',
            onTap: () {
              Navigator.pop(sheet);
              state.togglePin(bot);
            },
          ),
          SheetRow(
            Icons.copy_rounded,
            'Duplicate',
            onTap: () {
              Navigator.pop(sheet);
              state.duplicate(bot);
            },
          ),
          SheetRow(
            Icons.visibility_off_outlined,
            bot.hidden ? 'Unhide' : 'Hide',
            onTap: () {
              Navigator.pop(sheet);
              state.toggleHidden(bot);
            },
          ),
          SheetRow(
            Icons.delete_outline_rounded,
            'Delete…',
            danger: true,
            onTap: () {
              Navigator.pop(sheet);
              confirmDelete(context, bot);
            },
          ),
          const SizedBox(height: 8),
        ],
      ),
    ),
  );
}

Future<bool> confirmDelete(BuildContext context, Bot bot) async {
  final ok = await showDialog<bool>(
    context: context,
    builder: (d) => AlertDialog(
      title: Text('Delete ${bot.name}?'),
      content: const Text('Its conversation, routines and skills are removed. Files it saved on the computer stay.'),
      actions: [
        TextButton(onPressed: () => Navigator.pop(d, false), child: const Text('Cancel')),
        TextButton(
          onPressed: () => Navigator.pop(d, true),
          child: Text('Delete', style: TextStyle(color: context.p.danger)),
        ),
      ],
    ),
  );
  if (ok == true && context.mounted) AppScope.read(context).deleteBot(bot);
  return ok == true;
}

/// Your own avatar: a circle with your initial.
class UserAvatar extends StatelessWidget {
  const UserAvatar(this.name, {super.key, this.size = 32});
  final String name;
  final double size;

  @override
  Widget build(BuildContext context) => CircleAvatar(
    radius: size / 2,
    backgroundColor: Color.lerp(context.p.accent, context.p.text, 0.25),
    child: Text(
      name.isEmpty ? '?' : name[0].toUpperCase(),
      style: TextStyle(color: Colors.white, fontSize: size * 0.44, fontWeight: FontWeight.w600),
    ),
  );
}
