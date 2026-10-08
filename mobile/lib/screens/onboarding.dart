// First run: sign in, pick your apps, wait for the computer, meet a Bot.

import 'package:flutter/material.dart';

import '../data.dart';
import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/avatar.dart';
import '../widgets/kit.dart';

class OnboardingScreen extends StatelessWidget {
  const OnboardingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final body = switch (state.stage) {
      Stage.signIn => const _SignIn(),
      Stage.authorizing => const _Authorizing(),
      Stage.tools => const _Tools(),
      Stage.settingUp => const _SettingUp(),
      _ => const _Meet(),
    };
    return Scaffold(
      body: SafeArea(
        child: AnimatedSwitcher(
          duration: const Duration(milliseconds: 520),
          switchInCurve: Curves.easeOutQuint,
          switchOutCurve: Curves.easeIn,
          transitionBuilder: (child, a) => FadeTransition(
            opacity: a,
            child: SlideTransition(
              position: Tween(begin: const Offset(0, 0.05), end: Offset.zero).animate(a),
              child: child,
            ),
          ),
          child: KeyedSubtree(key: ValueKey(state.stage), child: body),
        ),
      ),
    );
  }
}

class _Page extends StatelessWidget {
  const _Page({required this.children});
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Center(
    child: SingleChildScrollView(
      padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 24),
      child: Column(mainAxisSize: MainAxisSize.min, children: children),
    ),
  );
}

Widget _title(BuildContext context, String text, {double size = 28}) => Text(
  text,
  textAlign: TextAlign.center,
  style: TextStyle(fontSize: size, fontWeight: FontWeight.w600, height: 1.2, color: context.p.text),
);

Widget _subtitle(BuildContext context, String text) => Text(
  text,
  textAlign: TextAlign.center,
  style: TextStyle(fontSize: 16, height: 1.5, color: context.p.muted),
);

class _SignIn extends StatelessWidget {
  const _SignIn();

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    return _Page(
      children: [
        CustomPaint(
          size: const Size.square(84),
          painter: FacePainter(Shape.circle, p.text, eyes: p.bg),
        ),
        const SizedBox(height: 18),
        Text(
          'Suzhou',
          style: TextStyle(fontSize: 56, fontWeight: FontWeight.w500, color: p.text, height: 1.1),
        ),
        const SizedBox(height: 18),
        Text(
          'Your team of always-on Bots that you can give real work to.',
          textAlign: TextAlign.center,
          style: TextStyle(fontSize: 20, height: 1.45, color: p.text),
        ),
        const SizedBox(height: 36),
        Container(
          padding: const EdgeInsets.all(2),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(40),
            border: Border.all(color: p.accent, width: 2),
          ),
          child: Material(
            color: p.primary,
            borderRadius: BorderRadius.circular(40),
            child: InkWell(
              borderRadius: BorderRadius.circular(40),
              onTap: () => AppScope.read(context).go(Stage.authorizing),
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 30, vertical: 13),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      'Sign in',
                      style: TextStyle(fontSize: 18, fontWeight: FontWeight.w500, color: p.onPrimary),
                    ),
                    const SizedBox(width: 8),
                    Icon(Icons.arrow_forward_rounded, size: 18, color: p.onPrimary),
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _Authorizing extends StatelessWidget {
  const _Authorizing();

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    final state = AppScope.read(context);
    return _Page(
      children: [
        CustomPaint(
          size: const Size.square(56),
          painter: FacePainter(Shape.circle, p.text, eyes: p.bg),
        ),
        const SizedBox(height: 20),
        _title(context, 'Finish signing in in your browser', size: 26),
        const SizedBox(height: 12),
        _subtitle(context, 'We opened a page in your browser. Approve the sign-in there and you will come straight back here.'),
        const SizedBox(height: 22),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const SmallSpinner(size: 16),
            const SizedBox(width: 10),
            Flexible(
              child: Text('Waiting for your browser…', style: TextStyle(color: p.muted, fontSize: 15)),
            ),
          ],
        ),
        const SizedBox(height: 26),
        Wrap(
          alignment: WrapAlignment.center,
          spacing: 10,
          runSpacing: 10,
          children: [
            SecondaryButton('Cancel', onPressed: () => state.go(Stage.signIn)),
            PrimaryButton("I've signed in", onPressed: () => state.go(Stage.tools)),
          ],
        ),
      ],
    );
  }
}

