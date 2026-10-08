// Colours and type, matched to the desktop app (src/theme.rs).

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

const kFont = 'Inter';
const kMono = 'DejaVuSansMono';

/// The ten Bot colours, in picker order.
const botColors = <Color>[
  Color(0xFF8D5B37),
  Color(0xFFE8423F),
  Color(0xFFEF7430),
  Color(0xFFF2A23A),
  Color(0xFF4CC16E),
  Color(0xFF4DB6A6),
  Color(0xFF3B82F6),
  Color(0xFF8B5CF6),
  Color(0xFFEC4899),
  Color(0xFF6F6F6F),
];

Color botColor(int i) => botColors[i % botColors.length];

/// App-specific colours that Material's ColorScheme doesn't name.
@immutable
class Palette extends ThemeExtension<Palette> {
  const Palette({
    required this.bg,
    required this.sidebar,
    required this.selected,
    required this.border,
    required this.borderStrong,
    required this.text,
    required this.muted,
    required this.faint,
    required this.userBubble,
    required this.userText,
    required this.botBubble,
    required this.field,
    required this.surface,
    required this.primary,
    required this.onPrimary,
    required this.accent,
    required this.link,
    required this.code,
    required this.codeBg,
    required this.online,
    required this.warning,
    required this.danger,
  });

  final Color bg, sidebar, selected, border, borderStrong, text, muted, faint;
  final Color userBubble, userText, botBubble, field, surface, primary, onPrimary;
  final Color accent, link, code, codeBg, online, warning, danger;

  static const light = Palette(
    bg: Color(0xFFFBFBFB),
    sidebar: Color(0xFFF6F6F6),
    selected: Color(0xFFE4E4E4),
    border: Color(0xFFEAEAEA),
    borderStrong: Color(0xFFDADADA),
    text: Color(0xFF161616),
    muted: Color(0xFF6B6B6B),
    faint: Color(0xFF9C9C9C),
    userBubble: Color(0xFF0B0B0B),
    userText: Color(0xFFFAFAFA),
    botBubble: Color(0xFFEEEEEE),
    field: Color(0xFFECECEC),
    surface: Color(0xFFFFFFFF),
    primary: Color(0xFF0B0B0B),
    onPrimary: Color(0xFFFFFFFF),
    accent: Color(0xFF3B82F6),
    link: Color(0xFF2D6BD6),
    code: Color(0xFFC0283E),
    codeBg: Color(0xFFF4F4F4),
    online: Color(0xFF34C759),
    warning: Color(0xFFF5A33C),
    danger: Color(0xFFE5484D),
  );

  static const dark = Palette(
    bg: Color(0xFF0E0E0E),
    sidebar: Color(0xFF151515),
    selected: Color(0xFF2A2A2A),
    border: Color(0xFF232323),
    borderStrong: Color(0xFF333333),
    text: Color(0xFFEDEDED),
    muted: Color(0xFF9A9A9A),
    faint: Color(0xFF6A6A6A),
    userBubble: Color(0xFF4A4A4A),
    userText: Color(0xFFF5F5F5),
    botBubble: Color(0xFF242424),
    field: Color(0xFF1F1F1F),
    surface: Color(0xFF1A1A1A),
    primary: Color(0xFFF2F2F2),
    onPrimary: Color(0xFF0B0B0B),
    accent: Color(0xFF4C8DF6),
    link: Color(0xFF5B9BFF),
    code: Color(0xFFF2627A),
    codeBg: Color(0xFF1C1C1C),
    online: Color(0xFF30D158),
    warning: Color(0xFFF5A33C),
    danger: Color(0xFFF0585D),
  );

  @override
  Palette copyWith() => this;

  @override
  Palette lerp(ThemeExtension<Palette>? other, double t) {
    if (other is! Palette) return this;
    Color l(Color a, Color b) => Color.lerp(a, b, t)!;
    return Palette(
      bg: l(bg, other.bg),
      sidebar: l(sidebar, other.sidebar),
      selected: l(selected, other.selected),
      border: l(border, other.border),
      borderStrong: l(borderStrong, other.borderStrong),
      text: l(text, other.text),
      muted: l(muted, other.muted),
      faint: l(faint, other.faint),
      userBubble: l(userBubble, other.userBubble),
      userText: l(userText, other.userText),
      botBubble: l(botBubble, other.botBubble),
      field: l(field, other.field),
      surface: l(surface, other.surface),
      primary: l(primary, other.primary),
      onPrimary: l(onPrimary, other.onPrimary),
      accent: l(accent, other.accent),
      link: l(link, other.link),
      code: l(code, other.code),
      codeBg: l(codeBg, other.codeBg),
      online: l(online, other.online),
      warning: l(warning, other.warning),
      danger: l(danger, other.danger),
    );
  }
}

extension PaletteOf on BuildContext {
  Palette get p => Theme.of(this).extension<Palette>()!;
}

ThemeData buildTheme(Brightness brightness) {
  final p = brightness == Brightness.dark ? Palette.dark : Palette.light;
  final base = ThemeData(
    brightness: brightness,
    useMaterial3: true,
    fontFamily: kFont,
    scaffoldBackgroundColor: p.bg,
    colorScheme: ColorScheme.fromSeed(seedColor: p.accent, brightness: brightness, surface: p.bg, primary: p.primary, onPrimary: p.onPrimary),
    splashFactory: InkSparkle.splashFactory,
    extensions: [p],
  );
  return base.copyWith(
    textTheme: base.textTheme.apply(bodyColor: p.text, displayColor: p.text),
    dividerColor: p.border,
    appBarTheme: AppBarTheme(
      backgroundColor: p.bg,
      foregroundColor: p.text,
      surfaceTintColor: Colors.transparent,
      elevation: 0,
      scrolledUnderElevation: 0,
      titleTextStyle: TextStyle(fontFamily: kFont, fontSize: 17, fontWeight: FontWeight.w500, color: p.text),
    ),
    bottomSheetTheme: BottomSheetThemeData(
      backgroundColor: p.surface,
      surfaceTintColor: Colors.transparent,
      showDragHandle: true,
      dragHandleColor: p.borderStrong,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.vertical(top: Radius.circular(22))),
    ),
    switchTheme: SwitchThemeData(
      trackColor: WidgetStateProperty.resolveWith((s) => s.contains(WidgetState.selected) ? p.accent : p.borderStrong),
      thumbColor: const WidgetStatePropertyAll(Colors.white),
      trackOutlineColor: const WidgetStatePropertyAll(Colors.transparent),
    ),
    pageTransitionsTheme: const PageTransitionsTheme(
      builders: {
        TargetPlatform.android: CupertinoPageTransitionsBuilder(),
        TargetPlatform.iOS: CupertinoPageTransitionsBuilder(),
        TargetPlatform.linux: CupertinoPageTransitionsBuilder(),
        TargetPlatform.macOS: CupertinoPageTransitionsBuilder(),
        TargetPlatform.windows: CupertinoPageTransitionsBuilder(),
      },
    ),
  );
}
