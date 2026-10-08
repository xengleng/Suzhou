// A file a Bot sent, shown as a readable page.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../theme.dart';
import '../widgets/rich.dart';

class FileViewer extends StatelessWidget {
  const FileViewer({super.key, required this.name, required this.content});
  final String name, content;

  static Route<void> route(String name, String content) => MaterialPageRoute(
    builder: (_) => FileViewer(name: name, content: content),
    fullscreenDialog: true,
  );

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(tooltip: 'Close', icon: const Icon(Icons.close_rounded), onPressed: () => Navigator.pop(context)),
        title: Text(name),
        actions: [
          IconButton(
            tooltip: 'Copy',
            icon: const Icon(Icons.copy_rounded),
            onPressed: () {
              Clipboard.setData(ClipboardData(text: content));
              ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Copied $name')));
            },
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(22, 18, 22, 40),
        child: name.endsWith('.md')
            ? Markdown(content, size: 16)
            : Text(
                content,
                style: TextStyle(fontFamily: kMono, fontSize: 13, color: context.p.text),
              ),
      ),
    );
  }
}
