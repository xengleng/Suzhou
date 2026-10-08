// Settings: appearance, backend, notifications, approvals, computer, account.

import 'package:flutter/material.dart';

import '../state.dart';
import '../theme.dart';
import '../widgets/kit.dart';
import 'home.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  static Route<void> route() => MaterialPageRoute(builder: (_) => const SettingsScreen());

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final s = state.settings;
    final p = context.p;

    Widget toggle(String title, String detail, bool value, void Function(bool) set) => SwitchListTile(
      contentPadding: const EdgeInsets.symmetric(horizontal: 4),
      title: Text(title),
      subtitle: Text(detail, style: TextStyle(color: p.muted)),
      value: value,
      onChanged: (v) {
        set(v);
        state.save();
      },
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 0, 16, 40),
        children: [
          const SizedBox(height: 8),
          Row(
            children: [
              UserAvatar(state.userName, size: 52),
              const SizedBox(width: 14),
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(state.userName, style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w600)),
                  Text('${state.bots.length} Bots', style: TextStyle(color: p.muted)),
                ],
              ),
            ],
          ),
          const SectionLabel('Appearance'),
          SegmentedButton<ThemeMode>(
            showSelectedIcon: false,
            segments: const [
              ButtonSegment(value: ThemeMode.system, label: Text('System')),
              ButtonSegment(value: ThemeMode.light, label: Text('Light')),
              ButtonSegment(value: ThemeMode.dark, label: Text('Dark')),
            ],
            selected: {state.themeMode},
            onSelectionChanged: (v) {
              state.themeMode = v.first;
              state.save();
            },
          ),
          const SectionLabel('Backend'),
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 4),
            title: Text(state.backend.name == 'Pi' ? 'Pi' : 'Demo Bot (built in)'),
            subtitle: Text(state.piBridge.isEmpty ? 'Simulated replies so you can try everything.' : state.piBridge, style: TextStyle(color: p.muted)),
            trailing: const Icon(Icons.chevron_right_rounded),
            onTap: () => _editBridge(context, state),
          ),
          const SectionLabel('Notifications'),
          toggle('Notifications', 'A card at the top when a Bot finishes, asks or needs approval.', s.notifications, (v) => s.notifications = v),
          toggle('Sounds', 'Play a soft sound with each notification.', s.sounds, (v) => s.sounds = v),
          toggle('Only when a Bot needs me', 'Skip “finished” cards; keep questions and approvals.', s.onlyWhenNeeded, (v) => s.onlyWhenNeeded = v),
          const SectionLabel('Approvals'),
          toggle('Auto Review', 'Let a reviewer model approve low-risk actions.', s.autoReview, (v) => s.autoReview = v),
          toggle('Ask before sending messages', 'Email, chat and invitations.', s.askSending, (v) => s.askSending = v),
          toggle('Ask before purchases', 'Anything that spends money.', s.askPurchases, (v) => s.askPurchases = v),
          toggle('Ask before publishing', 'Posts, pushes, deploys and public changes.', s.askPublishing, (v) => s.askPublishing = v),
          const SectionLabel('Computer'),
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 4),
            leading: CircleAvatar(radius: 5, backgroundColor: p.online),
            title: const Text('Running'),
            subtitle: Text('One cloud computer; every Bot has its own screen.', style: TextStyle(color: p.muted)),
          ),
          Row(
            children: [
              SecondaryButton(
                'Recover',
                onPressed: () => ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Restarted the computer’s apps. Files kept.'))),
              ),
              const SizedBox(width: 8),
              SecondaryButton(
                'Reset…',
                danger: true,
                onPressed: () => ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Reset is a last resort. Try Recover first.'))),
              ),
            ],
          ),
          const SectionLabel('Account'),
          SheetRow(
            Icons.logout_rounded,
            'Sign out',
            onTap: () {
              Navigator.of(context).popUntil((r) => r.isFirst);
              state.signOut();
            },
          ),
        ],
      ),
    );
  }

  Future<void> _editBridge(BuildContext context, AppState state) async {
    final controller = TextEditingController(text: state.piBridge);
    final result = await showDialog<String>(
      context: context,
      builder: (d) => AlertDialog(
        title: const Text('Connect to Pi'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Run tool/pi_bridge.dart on the computer where Pi is installed, then enter its address. Leave empty for the demo Bot.'),
            const SizedBox(height: 12),
            Field(controller: controller, hint: 'ws://192.168.1.20:8787'),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(d), child: const Text('Cancel')),
          TextButton(onPressed: () => Navigator.pop(d, controller.text), child: const Text('Save')),
        ],
      ),
    );
    if (result != null) state.setPiBridge(result);
  }
}
