// Plugins: search, categories, Featured and Team sections, Add.

import 'package:flutter/material.dart';

import '../data.dart';
import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/kit.dart';

class PluginsScreen extends StatefulWidget {
  const PluginsScreen({super.key});

  static Route<void> route() => MaterialPageRoute(builder: (_) => const PluginsScreen(), fullscreenDialog: true);

  @override
  State<PluginsScreen> createState() => _PluginsScreenState();
}

class _PluginsScreenState extends State<PluginsScreen> {
  final _search = TextEditingController();
  int _category = 0;

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  bool _matches(Plugin pl) {
    final q = _search.text.toLowerCase();
    final cat = pluginCategories[_category];
    final inCat = switch (cat) {
      'All' => true,
      'Featured' => pl.featured,
      'Team plugins' => pl.team,
      _ => pl.categories.contains(cat),
    };
    return inCat && (q.isEmpty || pl.name.toLowerCase().contains(q) || pl.description.toLowerCase().contains(q));
  }

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final installed = state.pluginList.where((x) => x.installed).toList();
    final sections = [
      ('Featured', state.pluginList.where((x) => x.featured && _matches(x)).toList()),
      ('Team plugins', state.pluginList.where((x) => x.team && _matches(x)).toList()),
      ('More', state.pluginList.where((x) => !x.featured && !x.team && _matches(x)).toList()),
    ];
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(tooltip: 'Close', icon: const Icon(Icons.close_rounded), onPressed: () => Navigator.pop(context)),
        title: const Text('Plugins'),
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 4, 16, 40),
        children: [
          Row(
            children: [
              SizedBox(
                width: 30.0 + 24 * (installed.length.clamp(1, 4) - 1),
                height: 32,
                child: Stack(
                  children: [
                    for (final (i, pl) in installed.take(4).indexed)
                      Positioned(
                        left: 24.0 * i,
                        child: Container(
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(9),
                            border: Border.all(color: p.bg, width: 2),
                          ),
                          child: GlyphTile(pl.glyph, pl.color, size: 28),
                        ),
                      ),
                  ],
                ),
              ),
              const SizedBox(width: 10),
              AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                transitionBuilder: (c, a) => SlideTransition(
                  position: Tween(begin: const Offset(0, 0.4), end: Offset.zero).animate(a),
                  child: FadeTransition(opacity: a, child: c),
                ),
                child: Text(
                  '${installed.length} installed',
                  key: ValueKey(installed.length),
                  style: TextStyle(fontSize: 15, color: p.muted),
                ),
              ),
            ],
          ),
          const SizedBox(height: 14),
          TextField(
            controller: _search,
            onChanged: (_) => setState(() {}),
            decoration: InputDecoration(
              hintText: 'Search plugins',
              hintStyle: TextStyle(color: p.muted),
              prefixIcon: Icon(Icons.search_rounded, color: p.muted),
              filled: true,
              fillColor: p.field,
              isDense: true,
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide.none),
            ),
          ),
          const SizedBox(height: 12),
          SizedBox(
            height: 36,
            child: ListView.separated(
              scrollDirection: Axis.horizontal,
              itemCount: pluginCategories.length,
              separatorBuilder: (_, _) => const SizedBox(width: 8),
              itemBuilder: (context, i) {
                final selected = i == _category;
                return GestureDetector(
                  onTap: () => setState(() => _category = i),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: selected ? p.field : Colors.transparent,
                      borderRadius: BorderRadius.circular(9),
                      border: Border.all(color: selected ? p.field : p.border),
                    ),
                    child: Text(pluginCategories[i], style: const TextStyle(fontSize: 14)),
                  ),
                );
              },
            ),
          ),
          for (final (title, list) in sections)
            if (list.isNotEmpty) ...[SectionLabel(title), for (final pl in list) _PluginRow(pl)],
          if (sections.every((s) => s.$2.isEmpty))
            Padding(
              padding: const EdgeInsets.all(40),
              child: Center(
                child: Text('No plugins match', style: TextStyle(color: p.muted)),
              ),
            ),
        ],
      ),
    );
  }
}

class _PluginRow extends StatelessWidget {
  const _PluginRow(this.pl);
  final Plugin pl;

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final connecting = state.connecting.contains(pl.id);
    final Widget button = connecting
        ? Row(
            key: const ValueKey('c'),
            mainAxisSize: MainAxisSize.min,
            children: [
              const SmallSpinner(),
              const SizedBox(width: 6),
              Text('Connecting', style: TextStyle(color: p.muted, fontSize: 14)),
            ],
          )
        : pl.installed
        ? TextButton.icon(
            key: const ValueKey('i'),
            onPressed: () => state.uninstall(pl),
            icon: Icon(Icons.check_rounded, size: 16, color: p.online),
            label: Text('Added', style: TextStyle(color: p.muted)),
          )
        : FilledButton.tonal(
            key: const ValueKey('a'),
            style: FilledButton.styleFrom(backgroundColor: p.field, foregroundColor: p.text),
            onPressed: () => state.install(pl),
            child: const Text('Add'),
          );
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          pl.team
              ? Container(
                  width: 46,
                  height: 46,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(color: p.field, borderRadius: BorderRadius.circular(11)),
                  child: Text(pl.glyph, style: TextStyle(fontSize: 17, color: p.muted)),
                )
              : GlyphTile(pl.glyph, pl.color, size: 46),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Flexible(
                      child: Text(
                        pl.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
                      ),
                    ),
                    if (pl.team) ...[
                      const SizedBox(width: 6),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(color: p.field, borderRadius: BorderRadius.circular(10)),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.group_outlined, size: 12, color: p.muted),
                            const SizedBox(width: 3),
                            Text('Team', style: TextStyle(fontSize: 12, color: p.muted)),
                          ],
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 2),
                Text(
                  pl.description,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(fontSize: 14, color: p.muted, height: 1.35),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 250),
            transitionBuilder: (c, a) => ScaleTransition(
              scale: CurvedAnimation(parent: a, curve: Curves.easeOutBack),
              child: c,
            ),
            child: button,
          ),
        ],
      ),
    );
  }
}
