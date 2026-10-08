// The Agent Computer: watch the Bot work, take over, hand back.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/computer_screen.dart';
import '../widgets/kit.dart';
import 'chat.dart';

class AgentComputerPage extends StatelessWidget {
  const AgentComputerPage({super.key, required this.bot});
  final Bot bot;

  static Route<void> route(Bot bot) => MaterialPageRoute(builder: (_) => AgentComputerPage(bot: bot));

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final working = bot.status == BotStatus.working;
    return Scaffold(
      appBar: AppBar(title: const Text('Agent Computer')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Hero(
            tag: 'screen-${bot.id}',
            child: Container(
              clipBehavior: Clip.antiAlias,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: p.border),
              ),
              child: ComputerScreen(state: bot.computer, live: working),
            ),
          ),
          const SizedBox(height: 14),
          Row(
            children: [
              working ? const SmallSpinner() : CircleAvatar(radius: 4, backgroundColor: p.faint),
              const SizedBox(width: 8),
              Expanded(
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 250),
                  child: Text(
                    working ? bot.computer.status : 'Idle',
                    key: ValueKey(bot.computer.status + working.toString()),
                    style: TextStyle(color: p.muted),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: PrimaryButton(
                  'Take over',
                  big: true,
                  onPressed: () {
                    HapticFeedback.mediumImpact();
                    state.takeOver(bot);
                    Navigator.of(context).push(TakeoverPage.route(bot));
                  },
                ),
              ),
              const SizedBox(width: 10),
              SecondaryButton('Teach a task', onPressed: () => showTeachSheet(context, bot)),
            ],
          ),
          const SizedBox(height: 16),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(color: p.field, borderRadius: BorderRadius.circular(14)),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(Icons.lock_outline_rounded, size: 18, color: p.muted),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    'Take over for passwords, passkeys, 2FA, CAPTCHAs and payments, then hand back. Never paste secrets in chat.',
                    style: TextStyle(fontSize: 14, height: 1.45, color: p.muted),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),
          Text(
            'One cloud computer per account. Each Bot gets its own screen; cookies, files and command-line sign-ins are shared.',
            style: TextStyle(fontSize: 13, height: 1.45, color: p.faint),
          ),
        ],
      ),
    );
  }
}

/// Full-screen control of the Bot's screen. Turn the phone sideways for room.
class TakeoverPage extends StatelessWidget {
  const TakeoverPage({super.key, required this.bot});
  final Bot bot;

  static Route<void> route(Bot bot) => PageRouteBuilder(
    opaque: true,
    transitionDuration: const Duration(milliseconds: 380),
    reverseTransitionDuration: const Duration(milliseconds: 280),
    pageBuilder: (_, _, _) => TakeoverPage(bot: bot),
    transitionsBuilder: (_, a, _, child) => FadeTransition(
      opacity: a,
      child: ScaleTransition(
        scale: Tween(begin: 0.96, end: 1.0).animate(CurvedAnimation(parent: a, curve: Curves.easeOutQuint)),
        child: child,
      ),
    ),
  );

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    return Scaffold(
      backgroundColor: const Color(0xFF111111),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
              child: Row(
                children: [
                  BotFace(shape: bot.shape, color: bot.color, size: 26),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                      decoration: BoxDecoration(color: p.warning.withValues(alpha: 0.18), borderRadius: BorderRadius.circular(20)),
                      child: Row(
                        children: [
                          Icon(Icons.back_hand_outlined, size: 14, color: p.warning),
                          const SizedBox(width: 6),
                          Flexible(
                            child: Text(
                              "You're in control · ${bot.name} is paused",
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: const TextStyle(color: Colors.white, fontSize: 13),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(width: 10),
                  FilledButton(
                    style: FilledButton.styleFrom(backgroundColor: Colors.white, foregroundColor: Colors.black),
                    onPressed: () {
                      state.handBack(bot);
                      Navigator.pop(context);
                    },
                    child: const Text('Hand back'),
                  ),
                ],
              ),
            ),
            Expanded(
              child: Center(
                child: Hero(
                  tag: 'screen-${bot.id}',
                  child: InteractiveViewer(
                    maxScale: 4,
                    child: ComputerScreen(
                      state: bot.computer,
                      live: false,
                      onTapAt: (x, y) {
                        HapticFeedback.selectionClick();
                        state.moveComputerPointer(bot, x, y);
                      },
                    ),
                  ),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text('Tap to click. Pinch to zoom.', style: TextStyle(color: Colors.white.withValues(alpha: 0.5), fontSize: 13)),
            ),
          ],
        ),
      ),
    );
  }
}
