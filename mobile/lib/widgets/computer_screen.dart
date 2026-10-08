// A picture of a Bot's screen on the Agent Computer: a browser window on a
// desktop with a dock, and the pointer gliding to whatever it is doing.

import 'package:flutter/material.dart';

import '../models.dart';

class ComputerScreen extends StatelessWidget {
  const ComputerScreen({super.key, required this.state, this.live = true, this.onTapAt});

  final ComputerState state;
  final bool live;

  /// Called with the tap position as fractions of the screen (takeover).
  final void Function(double x, double y)? onTapAt;

  @override
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: 16 / 10,
      child: LayoutBuilder(
        builder: (context, box) {
          final w = box.maxWidth, h = box.maxHeight;
          final s = w / 900;
          final blank = state.url == 'about:blank';
          final dark = state.url.contains('datacamp');
          final pageBg = dark ? const Color(0xFF05192D) : Colors.white;
          final fg = dark ? Colors.white : const Color(0xFF1B1B1B);
          final muted = dark ? const Color(0xFF9AA7B4) : const Color(0xFF8A8A8A);
          final host = state.url.replaceFirst(RegExp(r'^https?://'), '');
          Widget bar(double bw) => Container(
            width: bw * s,
            height: 10 * s,
            margin: EdgeInsets.only(bottom: 12 * s),
            decoration: BoxDecoration(color: muted.withValues(alpha: 0.45), borderRadius: BorderRadius.circular(3 * s)),
          );

          final page = blank
              ? Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.public, size: 40 * s, color: muted),
                      SizedBox(height: 8 * s),
                      Text(
                        'Nothing open yet',
                        style: TextStyle(fontSize: 15 * s, color: muted),
                      ),
                    ],
                  ),
                )
              : Padding(
                  padding: EdgeInsets.all(28 * s),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Text(
                            host.split('/').first,
                            style: TextStyle(fontSize: 15 * s, fontWeight: FontWeight.w600, color: fg),
                          ),
                          const Spacer(),
                          Container(
                            padding: EdgeInsets.symmetric(horizontal: 10 * s, vertical: 5 * s),
                            decoration: BoxDecoration(
                              border: Border.all(color: muted),
                              borderRadius: BorderRadius.circular(5 * s),
                            ),
                            child: Text(
                              'Sign in',
                              style: TextStyle(fontSize: 11 * s, color: fg),
                            ),
                          ),
                        ],
                      ),
                      SizedBox(height: 30 * s),
                      Text(
                        state.title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(fontSize: 30 * s, fontWeight: FontWeight.w600, color: fg),
                      ),
                      SizedBox(height: 16 * s),
                      bar(520),
                      bar(460),
                      bar(380),
                    ],
                  ),
                );

          final browser = Positioned(
            left: 60 * s,
            top: 28 * s,
            width: 780 * s,
            height: h - 90 * s,
            child: Container(
              clipBehavior: Clip.antiAlias,
              decoration: BoxDecoration(
                color: const Color(0xFFDEE1E6),
                borderRadius: BorderRadius.circular(8 * s),
                boxShadow: [BoxShadow(color: Colors.black26, blurRadius: 30 * s, offset: Offset(0, 10 * s))],
              ),
              child: Column(
                children: [
                  SizedBox(
                    height: 30 * s,
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        SizedBox(width: 10 * s),
                        Container(
                          height: 24 * s,
                          width: 200 * s,
                          padding: EdgeInsets.symmetric(horizontal: 10 * s),
                          decoration: BoxDecoration(
                            color: Colors.white,
                            borderRadius: BorderRadius.vertical(top: Radius.circular(7 * s)),
                          ),
                          child: Row(
                            children: [
                              Container(width: 10 * s, height: 10 * s, color: const Color(0xFF10B981)),
                              SizedBox(width: 6 * s),
                              Expanded(
                                child: Text(
                                  state.title,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(fontSize: 10 * s, color: const Color(0xFF333333)),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                  Container(
                    height: 30 * s,
                    color: Colors.white,
                    padding: EdgeInsets.symmetric(horizontal: 10 * s),
                    child: Row(
                      children: [
                        Icon(Icons.chevron_left, size: 14 * s, color: Colors.grey),
                        Icon(Icons.chevron_right, size: 14 * s, color: Colors.grey.shade400),
                        Icon(Icons.refresh, size: 12 * s, color: Colors.grey),
                        SizedBox(width: 8 * s),
                        Expanded(
                          child: Container(
                            height: 20 * s,
                            padding: EdgeInsets.symmetric(horizontal: 10 * s),
                            alignment: Alignment.centerLeft,
                            decoration: BoxDecoration(color: const Color(0xFFF1F3F4), borderRadius: BorderRadius.circular(10 * s)),
                            child: Text(
                              blank ? 'about:blank' : host,
                              maxLines: 1,
                              style: TextStyle(fontSize: 10.5 * s, color: const Color(0xFF333333)),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: ColoredBox(
                      color: pageBg,
                      child: SizedBox.expand(child: page),
                    ),
                  ),
                ],
              ),
            ),
          );

          final dock = Positioned(
            left: 0,
            right: 0,
            bottom: 10 * s,
            child: Center(
              child: Container(
                padding: EdgeInsets.symmetric(horizontal: 12 * s, vertical: 7 * s),
                decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.55), borderRadius: BorderRadius.circular(12 * s)),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    for (final (icon, c) in [(Icons.public, 0xFF1A73E8), (Icons.folder_rounded, 0xFFF59E0B), (Icons.terminal_rounded, 0xFF111111)])
                      Container(
                        margin: EdgeInsets.symmetric(horizontal: 5 * s),
                        width: 30 * s,
                        height: 30 * s,
                        decoration: BoxDecoration(color: Color(c), borderRadius: BorderRadius.circular(8 * s)),
                        child: Icon(icon, size: 16 * s, color: Colors.white),
                      ),
                  ],
                ),
              ),
            ),
          );

          final pointer = AnimatedPositioned(
            duration: const Duration(milliseconds: 750),
            curve: Curves.easeInOutCubic,
            left: state.x * w,
            top: state.y * h,
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                Icon(Icons.navigation_rounded, size: (22 * s).clamp(14, 26), color: const Color(0xFF111111)),
                if (live && state.typing != null)
                  Positioned(
                    left: 18,
                    top: 18,
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                      decoration: BoxDecoration(color: const Color(0xFF111111), borderRadius: BorderRadius.circular(6)),
                      child: Text('⌨ ${state.typing}', style: const TextStyle(color: Colors.white, fontSize: 11)),
                    ),
                  ),
              ],
            ),
          );

          return GestureDetector(
            onTapDown: onTapAt == null ? null : (d) => onTapAt!((d.localPosition.dx / w).clamp(0, 1), (d.localPosition.dy / h).clamp(0, 1)),
            child: Container(
              decoration: const BoxDecoration(
                gradient: LinearGradient(begin: Alignment.topLeft, end: Alignment.bottomRight, colors: [Color(0xFFC9CED6), Color(0xFF8E97A5)]),
              ),
              child: Stack(
                clipBehavior: Clip.hardEdge,
                children: [
                  browser,
                  dock,
                  pointer,
                  if (live && state.status.isNotEmpty && state.status != 'Idle')
                    Positioned(
                      top: 10,
                      left: 0,
                      right: 0,
                      child: Center(
                        child: AnimatedSwitcher(
                          duration: const Duration(milliseconds: 250),
                          child: Container(
                            key: ValueKey(state.status),
                            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                            decoration: BoxDecoration(color: const Color(0xC7111111), borderRadius: BorderRadius.circular(20)),
                            child: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                const _LiveDot(),
                                const SizedBox(width: 6),
                                Text(state.status, style: const TextStyle(color: Colors.white, fontSize: 12)),
                              ],
                            ),
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

class _LiveDot extends StatefulWidget {
  const _LiveDot();
  @override
  State<_LiveDot> createState() => _LiveDotState();
}

class _LiveDotState extends State<_LiveDot> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1000))..repeat(reverse: true);

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => FadeTransition(
    opacity: Tween(begin: 0.3, end: 1.0).animate(_c),
    child: const CircleAvatar(radius: 3, backgroundColor: Color(0xFF34C759)),
  );
}
