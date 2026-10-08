import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:suzhou/main.dart';
import 'package:suzhou/screens/chat.dart';
import 'package:suzhou/state.dart';

Future<void> settle(WidgetTester t, [int ms = 1200]) async {
  for (var i = 0; i < ms ~/ 100; i++) {
    await t.pump(const Duration(milliseconds: 100));
  }
}

void main() {
  setUp(() => SharedPreferences.setMockInitialValues({}));

  testWidgets('first run: sign in through to the New Bot screen', (t) async {
    t.view.physicalSize = const Size(390 * 3, 844 * 3);
    t.view.devicePixelRatio = 3;
    final state = AppState();
    await state.load();
    await t.pumpWidget(SuzhouApp(state: state));
    await settle(t);
    expect(find.text('Suzhou'), findsOneWidget);

    await t.tap(find.text('Sign in'));
    await settle(t);
    expect(find.text('Finish signing in in your browser'), findsOneWidget);
    await t.tap(find.text("I've signed in"));
    await settle(t);
    expect(find.text('What do you use for work?'), findsOneWidget);
    await t.ensureVisible(find.text('Continue'));
    await settle(t, 300);
    await t.tap(find.text('Continue'));
    await settle(t, 4200);
    expect(find.text('Meet a future teammate'), findsOneWidget);
    await t.tap(find.text('Create your first Bot'));
    await settle(t, 1500);
    expect(find.text('Get started'), findsOneWidget);

    await t.enterText(find.byType(TextField).first, 'Blog Pulse');
    await settle(t, 300);
    await t.tap(find.text('Get started'));
    await settle(t, 1500);
    expect(find.byType(ChatScreen), findsOneWidget);
    expect(find.text('Blog Pulse joined your team'), findsOneWidget);
    await settle(t, 3000);
    await t.pumpWidget(const SizedBox());
    await settle(t, 6000);
  });

  testWidgets('demo team: open a chat, send, get a reply', (t) async {
    t.view.physicalSize = const Size(390 * 3, 844 * 3);
    t.view.devicePixelRatio = 3;
    final state = AppState(demo: true);
    await state.load();
    await t.pumpWidget(SuzhouApp(state: state));
    await settle(t);
    expect(find.text('Inbox Triage'), findsOneWidget);

    await t.tap(find.text('Inbox Triage'));
    await settle(t);
    expect(find.text('Send 3 drafted replies'), findsOneWidget);
    await t.tap(find.text('Approve'));
    await settle(t, 3500);
    expect(find.text('Approved'), findsOneWidget);
    expect(find.textContaining("Sent. I'll watch for replies"), findsOneWidget);

    await t.enterText(find.byType(TextField).last, 'hello there');
    await settle(t, 300);
    await t.tap(find.byIcon(Icons.arrow_upward_rounded));
    await settle(t, 6000);
    // The reply quotes the message back.
    expect(find.textContaining('things stand on \u201chello there\u201d', findRichText: true), findsOneWidget);
    await t.pumpWidget(const SizedBox());
    await settle(t, 6000);
  });
}
