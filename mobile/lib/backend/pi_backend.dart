// Pi's RPC protocol, over a WebSocket.
//
// Each Bot opens `ws://<bridge>/bot/<id>`. The bridge (tool/pi_bridge.dart)
// runs one `pi --mode rpc` per Bot on the computer where Pi is installed and
// relays JSON lines both ways. The event mapping matches the desktop app's
// src/backend/pi.rs and was written from Pi's RPC docs, not tested against a
// live Pi: unknown events are ignored.

import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import 'backend.dart';

class PiBackend implements Backend {
  PiBackend(this.bridgeUrl);

  /// For example `ws://192.168.1.20:8787`.
  final String bridgeUrl;
  final _events = StreamController<BotEvent>.broadcast();
  final _sockets = <int, WebSocketChannel>{};
  final _pendingUi = <int, List<(String, bool)>>{}; // (id, isConfirm)

  @override
  String get name => 'Pi';

  @override
  Stream<BotEvent> get events => _events.stream;

  WebSocketChannel _socket(int bot) {
    return _sockets.putIfAbsent(bot, () {
      final ch = WebSocketChannel.connect(Uri.parse('$bridgeUrl/bot/$bot'));
      var streamed = false;
      ch.stream.listen(
        (data) {
          for (final line in const LineSplitter().convert(data.toString())) {
            if (line.trim().isEmpty) continue;
            final Object? json;
            try {
              json = jsonDecode(line);
            } catch (_) {
              continue;
            }
            if (json is! Map<String, dynamic>) continue;
            final (events, nowStreamed) = translate(bot, json, streamed, _pendingUi);
            streamed = nowStreamed;
            events.forEach(_events.add);
          }
        },
        onError: (Object e) {
          _events.add(Failed(bot, "Couldn't reach the Pi bridge at $bridgeUrl: $e"));
          _events.add(Finished(bot));
          _sockets.remove(bot);
        },
        onDone: () => _sockets.remove(bot),
      );
      return ch;
    });
  }

  void _write(int bot, Map<String, dynamic> command) {
    try {
      _socket(bot).sink.add('${jsonEncode(command)}\n');
    } catch (e) {
      _events.add(Failed(bot, 'Lost the connection to Pi: $e'));
      _events.add(Finished(bot));
    }
  }

  bool _answerUi(int bot, bool confirm, Object value) {
    final queue = _pendingUi[bot];
    final at = queue?.indexWhere((p) => p.$2 == confirm) ?? -1;
    if (queue == null || at < 0) return false;
    final (id, _) = queue.removeAt(at);
    _write(bot, {'type': 'extension_ui_response', 'id': id, if (confirm) 'confirmed': value else 'value': value});
    return true;
  }

  @override
  void send(Request request) {
    switch (request) {
      case PromptRequest(:final bot, :final persona, :final text, :final first):
        _write(bot, {'type': 'prompt', 'message': first ? '${persona.preamble}\n\n---\n\n$text' : text});
      case AnswerRequest(:final bot, :final text):
        if (!_answerUi(bot, false, text)) _write(bot, {'type': 'prompt', 'message': text});
      case ApprovalRequest(:final bot, :final approved):
        if (!_answerUi(bot, true, approved)) {
          _write(bot, {'type': 'prompt', 'message': approved ? 'Approved. Go ahead.' : "Denied. Don't do that."});
        }
      case HandBackRequest(:final bot):
        _write(bot, {'type': 'prompt', 'message': "I've finished on the computer and handed control back. Please continue."});
      case TestRunRequest(:final bot, :final persona, :final instruction):
        _write(bot, {'type': 'prompt', 'message': '${persona.preamble}\n\n---\n\nRoutine test run. $instruction'});
      case AbortRequest(:final bot):
        _write(bot, {'type': 'abort'});
      case ForgetRequest(:final bot):
        _sockets.remove(bot)?.sink.close();
        _pendingUi.remove(bot);
    }
  }

  @override
  void dispose() {
    for (final s in _sockets.values) {
      s.sink.close();
    }
    _events.close();
  }
}

String _toolLabel(String name, Map<String, dynamic>? args) {
  String? detail;
  for (final key in ['command', 'path', 'file_path', 'url', 'query', 'pattern']) {
    final v = args?[key];
    if (v is String) {
      detail = v.split('\n').first;
      if (detail.length > 80) detail = detail.substring(0, 80);
      break;
    }
  }
  return switch ((name, detail)) {
    ('bash', final String cmd) => 'Ran `$cmd`',
    ('read', final String p) => 'Read $p',
    ('write', final String p) => 'Wrote $p',
    ('edit', final String p) => 'Edited $p',
    (_, final String d) => '$name: $d',
    _ => 'Used $name',
  };
}

/// One line of Pi output -> UI events. Returns the events and whether text
/// has streamed in this message (so `message_end` doesn't repeat it).
(List<BotEvent>, bool) translate(int bot, Map<String, dynamic> v, bool streamed, Map<int, List<(String, bool)>> pendingUi) {
  final type = v['type'];
  switch (type) {
    case 'agent_start':
      return ([Started(bot)], false);
    case 'agent_end':
      return ([Finished(bot)], streamed);
    case 'message_start':
      return (const [], false);
    case 'message_update':
      final inner = v['assistantMessageEvent'];
      if (inner is Map && inner['type'] == 'text_delta' && inner['delta'] is String) {
        return ([Delta(bot, inner['delta'] as String)], true);
      }
      return (const [], streamed);
    case 'message_end':
      final m = v['message'];
      if (m is! Map || m['role'] != 'assistant' || streamed) return (const [], streamed);
      final parts = (m['content'] as List? ?? const []).whereType<Map>().where((p) => p['type'] == 'text').map((p) => p['text']).whereType<String>();
      final text = parts.join('\n\n');
      return (text.isEmpty ? const [] : [Delta(bot, text)], streamed);
    case 'tool_execution_start' || 'tool_execution_end':
      final name = v['toolName'] as String? ?? 'tool';
      final label = _toolLabel(name, (v['args'] as Map?)?.cast<String, dynamic>());
      final end = type == 'tool_execution_end';
      return ([Activity(bot, label, done: end), if (end && v['isError'] == true) Note(bot, '$name failed')], streamed);
    case 'extension_ui_request':
      final id = v['id']?.toString() ?? '';
      final title = (v['title'] ?? v['message'] ?? 'Pi needs you').toString();
      final detail = (v['message'] ?? '').toString();
      switch (v['method']) {
        case 'confirm':
          pendingUi.putIfAbsent(bot, () => []).add((id, true));
          return ([ApprovalNeeded(bot, title, detail)], streamed);
        case 'input' || 'editor' || 'select':
          pendingUi.putIfAbsent(bot, () => []).add((id, false));
          final options = (v['options'] as List?)?.whereType<String>().join(' / ') ?? '';
          return ([Question(bot, title, options.isNotEmpty ? options : (v['placeholder'] ?? 'Answer').toString())], streamed);
        case 'notify':
          return ([Note(bot, title)], streamed);
      }
      return (const [], streamed);
    case 'response':
      if (v['success'] == false) {
        return ([Failed(bot, (v['error'] ?? 'Pi refused the request').toString()), Finished(bot)], streamed);
      }
  }
  return (const [], streamed);
}
