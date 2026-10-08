// Just enough Markdown for chat: paragraphs, lists, headings, `code`,
// **bold** and links. Same rules as src/ui/rich.rs.

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../theme.dart';

enum SpanKind { plain, code, bold, link }

/// Splits a line into styled pieces, dropping the Markdown markers.
List<(String, SpanKind)> parseInline(String line) {
  final out = <(String, SpanKind)>[];
  void add(String text, SpanKind kind) {
    if (text.isEmpty) return;
    if (out.isNotEmpty && out.last.$2 == kind) {
      out[out.length - 1] = (out.last.$1 + text, kind);
    } else {
      out.add((text, kind));
    }
  }

  var rest = line;
  final link = RegExp(r'https?://');
  while (rest.isNotEmpty) {
    final code = rest.indexOf('`');
    final bold = rest.indexOf('**');
    final url = rest.indexOf(link);
    final candidates = [code, bold, url].where((i) => i >= 0).toList()..sort();
    if (candidates.isEmpty) {
      add(rest, SpanKind.plain);
      break;
    }
    final at = candidates.first;
    add(rest.substring(0, at), SpanKind.plain);
    rest = rest.substring(at);
    if (at == code) {
      final end = rest.indexOf('`', 1);
      if (end > 0) {
        add(rest.substring(1, end), SpanKind.code);
        rest = rest.substring(end + 1);
      } else {
        add('`', SpanKind.plain);
        rest = rest.substring(1);
      }
    } else if (at == bold) {
      final end = rest.indexOf('**', 2);
      if (end > 0) {
        add(rest.substring(2, end), SpanKind.bold);
        rest = rest.substring(end + 2);
      } else {
        add('**', SpanKind.plain);
        rest = rest.substring(2);
      }
    } else {
      var end = rest.indexOf(RegExp(r'\s'));
      if (end < 0) end = rest.length;
      var u = rest.substring(0, end);
      while (u.isNotEmpty && '.,);:!?”'.contains(u[u.length - 1])) {
        u = u.substring(0, u.length - 1);
      }
      add(u, SpanKind.link);
      rest = rest.substring(u.length);
    }
  }
  return out;
}

TextSpan inlineSpan(BuildContext context, String line, TextStyle base) {
  final p = context.p;
  return TextSpan(
    style: base,
    children: [
      for (final (text, kind) in parseInline(line))
        switch (kind) {
          SpanKind.plain => TextSpan(text: text),
          SpanKind.bold => TextSpan(
            text: text,
            style: const TextStyle(fontWeight: FontWeight.w600),
          ),
          SpanKind.code => TextSpan(
            text: text,
            style: TextStyle(fontFamily: kMono, fontSize: (base.fontSize ?? 16) * 0.92, color: p.code, backgroundColor: p.codeBg),
          ),
          SpanKind.link => TextSpan(
            text: text,
            style: TextStyle(color: p.link),
            recognizer: TapGestureRecognizer()..onTap = () => launchUrl(Uri.parse(text), mode: LaunchMode.externalApplication),
          ),
        },
    ],
  );
}

class Markdown extends StatelessWidget {
  const Markdown(this.text, {super.key, this.size = 16, this.color});
  final String text;
  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final base = TextStyle(fontSize: size, height: 1.55, color: color ?? context.p.text, fontFamily: kFont);
    final children = <Widget>[];
    for (final raw in text.split('\n')) {
      final line = raw.trimRight();
      if (line.trim().isEmpty) {
        children.add(SizedBox(height: size * 0.6));
        continue;
      }
      if (line.startsWith('### ')) {
        children.add(_heading(context, line.substring(4), size + 2, base));
      } else if (line.startsWith('## ')) {
        children.add(Padding(padding: const EdgeInsets.only(top: 6), child: _heading(context, line.substring(3), size + 6, base)));
      } else if (line.startsWith('# ')) {
        children.add(_heading(context, line.substring(2), size + 12, base));
      } else if (line.startsWith('- ') || line.startsWith('* ')) {
        children.add(
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: EdgeInsets.only(top: size * 0.62, left: 4, right: 10),
                child: CircleAvatar(radius: 2.4, backgroundColor: base.color),
              ),
              Expanded(child: Text.rich(inlineSpan(context, line.substring(2), base))),
            ],
          ),
        );
      } else if (RegExp(r'^\d{1,3}\. ').hasMatch(line)) {
        final dot = line.indexOf('. ');
        children.add(
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(
                width: size * 1.3,
                child: Text('${line.substring(0, dot)}.', style: base),
              ),
              Expanded(child: Text.rich(inlineSpan(context, line.substring(dot + 2), base))),
            ],
          ),
        );
      } else {
        children.add(Text.rich(inlineSpan(context, line, base)));
      }
    }
    return Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: children);
  }

  Widget _heading(BuildContext context, String text, double size, TextStyle base) =>
      Text.rich(inlineSpan(context, text, base.copyWith(fontSize: size, height: 1.3, fontWeight: FontWeight.w600)));
}
