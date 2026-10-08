// A Bot's face: its shape in its colour with two little eyes. It blinks now
// and then, and hops while the Bot works. Same geometry as the desktop
// app's SVGs in assets/avatars, drawn on a 100x100 grid.

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../models.dart';
import '../theme.dart';

const _eyeTop = {
  Shape.circle: 40.0,
  Shape.blob: 42.0,
  Shape.square: 40.0,
  Shape.pill: 40.0,
  Shape.triangle: 56.0,
  Shape.hexagon: 42.0,
  Shape.cloud: 48.0,
  Shape.drop: 54.0,
};

class FacePainter extends CustomPainter {
  FacePainter(this.shape, this.color, {this.eyes = Colors.white, this.closed = 0});

  final Shape shape;
  final Color color;
  final Color eyes;

  /// 0 = open, 1 = shut.
  final double closed;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 100, size.height / 100);
    final fill = Paint()..color = color;
    canvas.drawPath(bodyPath(shape), fill);
    if (shape == Shape.hexagon) {
      canvas.drawPath(
        bodyPath(shape),
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 12
          ..strokeJoin = StrokeJoin.round,
      );
    }
    final y = _eyeTop[shape]!;
    final eye = Paint()
      ..color = eyes
      ..strokeCap = StrokeCap.round;
    if (closed > 0.5) {
      eye.strokeWidth = 7;
      canvas.drawLine(Offset(38, y + 8), Offset(46, y + 8), eye);
      canvas.drawLine(Offset(55, y + 8), Offset(63, y + 8), eye);
    } else {
      eye.strokeWidth = 8;
      final squash = 1 - closed;
      final mid = y + 6.5;
      canvas.drawLine(Offset(41, mid - 6.5 * squash), Offset(43, mid + 6.5 * squash), eye);
      canvas.drawLine(Offset(58, mid - 6.5 * squash), Offset(60, mid + 6.5 * squash), eye);
    }
  }

  @override
  bool shouldRepaint(FacePainter old) => old.shape != shape || old.color != color || old.closed != closed || old.eyes != eyes;
}

Path bodyPath(Shape shape) {
  switch (shape) {
    case Shape.circle:
      return Path()..addOval(Rect.fromCircle(center: const Offset(50, 50), radius: 46));
    case Shape.blob:
      return Path()..addOval(Rect.fromCenter(center: const Offset(50, 52), width: 96, height: 80));
    case Shape.square:
      return Path()..addRRect(RRect.fromLTRBR(7, 7, 93, 93, const Radius.circular(24)));
    case Shape.pill:
      return Path()..addRRect(RRect.fromLTRBR(2, 22, 98, 78, const Radius.circular(28)));
    case Shape.triangle:
      return Path()
        ..moveTo(50, 9)
        ..quadraticBezierTo(56, 9, 60, 15)
        ..lineTo(95, 77)
        ..quadraticBezierTo(100, 90, 86, 90)
        ..lineTo(14, 90)
        ..quadraticBezierTo(0, 90, 5, 77)
        ..lineTo(40, 15)
        ..quadraticBezierTo(44, 9, 50, 9)
        ..close();
    case Shape.hexagon:
      return Path()
        ..moveTo(50, 8)
        ..lineTo(88, 30)
        ..lineTo(88, 70)
        ..lineTo(50, 92)
        ..lineTo(12, 70)
        ..lineTo(12, 30)
        ..close();
    case Shape.cloud:
      return Path()
        ..addOval(Rect.fromCircle(center: const Offset(30, 58), radius: 22))
        ..addOval(Rect.fromCircle(center: const Offset(54, 40), radius: 26))
        ..addOval(Rect.fromCircle(center: const Offset(76, 58), radius: 21))
        ..addRRect(RRect.fromLTRBR(28, 50, 78, 80, const Radius.circular(8)))
        ..addOval(Rect.fromCircle(center: const Offset(50, 66), radius: 16));
    case Shape.drop:
      return Path()
        ..moveTo(50, 3)
        ..cubicTo(62, 20, 88, 44, 88, 62)
        ..arcToPoint(const Offset(12, 62), radius: const Radius.circular(38), clockwise: true)
        ..cubicTo(12, 44, 38, 20, 50, 3)
        ..close();
  }
}

/// A face that blinks on its own and hops while [working].
class BotFace extends StatefulWidget {
  const BotFace({super.key, required this.shape, required this.color, this.size = 44, this.working = false, this.hop = 4, this.eyes});

  final Shape shape;
  final int color;
  final double size;
  final bool working;
  final double hop;
  final Color? eyes;

  @override
  State<BotFace> createState() => _BotFaceState();
}

class _BotFaceState extends State<BotFace> with TickerProviderStateMixin {
  late final AnimationController _blink = AnimationController(vsync: this, duration: const Duration(milliseconds: 180));
  late final AnimationController _hop = AnimationController(vsync: this, duration: const Duration(milliseconds: 900));
  final _rng = math.Random();
  bool _alive = true;

  @override
  void initState() {
    super.initState();
    _scheduleBlink();
    if (widget.working) _hop.repeat(reverse: true);
  }

  Timer? _timer;

  void _scheduleBlink() {
    _timer = Timer(Duration(milliseconds: 2200 + _rng.nextInt(4200)), () async {
      if (!_alive || !mounted) return;
      await _blink.forward();
      if (!_alive || !mounted) return;
      await _blink.reverse();
      if (_alive && mounted) _scheduleBlink();
    });
  }

  @override
  void didUpdateWidget(BotFace old) {
    super.didUpdateWidget(old);
    if (widget.working && !_hop.isAnimating) {
      _hop.repeat(reverse: true);
    } else if (!widget.working && _hop.isAnimating) {
      _hop.animateTo(0, duration: const Duration(milliseconds: 200));
    }
  }

  @override
  void dispose() {
    _alive = false;
    _timer?.cancel();
    _blink.dispose();
    _hop.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: Listenable.merge([_blink, _hop]),
      builder: (context, _) {
        final lift = Curves.easeInOutCubic.transform(_hop.value) * widget.hop;
        return Transform.translate(
          offset: Offset(0, -lift),
          child: TweenAnimationBuilder<Color?>(
            tween: ColorTween(end: botColor(widget.color)),
            duration: const Duration(milliseconds: 300),
            builder: (context, color, _) => CustomPaint(
              size: Size.square(widget.size),
              painter: FacePainter(widget.shape, color ?? botColor(widget.color), eyes: widget.eyes ?? Colors.white, closed: _blink.value),
            ),
          ),
        );
      },
    );
  }
}

/// A small coloured dot on the face: green idle, orange needs you.
class StatusDot extends StatelessWidget {
  const StatusDot({super.key, required this.status, required this.ring});
  final BotStatus status;
  final Color ring;

  @override
  Widget build(BuildContext context) {
    final p = context.p;
    final color = switch (status) {
      BotStatus.idle => p.online,
      BotStatus.needsYou => p.warning,
      BotStatus.working => Colors.transparent,
    };
    return AnimatedScale(
      scale: status == BotStatus.working ? 0 : 1,
      duration: const Duration(milliseconds: 220),
      curve: Curves.easeOutBack,
      child: Container(
        width: 13,
        height: 13,
        decoration: BoxDecoration(
          color: color,
          shape: BoxShape.circle,
          border: Border.all(color: ring, width: 2),
        ),
      ),
    );
  }
}
