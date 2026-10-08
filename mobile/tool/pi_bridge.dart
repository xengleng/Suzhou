// Relays the phone app to Pi.
//
// Run this on the computer where `pi` is installed:
//
//   dart run tool/pi_bridge.dart --port 8787 [-- extra pi arguments]
//
// Then in the app, Settings > Backend > Pi, enter ws://<this computer>:8787.
// Each Bot connects to /bot/<id>; the bridge starts `pi --mode rpc` for it
// in its own folder (~/.local/share/suzhou/bots/<id>) and pipes JSON lines
// both ways. There is no authentication: only run it on a network you trust,
// or put it behind a tunnel that adds one.

import 'dart:convert';
import 'dart:io';

Future<void> main(List<String> argv) async {
  var port = 8787;
  final extra = <String>[];
  for (var i = 0; i < argv.length; i++) {
    if (argv[i] == '--port' && i + 1 < argv.length) {
      port = int.parse(argv[++i]);
    } else if (argv[i] == '--') {
      extra.addAll(argv.skip(i + 1));
      break;
    }
  }
  final bin = Platform.environment['SUZHOU_PI_BIN'] ?? 'pi';
  final home = Platform.environment['HOME'] ?? Directory.systemTemp.path;
  final server = await HttpServer.bind(InternetAddress.anyIPv4, port);
  stdout.writeln('Pi bridge listening on ws://0.0.0.0:$port (pi: $bin ${extra.join(' ')})');

  await for (final req in server) {
    final match = RegExp(r'^/bot/(\d+)$').firstMatch(req.uri.path);
    if (match == null || !WebSocketTransformer.isUpgradeRequest(req)) {
      req.response
        ..statusCode = HttpStatus.notFound
        ..close();
      continue;
    }
    final bot = match.group(1)!;
    final socket = await WebSocketTransformer.upgrade(req);
    final dir = Directory('$home/.local/share/suzhou/bots/$bot')..createSync(recursive: true);
    Process proc;
    try {
      proc = await Process.start(bin, ['--mode', 'rpc', ...extra], workingDirectory: dir.path);
    } catch (e) {
      socket.add(jsonEncode({'type': 'response', 'success': false, 'error': "Couldn't start `$bin` on the bridge: $e"}));
      await socket.close();
      continue;
    }
    stdout.writeln('bot $bot: started pi (pid ${proc.pid})');
    proc.stdout.transform(utf8.decoder).transform(const LineSplitter()).listen(socket.add, onDone: () => socket.close());
    proc.stderr.transform(utf8.decoder).transform(const LineSplitter()).listen((l) => stderr.writeln('pi[$bot]: $l'));
    socket.listen(
      (data) => proc.stdin.write(data.toString().endsWith('\n') ? data : '$data\n'),
      onDone: () {
        stdout.writeln('bot $bot: disconnected, stopping pi');
        proc.kill();
      },
    );
  }
}