class _Tools extends StatelessWidget {
  const _Tools();

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    return _Page(
      children: [
        _title(context, 'What do you use for work?'),
        const SizedBox(height: 12),
        _subtitle(context, 'Your Bots can use these apps on your behalf. You can change this any time in Plugins.'),
        const SizedBox(height: 22),
        for (final (id, name) in workTools)
          Builder(
            builder: (context) {
              final picked = state.toolsPicked.contains(id);
              final plugin = state.pluginList.firstWhere((x) => x.id == id);
              return Padding(
                padding: const EdgeInsets.only(bottom: 10),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 220),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(14),
                    border: Border.all(color: picked ? p.accent : p.border),
                  ),
                  child: ListTile(
                    tileColor: picked ? p.accent.withValues(alpha: 0.08) : p.surface,
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
                    leading: GlyphTile(plugin.glyph, plugin.color, size: 30),
                    title: Text(name, style: const TextStyle(fontSize: 16)),
                    trailing: AnimatedSwitcher(
                      duration: const Duration(milliseconds: 220),
                      transitionBuilder: (c, a) => ScaleTransition(scale: a, child: c),
                      child: picked
                          ? CircleAvatar(
                              key: const ValueKey(1),
                              radius: 11,
                              backgroundColor: p.accent,
                              child: const Icon(Icons.check, size: 14, color: Colors.white),
                            )
                          : Container(
                              key: const ValueKey(0),
                              width: 22,
                              height: 22,
                              decoration: BoxDecoration(
                                shape: BoxShape.circle,
                                border: Border.all(color: p.borderStrong),
                              ),
                            ),
                    ),
                    onTap: () => state.toggleTool(id),
                  ),
                ),
              );
            },
          ),
        const SizedBox(height: 14),
        Wrap(
          alignment: WrapAlignment.center,
          spacing: 10,
          runSpacing: 10,
          children: [
            SecondaryButton(
              'Skip',
              onPressed: () {
                state.toolsPicked.clear();
                state.go(Stage.settingUp);
              },
            ),
            PrimaryButton('Continue', onPressed: () => state.go(Stage.settingUp)),
          ],
        ),
      ],
    );
  }
}

class _SettingUp extends StatelessWidget {
  const _SettingUp();

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    return _Page(
      children: [
        Container(
          width: 72,
          height: 72,
          decoration: BoxDecoration(color: p.field, borderRadius: BorderRadius.circular(18)),
          child: Icon(Icons.desktop_windows_outlined, size: 34, color: p.text),
        ),
        const SizedBox(height: 20),
        _title(context, 'Setting up your computer', size: 26),
        const SizedBox(height: 12),
        _subtitle(context, 'Every Bot gets its own screen on one cloud computer. Files, sign-ins and apps are shared between them.'),
        const SizedBox(height: 26),
        TweenAnimationBuilder<double>(
          tween: Tween(begin: 0, end: 1),
          duration: const Duration(milliseconds: 3400),
          curve: Curves.easeInOutCubic,
          builder: (context, v, _) => Column(
            children: [
              ClipRRect(
                borderRadius: BorderRadius.circular(4),
                child: LinearProgressIndicator(value: v, minHeight: 6, backgroundColor: p.field, color: p.accent),
              ),
              const SizedBox(height: 14),
              Text(
                v < 0.34 ? 'Starting your cloud computer…' : (v < 0.68 ? 'Installing a browser and terminal…' : 'Syncing your connected apps…'),
                style: TextStyle(fontSize: 14, color: p.muted),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _Meet extends StatelessWidget {
  const _Meet();

  @override
  Widget build(BuildContext context) {
    return _Page(
      children: [
        const SizedBox(
          height: 130,
          child: Align(
            alignment: Alignment.bottomCenter,
            child: BotFace(shape: Shape.circle, color: 6, size: 108, working: true, hop: 16),
          ),
        ),
        const SizedBox(height: 20),
        _title(context, 'Meet a future teammate', size: 30),
        const SizedBox(height: 12),
        _subtitle(context, 'Bots work on their own computer, remember what you teach them, and ask before anything risky. Give one a name and a job.'),
        const SizedBox(height: 28),
        PrimaryButton('Create your first Bot', big: true, onPressed: () => AppScope.read(context).go(Stage.main)),
      ],
    );
  }
}
