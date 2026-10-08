import 'package:flutter_test/flutter_test.dart';
import 'package:suzhou/backend/backend.dart';
import 'package:suzhou/backend/mock_backend.dart';
import 'package:suzhou/backend/pi_backend.dart';
import 'package:suzhou/widgets/rich.dart';

void main() {
  group('parseInline', () {
    test('finds code, links and bold', () {
      final parts = parseInline('See `rss.xml` at https://example.com. **Done**');
      expect(parts.map((p) => p.$1).join(), 'See rss.xml at https://example.com. Done');
      expect(parts, contains(('rss.xml', SpanKind.code)));
      expect(parts, contains(('https://example.com', SpanKind.link)));
      expect(parts, contains(('Done', SpanKind.bold)));
    });

    test('leaves unclosed markers alone', () {
      expect(parseInline('a ` b ** c').map((p) => p.$1).join(), 'a ` b ** c');
    });
  });

  group('Pi translate', () {
    List<BotEvent> run(List<Map<String, dynamic>> lines) {
      final pending = <int, List<(String, bool)>>{};
      var streamed = false;
      final out = <BotEvent>[];
      for (final l in lines) {
        final (events, s) = translate(7, l, streamed, pending);
        streamed = s;
        out.addAll(events);
      }
      return out;
    }

    test('streams text and tools', () {
      final e = run([
        {'type': 'agent_start'},
        {
          'type': 'message_start',
          'message': {'role': 'assistant'},
        },
        {
          'type': 'message_update',
          'assistantMessageEvent': {'type': 'text_delta', 'delta': 'Hi'},
        },
        {
          'type': 'message_end',
          'message': {
            'role': 'assistant',
            'content': [
              {'type': 'text', 'text': 'Hi'},
            ],
          },
        },
        {
          'type': 'tool_execution_start',
          'toolName': 'bash',
          'args': {'command': 'ls -la'},
        },
        {
          'type': 'tool_execution_end',
          'toolName': 'bash',
          'args': {'command': 'ls -la'},
          'isError': false,
        },
        {'type': 'agent_end'},
      ]);
      expect(e.length, 5);
      expect(e[0], isA<Started>());
      expect((e[1] as Delta).text, 'Hi');
      expect((e[2] as Activity).label, 'Ran `ls -la`');
      expect((e[3] as Activity).done, isTrue);
      expect(e[4], isA<Finished>());
    });

    test('confirm becomes an approval', () {
      final e = run([
        {'type': 'extension_ui_request', 'id': 'u1', 'method': 'confirm', 'title': 'Delete file?', 'message': 'rm notes.md'},
      ]);
      expect((e.single as ApprovalNeeded).action, 'Delete file?');
    });

    test('failed command reports an error', () {
      final e = run([
        {'type': 'response', 'command': 'prompt', 'success': false, 'error': 'no model'},
      ]);
      expect((e.first as Failed).message, 'no model');
    });
  });

  test('mock backend answers a prompt with a streamed reply', () async {
    final backend = MockBackend();
    final events = <BotEvent>[];
    final sub = backend.events.listen(events.add);
    backend.send(const PromptRequest(1, Persona('Pulse', '', ''), 'hello there', first: true));
    await Future<void>.delayed(const Duration(seconds: 6));
    await sub.cancel();
    expect(events.first, isA<Started>());
    expect(events.last, isA<Finished>());
    final text = events.whereType<Delta>().map((d) => d.text).join();
    expect(text, startsWith('On it.'));
  });
}
