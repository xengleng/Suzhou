// Suzhou for phones: your team of always-on Bots.
//
// Run with `--dart-define=DEMO=true` to start signed in with a sample team.

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';

import 'screens/chat.dart';
import 'screens/home.dart';
import 'screens/onboarding.dart';
import 'state.dart';
import 'theme.dart';
import 'widgets/avatar.dart';

const _demo = bool.fromEnvironment('DEMO');

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // `?demo` in the address also works for the web build.
  final demo = _demo || Uri.base.queryParameters.containsKey('demo');
  final state = AppState(demo: demo);
  // `?semantics` builds the accessibility tree up front (used by the browser tour).
  if (Uri.base.queryParameters.containsKey('semantics')) SemanticsBinding.instance.ensureSemantics();
  await state.load();
  runApp(SuzhouApp(state: state));
}

final navigatorKey = GlobalKey<NavigatorState>();

class SuzhouApp extends StatelessWidget {
  const SuzhouApp({super.key, required this.state});
  final AppState state;

  @override
  Widget build(BuildContext context) {
    return AppScope(
      state: state,
      child: ListenableBuilder(
        listenable: state,
        builder: (context, _) => MaterialApp(
          title: 'Suzhou',
          debugShowCheckedModeBanner: false,
          navigatorKey: navigatorKey,
          themeMode: state.themeMode,
          theme: buildTheme(Brightness.light),
          darkTheme: buildTheme(Brightness.dark),
          themeAnimationDuration: const Duration(milliseconds: 350),
          builder: (context, child) => AnnotatedRegion<SystemUiOverlayStyle>(
            value: Theme.of(context).brightness == Brightness.dark ? SystemUiOverlayStyle.light : SystemUiOverlayStyle.dark,
            child: Stack(children: [child!, const ToastLayer()]),
          ),
          home: const Root(),
        ),
      ),
    );
  }
}

/// Onboarding until signed in, then the Bot list.
class Root extends StatelessWidget {
  const Root({super.key});

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final signedIn = state.stage == Stage.main;
    return AnimatedSwitcher(
      duration: const Duration(milliseconds: 500),
      switchInCurve: Curves.easeOutQuint,
      transitionBuilder: (child, a) => FadeTransition(
        opacity: a,
        child: SlideTransition(
          position: Tween(begin: const Offset(0, 0.04), end: Offset.zero).animate(a),
          child: child,
        ),
      ),
      child: signedIn ? const HomeScreen(key: ValueKey('home')) : const OnboardingScreen(key: ValueKey('onboarding')),
    );
  }
}

/// Notification cards that drop in from the top when a Bot you aren't
/// looking at finishes, asks something, or needs approval.
class ToastLayer extends StatelessWidget {
  const ToastLayer({super.key});

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    return ValueListenableBuilder<List<Toast>>(
      valueListenable: state.toasts,
      builder: (context, toasts, _) => Positioned(
        top: MediaQuery.paddingOf(context).top + 8,
        left: 12,
        right: 12,
        child: Column(
          children: [
            for (final t in toasts)
              Dismissible(
                key: ValueKey(t.id),
                direction: DismissDirection.up,
                onDismissed: (_) => state.dismissToast(t.id),
                child: TweenAnimationBuilder<double>(
                  tween: Tween(begin: 0, end: 1),
                  duration: const Duration(milliseconds: 450),
                  curve: Curves.easeOutBack,
                  builder: (context, v, child) => Transform.translate(
                    offset: Offset(0, -60 * (1 - v)),
                    child: Opacity(opacity: v.clamp(0, 1), child: child),
                  ),
                  child: GestureDetector(
                    onTap: () {
                      state.dismissToast(t.id);
                      final bot = state.bot(t.bot);
                      if (bot != null) {
                        navigatorKey.currentState?.popUntil((r) => r.isFirst);
                        navigatorKey.currentState?.push(ChatScreen.route(bot));
                      }
                    },
                    child: Container(
                      margin: const EdgeInsets.only(bottom: 8),
                      padding: const EdgeInsets.all(14),
                      decoration: BoxDecoration(
                        color: p.surface,
                        borderRadius: BorderRadius.circular(18),
                        border: Border.all(color: p.border),
                        boxShadow: const [BoxShadow(color: Color(0x29000000), blurRadius: 30, offset: Offset(0, 10))],
                      ),
                      child: Row(
                        children: [
                          if (state.bot(t.bot) case final b?) BotFace(shape: b.shape, color: b.color, size: 34),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  t.title,
                                  style: TextStyle(
                                    fontSize: 15,
                                    fontWeight: FontWeight.w600,
                                    color: p.text,
                                    decoration: TextDecoration.none,
                                    fontFamily: kFont,
                                  ),
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  t.body,
                                  maxLines: 2,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(
                                    fontSize: 14,
                                    height: 1.35,
                                    color: p.muted,
                                    decoration: TextDecoration.none,
                                    fontFamily: kFont,
                                    fontWeight: FontWeight.w400,
                                  ),
                                ),
                              ],
                            ),
                          ),
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
  }
}
