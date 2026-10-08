// "New Bot": pick a colour, a shape and a name.

import 'package:flutter/material.dart';

import '../data.dart';
import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/kit.dart';
import 'chat.dart';

class NewBotScreen extends StatefulWidget {
  const NewBotScreen({super.key, this.first = false});
  final bool first;

  static Route<void> route({bool first = false}) => MaterialPageRoute(builder: (_) => NewBotScreen(first: first), fullscreenDialog: true);

  @override
  State<NewBotScreen> createState() => _NewBotScreenState();
}

class _NewBotScreenState extends State<NewBotScreen> {
  final _name = TextEditingController();
  late int _color;
  late Shape _shape;
  int _bump = 0;
  (String, String)? _role;

  @override
  void initState() {
    super.initState();
    final n = AppScope.read(context).bots.length;
    _color = (n * 3 + 6) % botColors.length;
    _shape = Shape.values[n % Shape.values.length];
  }

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  void _create() {
    final name = _name.text.trim();
    if (name.isEmpty) return;
    final bot = AppScope.read(context).createBot(name, _color, _shape, title: _role?.$1 ?? '', description: _role?.$2 ?? '');
    Navigator.of(context).pushReplacement(ChatScreen.route(bot));
  }

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    final ready = _name.text.trim().isNotEmpty;
    // Keep each picker on one row, even on a narrow phone.
    final avail = MediaQuery.sizeOf(context).width - 40;
    final swatch = ((avail - 2 * 9) / 10).clamp(26.0, 40.0);
    final tile = ((avail - 2 * 7) / 8).clamp(32.0, 48.0);
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(tooltip: 'Close', icon: const Icon(Icons.close_rounded), onPressed: () => Navigator.pop(context)),
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            BotFace(shape: _shape, color: _color, size: 24),
            const SizedBox(width: 10),
            Text(ready ? _name.text.trim() : (widget.first ? 'Your first Bot' : 'New Bot')),
          ],
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(20, 24, 20, 32),
        children: [
          Center(
            // Springs each time you change the colour or shape.
            child: TweenAnimationBuilder<double>(
              key: ValueKey(_bump),
              tween: Tween(begin: 0.8, end: 1),
              duration: const Duration(milliseconds: 600),
              curve: Curves.elasticOut,
              builder: (context, s, child) => Transform.scale(scale: s, child: child),
              child: BotFace(shape: _shape, color: _color, size: 112),
            ),
          ),
          const SizedBox(height: 28),
          Wrap(
            alignment: WrapAlignment.center,
            spacing: 2,
            runSpacing: 4,
            children: [
              for (var i = 0; i < botColors.length; i++)
                GestureDetector(
                  onTap: () => setState(() {
                    _color = i;
                    _bump++;
                  }),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: swatch,
                    height: swatch,
                    padding: const EdgeInsets.all(3),
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      border: Border.all(color: i == _color ? p.accent : Colors.transparent, width: 2),
                    ),
                    child: CircleAvatar(backgroundColor: botColor(i)),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 12),
          Wrap(
            alignment: WrapAlignment.center,
            spacing: 2,
            runSpacing: 4,
            children: [
              for (final s in Shape.values)
                GestureDetector(
                  onTap: () => setState(() {
                    _shape = s;
                    _bump++;
                  }),
                  child: AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    width: tile,
                    height: tile,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: s == _shape ? p.accent : Colors.transparent, width: 2),
                    ),
                    child: CustomPaint(size: Size.square(tile * 0.62), painter: FacePainter(s, botColor(_color))),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 30),
          Padding(
            padding: const EdgeInsets.only(left: 6, bottom: 8),
            child: Text('Name', style: TextStyle(fontSize: 15, color: p.muted)),
          ),
          Field(controller: _name, hint: 'New Bot', big: true, autofocus: !widget.first, onChanged: (_) => setState(() {}), onSubmitted: (_) => _create()),
          const SizedBox(height: 26),
          Center(child: PrimaryButton('Get started', big: true, onPressed: ready ? _create : null)),
          const SizedBox(height: 30),
          Padding(
            padding: const EdgeInsets.only(left: 4, bottom: 10),
            child: Text('Suggestions', style: TextStyle(fontSize: 15, color: p.muted)),
          ),
          for (final (i, s) in suggestions.indexed)
            Arrive(
              delay: Duration(milliseconds: 60 * i),
              child: Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: Material(
                  color: p.surface,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(20),
                    side: BorderSide(color: p.border),
                  ),
                  child: InkWell(
                    borderRadius: BorderRadius.circular(20),
                    onTap: () => setState(() {
                      _name.text = s.name;
                      _color = s.color;
                      _shape = s.shape;
                      _role = (s.title, s.description);
                      _bump++;
                    }),
                    child: Padding(
                      padding: const EdgeInsets.all(18),
                      child: Row(
                        children: [
                          BotFace(shape: s.shape, color: s.color, size: 52),
                          const SizedBox(width: 16),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(s.name, style: const TextStyle(fontSize: 20)),
                                const SizedBox(height: 3),
                                Text(s.blurb, style: TextStyle(fontSize: 15, height: 1.4, color: p.muted)),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
