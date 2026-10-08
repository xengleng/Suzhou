// Small building blocks shared by every screen.

import 'package:flutter/material.dart';

import '../theme.dart';

/// Icon names used in messages (kept as strings so they can be saved).
IconData iconFor(String? name) => switch (name) {
  'clock' => Icons.schedule_rounded,
  'copy' => Icons.copy_rounded,
  'book' => Icons.menu_book_rounded,
  'hand' => Icons.back_hand_outlined,
  'play' => Icons.play_arrow_rounded,
  'stop' => Icons.stop_rounded,
  'download' => Icons.download_rounded,
  _ => Icons.info_outline_rounded,
};

class PrimaryButton extends StatelessWidget {
  const PrimaryButton(this.label, {super.key, this.onPressed, this.big = false, this.danger = false, this.icon});
  final String label;
  final VoidCallback? onPressed;
  final bool big, danger;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    final enabled = onPressed != null;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 220),
      curve: Curves.easeOut,
      decoration: BoxDecoration(color: !enabled ? p.selected : (danger ? p.danger : p.primary), borderRadius: BorderRadius.circular(big ? 14 : 11)),
      child: Material(
        type: MaterialType.transparency,
        child: InkWell(
          borderRadius: BorderRadius.circular(big ? 14 : 11),
          onTap: onPressed,
          child: Padding(
            padding: EdgeInsets.symmetric(horizontal: big ? 22 : 16, vertical: big ? 14 : 10),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Flexible(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: big ? 17 : 15, fontWeight: FontWeight.w500, color: !enabled ? p.faint : (danger ? Colors.white : p.onPrimary)),
                  ),
                ),
                if (icon != null) ...[const SizedBox(width: 8), Icon(icon, size: 18, color: p.onPrimary)],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class SecondaryButton extends StatelessWidget {
  const SecondaryButton(this.label, {super.key, this.onPressed, this.danger = false});
  final String label;
  final VoidCallback? onPressed;
  final bool danger;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    return Material(
      color: p.field,
      borderRadius: BorderRadius.circular(11),
      child: InkWell(
        borderRadius: BorderRadius.circular(11),
        onTap: onPressed,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
          child: Text(
            label,
            style: TextStyle(fontSize: 15, fontWeight: FontWeight.w500, color: danger ? p.danger : p.text),
          ),
        ),
      ),
    );
  }
}

/// A coloured square with a short label, standing in for an app's logo.
class GlyphTile extends StatelessWidget {
  const GlyphTile(this.glyph, this.color, {super.key, this.size = 40});
  final String glyph;
  final int color;
  final double size;

  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    alignment: Alignment.center,
    decoration: BoxDecoration(color: Color(color), borderRadius: BorderRadius.circular(size * 0.24)),
    child: Text(
      glyph,
      style: TextStyle(color: Colors.white, fontWeight: FontWeight.w600, fontSize: glyph.length > 1 ? size * 0.36 : size * 0.5),
    ),
  );
}

class SmallSpinner extends StatelessWidget {
  const SmallSpinner({super.key, this.size = 14, this.color});
  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: size,
    height: size,
    child: CircularProgressIndicator(strokeWidth: 2, color: color ?? context.p.muted),
  );
}

/// Three dots pulsing in turn.
class TypingDots extends StatefulWidget {
  const TypingDots({super.key});
  @override
  State<TypingDots> createState() => _TypingDotsState();
}

class _TypingDotsState extends State<TypingDots> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1000))..repeat();

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: _c,
    builder: (context, _) => Row(
      mainAxisSize: MainAxisSize.min,
      children: List.generate(3, (i) {
        final phase = (_c.value - i * 0.18) % 1.0;
        final lift = (phase * 2 * 3.14159).clamp(0, 3.14159);
        final o = 0.35 + 0.65 * (lift == 0 ? 0 : (1 - (lift - 1.5708).abs() / 1.5708));
        return Padding(
          padding: const EdgeInsets.symmetric(horizontal: 2),
          child: Opacity(
            opacity: o.clamp(0.35, 1.0),
            child: CircleAvatar(radius: 3.5, backgroundColor: context.p.muted),
          ),
        );
      }),
    ),
  );
}

/// Fades and lifts its child in once, when first built.
class Arrive extends StatelessWidget {
  const Arrive({super.key, required this.child, this.enabled = true, this.fromRight = false, this.delay = Duration.zero});
  final Widget child;
  final bool enabled, fromRight;
  final Duration delay;

  @override
  Widget build(BuildContext context) {
    if (!enabled) return child;
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: 1),
      duration: const Duration(milliseconds: 420) + delay,
      curve: Interval(delay.inMilliseconds / (420 + delay.inMilliseconds), 1, curve: Curves.easeOutQuint),
      child: child,
      builder: (context, t, child) => Opacity(
        opacity: t,
        child: Transform.translate(offset: Offset(fromRight ? 12 * (1 - t) : 0, 12 * (1 - t)), child: child),
      ),
    );
  }
}

/// A section heading in lists and sheets.
class SectionLabel extends StatelessWidget {
  const SectionLabel(this.text, {super.key, this.trailing});
  final String text;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(4, 22, 4, 8),
    child: Row(
      children: [
        Expanded(
          child: Text(
            text,
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w500, color: context.p.muted),
          ),
        ),
        ?trailing,
      ],
    ),
  );
}

/// A rounded text field in the app's style.
class Field extends StatelessWidget {
  const Field({
    super.key,
    required this.controller,
    this.hint,
    this.maxLines = 1,
    this.onChanged,
    this.onSubmitted,
    this.autofocus = false,
    this.focusNode,
    this.big = false,
  });
  final TextEditingController controller;
  final String? hint;
  final int maxLines;
  final ValueChanged<String>? onChanged, onSubmitted;
  final bool autofocus, big;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    OutlineInputBorder border(Color c) => OutlineInputBorder(
      borderRadius: BorderRadius.circular(12),
      borderSide: BorderSide(color: c),
    );
    return TextField(
      controller: controller,
      focusNode: focusNode,
      autofocus: autofocus,
      maxLines: maxLines,
      minLines: 1,
      onChanged: onChanged,
      onSubmitted: onSubmitted,
      style: TextStyle(fontSize: big ? 17 : 15, color: p.text),
      decoration: InputDecoration(
        hintText: hint,
        hintStyle: TextStyle(color: p.faint),
        filled: true,
        fillColor: p.surface,
        isDense: true,
        contentPadding: EdgeInsets.symmetric(horizontal: 14, vertical: big ? 15 : 12),
        enabledBorder: border(p.borderStrong),
        focusedBorder: border(p.accent),
      ),
    );
  }
}

/// A list row with an icon, used in sheets and menus.
class SheetRow extends StatelessWidget {
  const SheetRow(this.icon, this.label, {super.key, this.onTap, this.danger = false, this.trailing});
  final IconData icon;
  final String label;
  final VoidCallback? onTap;
  final bool danger;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    return ListTile(
      leading: Icon(icon, color: danger ? p.danger : p.muted, size: 22),
      title: Text(label, style: TextStyle(fontSize: 16, color: danger ? p.danger : p.text)),
      trailing: trailing,
      onTap: onTap,
      visualDensity: VisualDensity.compact,
    );
  }
}
