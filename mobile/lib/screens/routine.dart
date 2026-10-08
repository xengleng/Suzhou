// A routine: on/off, test run, name, instruction, when to run, history.

import 'package:flutter/material.dart';

import '../models.dart';
import '../state.dart';
import '../theme.dart';
import '../widgets/kit.dart';

const _schedules = [
  'Every day at 9:00 AM',
  'Every weekday at 9:00 AM',
  'Every Monday at 8:00 AM',
  'Every day at 10:01 AM',
  'Every hour',
  'Every Friday at 5:00 PM',
];

class RoutineScreen extends StatefulWidget {
  const RoutineScreen({super.key, required this.bot, required this.routine});
  final Bot bot;
  final Routine routine;

  static Route<void> route(Bot bot, Routine r) => MaterialPageRoute(
    builder: (_) => RoutineScreen(bot: bot, routine: r),
  );

  @override
  State<RoutineScreen> createState() => _RoutineScreenState();
}

class _RoutineScreenState extends State<RoutineScreen> {
  late final _name = TextEditingController(text: widget.routine.name);
  late final _instruction = TextEditingController(text: widget.routine.instruction);

  Routine get r => widget.routine;

  @override
  void dispose() {
    _name.dispose();
    _instruction.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context);
    final p = context.p;
    final running = r.history.isNotEmpty && r.history.first.status == RunStatus.running;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Routine'),
        actions: [
          IconButton(
            tooltip: 'Delete',
            icon: Icon(Icons.delete_outline_rounded, color: p.danger),
            onPressed: () {
              state.deleteRoutine(widget.bot, r);
              Navigator.pop(context);
            },
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 40),
        children: [
          Row(
            children: [
              Switch(
                value: r.active,
                onChanged: (v) {
                  r.active = v;
                  state.save();
                },
              ),
              const SizedBox(width: 8),
              Expanded(
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 200),
                  child: Text(r.active ? 'Active' : 'Paused', key: ValueKey(r.active), style: const TextStyle(fontSize: 16)),
                ),
              ),
              PrimaryButton(running ? 'Running…' : 'Test run', onPressed: running ? null : () => state.testRun(widget.bot, r)),
            ],
          ),
          const SectionLabel('Name'),
          Field(
            controller: _name,
            onChanged: (v) {
              r.name = v;
              state.save();
            },
          ),
          const SectionLabel('Instruction'),
          Field(
            controller: _instruction,
            maxLines: 10,
            hint: 'What should happen each time it runs?',
            onChanged: (v) {
              r.instruction = v;
              state.save();
            },
          ),
          const SectionLabel('When to run'),
          Container(
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: p.border),
            ),
            child: Column(
              children: [
                for (final (i, s) in r.schedules.indexed)
                  ListTile(
                    leading: Icon(Icons.schedule_rounded, color: p.text),
                    title: Text.rich(
                      TextSpan(
                        children: [
                          TextSpan(text: s.split(' ').first),
                          TextSpan(
                            text: s.contains(' ') ? s.substring(s.indexOf(' ')) : '',
                            style: TextStyle(color: p.muted),
                          ),
                        ],
                      ),
                    ),
                    trailing: r.schedules.length > 1
                        ? IconButton(
                            icon: const Icon(Icons.close_rounded, size: 18),
                            onPressed: () {
                              r.schedules.removeAt(i);
                              state.save();
                            },
                          )
                        : null,
                    onTap: () {
                      final at = _schedules.indexOf(s);
                      r.schedules[i] = _schedules[(at + 1) % _schedules.length];
                      state.save();
                    },
                  ),
                ListTile(
                  leading: Icon(Icons.add_rounded, color: p.muted),
                  title: Text('Add another', style: TextStyle(color: p.muted)),
                  onTap: () {
                    r.schedules.add(_schedules.firstWhere((s) => !r.schedules.contains(s), orElse: () => _schedules.first));
                    state.save();
                  },
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.only(top: 6, left: 4),
            child: Text('Tap a time to change it. Times are in your local time zone.', style: TextStyle(fontSize: 12, color: p.faint)),
          ),
          const SectionLabel('Run history'),
          if (r.history.isEmpty)
            Padding(
              padding: const EdgeInsets.only(left: 4),
              child: Text('No runs yet', style: TextStyle(color: p.muted)),
            ),
          for (final h in r.history)
            Arrive(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 4),
                child: Row(
                  children: [
                    AnimatedSwitcher(
                      duration: const Duration(milliseconds: 250),
                      child: switch (h.status) {
                        RunStatus.running => const SmallSpinner(key: ValueKey(0), size: 16),
                        RunStatus.succeeded => Icon(Icons.check_rounded, key: const ValueKey(1), size: 18, color: p.online),
                        RunStatus.failed => Icon(Icons.close_rounded, key: const ValueKey(2), size: 18, color: p.danger),
                      },
                    ),
                    const SizedBox(width: 10),
                    Expanded(child: Text(h.when)),
                    Text(switch (h.status) {
                      RunStatus.running => 'Running',
                      RunStatus.succeeded => 'Completed',
                      RunStatus.failed => 'Failed',
                    }, style: TextStyle(fontSize: 13, color: p.muted)),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}
