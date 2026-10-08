// Fixed content: suggestions, the plugin catalogue and the demo team.
// Kept in step with src/data.rs in the desktop app.

import 'models.dart';

const _months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const _days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

String clock(DateTime t) {
  final h = t.hour % 12 == 0 ? 12 : t.hour % 12;
  final m = t.minute.toString().padLeft(2, '0');
  return '$h:$m ${t.hour < 12 ? 'AM' : 'PM'}';
}

String nowTime() => clock(DateTime.now());
String todayStamp() => 'Today ${nowTime()}';
String nowFull() {
  final t = DateTime.now();
  return '${_days[t.weekday - 1]} ${t.day} ${_months[t.month - 1]}, ${clock(t)}';
}

String isoDate() {
  final t = DateTime.now();
  return '${t.year}-${t.month.toString().padLeft(2, '0')}-${t.day.toString().padLeft(2, '0')}';
}

class Suggestion {
  const Suggestion(this.name, this.blurb, this.title, this.description, this.color, this.shape);
  final String name, blurb, title, description;
  final int color;
  final Shape shape;
}

const suggestions = [
  Suggestion(
    'Night Shift',
    'Works overnight and preps your morning digest',
    'Overnight researcher',
    'Works while you sleep. Reads what came in overnight, checks the sites and dashboards you care about, and leaves a short digest for the morning.',
    2,
    Shape.hexagon,
  ),
  Suggestion(
    'Inbox Triage',
    'Sorts your email and drafts replies in your voice',
    'Email assistant',
    'Sorts new email into Act, Read and Ignore, drafts replies in your voice, and never sends anything without asking first.',
    8,
    Shape.cloud,
  ),
  Suggestion(
    'Chief of Staff',
    'Keeps the team on track and runs your week',
    'Chief of staff',
    'Keeps a running list of what every other Bot is doing, chases what is stuck, and sends a short plan for the week every Monday.',
    0,
    Shape.square,
  ),
  Suggestion(
    'Deal Desk',
    'Watches the pipeline and preps every call',
    'Sales analyst',
    'Reads the CRM, preps a one-page brief before each call, and flags deals that have gone quiet.',
    4,
    Shape.drop,
  ),
];

const workTools = [
  ('gmail', 'Gmail'),
  ('gcal', 'Google Calendar'),
  ('gdrive', 'Google Drive'),
  ('slack', 'Slack'),
  ('github', 'GitHub'),
  ('notion', 'Notion'),
  ('linear', 'Linear'),
  ('figma', 'Figma'),
];

const pluginCategories = [
  'All',
  'Featured',
  'Team plugins',
  'Agent Orchestration',
  'Canvas',
  'Customer Support',
  'Data Analytics',
  'Design',
  'Finance And Legal',
  'Inbox And Collaboration',
  'Infrastructure',
  'MCP',
  'Payments',
  'Productivity',
  'Research',
  'Sales',
  'Scheduling',
];

List<Plugin> plugins() {
  final list = [
    Plugin('gmail', 'Gmail', 'Search, read, draft, and manage email.', ['Inbox And Collaboration', 'Productivity'], 'M', 0xFFEA4335, featured: true),
    Plugin('gcal', 'Google Calendar', 'Search events and schedule meetings.', ['Scheduling', 'Productivity'], '31', 0xFF1A73E8, featured: true),
    Plugin('gdrive', 'Google Drive', 'Search, read, create, and share files.', ['Productivity'], 'D', 0xFF0F9D58, featured: true),
    Plugin(
      'granola',
      'Granola',
      'Your meetings in your workflow. Notes, transcripts and action items.',
      ['Productivity', 'Research'],
      'G',
      0xFF9DBB3A,
      featured: true,
    ),
    Plugin('slack', 'Slack', 'Read channels, search threads and post updates.', ['Inbox And Collaboration'], '#', 0xFF611F69),
    Plugin('github', 'GitHub', 'Read repositories, open pull requests and triage issues.', ['Infrastructure'], 'GH', 0xFF24292F),
    Plugin('notion', 'Notion', 'Search and edit pages and databases.', ['Productivity', 'Canvas'], 'N', 0xFF191919),
    Plugin('linear', 'Linear', 'Create, update and triage issues and projects.', ['Productivity', 'Infrastructure'], 'L', 0xFF5E6AD2),
    Plugin('figma', 'Figma', 'Read designs, comments and components.', ['Design', 'Canvas'], 'F', 0xFFA259FF),
    Plugin('cloudflare', 'Cloudflare', 'Workers, Pages deployments and DNS.', ['Infrastructure'], 'CF', 0xFFF38020),
    Plugin('stripe', 'Stripe', 'Look up payments, refunds and subscriptions.', ['Payments', 'Finance And Legal'], 'S', 0xFF635BFF),
    Plugin('hubspot', 'HubSpot', 'Contacts, deals and pipeline reports.', ['Sales', 'Customer Support'], 'H', 0xFFFF7A59),
    Plugin('zendesk', 'Zendesk', 'Read and answer support tickets.', ['Customer Support'], 'Z', 0xFF03363D),
    Plugin('plausible', 'Plausible', 'Privacy-friendly site analytics.', ['Data Analytics'], 'P', 0xFF5850EC),
    Plugin('bigquery', 'BigQuery', 'Run SQL over your warehouse.', ['Data Analytics', 'Infrastructure'], 'BQ', 0xFF4285F4),
    Plugin('research', 'Deep Research', 'Long-form web research with sources.', ['Research'], 'R', 0xFF20808D),
    Plugin('mcp', 'Custom MCP server', 'Connect any Model Context Protocol server by URL.', ['MCP'], '{}', 0xFF444444),
    Plugin('orchestrator', 'Team Lead', 'Lets one Bot hand work to the others in its group.', ['Agent Orchestration'], 'TL', 0xFF0EA5E9),
    Plugin('quickbooks', 'QuickBooks', 'Invoices, bills and reconciliation.', ['Finance And Legal'], 'QB', 0xFF2CA01C),
    Plugin(
      'migration',
      'claude-opus-4-5-migration',
      'Migrate your code and prompts from Sonnet 4.x and Opus 4.1 to Opus 4.5.',
      ['Team plugins'],
      'C',
      0xFF9A9A9A,
      team: true,
    ),
    Plugin('frontend', 'frontend-design', 'Create distinctive, production-grade frontend interfaces.', ['Team plugins', 'Design'], 'F', 0xFF9A9A9A, team: true),
  ];
  for (final p in list) {
    p.installed = const {'cloudflare', 'plausible', 'github', 'linear', 'slack', 'notion'}.contains(p.id);
  }
  return list;
}

List<Skill> defaultSkills() => [
  Skill('daily-site-review', 'Check posts, RSS, deploys and traffic; write a dated report.'),
  Skill('summarize-inbox', 'Group unread email into Act, Read and Ignore.'),
  Skill('weekly-plan', "Turn this week's calendar and tasks into a one-page plan."),
  Skill('research-brief', 'Research a topic and write a sourced one-page brief.'),
];

/// The sample team used with the demo switch, built from the reference screenshots.
List<Bot> demoBots(int Function() nextId) {
  Message msg(
    MsgKind kind, {
    String text = '',
    String time = '',
    bool done = false,
    String? icon,
    String? link,
    int? routine,
    String? fileName,
    String? fileSize,
    String? detail,
  }) => Message(
    id: nextId(),
    kind: kind,
    text: text,
    time: time,
    done: done,
    icon: icon,
    link: link,
    routine: routine,
    fileName: fileName,
    fileSize: fileSize,
    detail: detail,
  );

  final pulse = Bot(
    id: nextId(),
    name: 'Blog Pulse',
    color: 7,
    shape: Shape.pill,
    title: 'Daily site reviewer',
    description: 'Reviews flaviocopes.com every morning: new posts, RSS, deploys and traffic. Separates verified facts from interpretation and links every claim to its source.',
  );
  pulse.pinned = true;
  pulse.skills = [defaultSkills().first];
  final rid = nextId();
  pulse.routines.add(
    Routine(
      id: rid,
      name: 'Daily blog review',
      instruction: "Run the daily flaviocopes.com review as Blog Pulse. Today is the user's local date in Europe/Rome.\n\nDo the full review, then send it in chat and save it to /workspace/blog-pulse/YYYY-MM-DD.md.",
      schedules: ['Every day at 10:01 AM'],
    ),
  );
  pulse.messages = [
    msg(MsgKind.stamp, text: 'Today 10:01 AM'),
    msg(MsgKind.event, text: 'Renamed to Blog Pulse'),
    msg(
      MsgKind.user,
      time: '10:01 AM',
      text: 'You are Blog Pulse, the daily reviewer for https://flaviocopes.com.\nYour job is to prepare a daily review of the site.\nFor every report:\n1. List posts published since the last report.\n2. Check the RSS feed includes them.\n3. Check that the latest deployment finished without errors.\n4. Link every claim to its source.\nDo not publish posts or send emails without approval.',
    ),
    msg(MsgKind.bot, time: '10:02 AM', text: "On it. I'll start today's review of flaviocopes.com and check which sources I can actually reach first."),
    msg(MsgKind.activity, text: 'Opened flaviocopes.com, /rss.xml and the Cloudflare dashboard', done: true),
    msg(MsgKind.bot, time: '10:05 AM', text: '`/index.xml` is a 404. The live feed is `/rss.xml`.'),
    msg(
      MsgKind.bot,
      time: '10:07 AM',
      text: 'What changed: two new posts, RSS path confirmed, rebuild Worker identified.\n\nWhat may need attention: no visitor comparison, and the 11:15 rebuild has not fired yet.\n\nWhat I could not check: Plausible stats and the private repo.\n\nNext: sign in to Plausible and connect GitHub. I will not guess those numbers.',
    ),
    msg(MsgKind.file, fileName: '2026-08-22.md', fileSize: '4.6 KB', text: report),
    msg(MsgKind.event, text: 'Created routine', icon: 'clock', link: 'Daily blog review', routine: rid),
    msg(MsgKind.bot, time: '10:07 AM', text: 'I also set a daily review at 10:01 Europe/Rome, all 7 days, since posts go out on weekends too.'),
  ];

  final inbox = Bot(id: nextId(), name: 'Inbox Triage', color: 8, shape: Shape.cloud, title: 'Email assistant', description: suggestions[1].description);
  inbox.messages = [
    msg(MsgKind.bot, time: '7:30 AM', text: 'Morning. 14 new emails: 3 need you, 6 to read, 5 I archived. Drafts are ready for the three.'),
    msg(
      MsgKind.approval,
      time: '7:31 AM',
      text: 'Send 3 drafted replies',
      detail: 'To: Marta (invoice), Jon (Thursday call), Priya (contract v2). Each draft is in your Gmail drafts folder.',
    ),
  ];
  inbox.status = BotStatus.needsYou;
  inbox.unread = true;

  final scout = Bot(
    id: nextId(),
    name: 'Scout',
    color: 4,
    shape: Shape.hexagon,
    title: 'Learning scout',
    description: 'Finds DataCamp courses that fit my weekly goal and time budget, and reports durations as ranges.',
  );
  scout.messages = [
    msg(MsgKind.user, time: '8:00 AM', text: "Plan this week's learning: 4 hours, intermediate, focus on AI agents."),
    msg(MsgKind.activity, text: 'Browsed datacamp.com/courses/introduction-to-ai-agents', done: true),
    msg(MsgKind.snapshot, text: 'Introduction to AI Agents', link: 'datacamp.com/courses/introduction-to-ai-agents'),
    msg(MsgKind.bot, time: '8:04 AM', text: "Monday DataCamp Learning Plan is on. It runs Mondays at 8:00 AM and uses the saved Skill. I won't fire it now."),
  ];
  scout.routines.add(
    Routine(
      id: nextId(),
      name: 'Monday DataCamp Learning Plan',
      schedules: ['Every Monday at 8:00 AM'],
      history: [RunRecord('Mon 29 Sep, 8:00 AM', RunStatus.succeeded)],
    ),
  );

  final night = Bot(id: nextId(), name: 'Night Shift', color: 2, shape: Shape.hexagon, title: suggestions[0].title, description: suggestions[0].description);
  night.messages = [msg(MsgKind.bot, time: '6:00 AM', text: 'Your morning digest is ready: 2 deploys, 1 failed check, 9 new issues.')];

  return [pulse, inbox, scout, night];
}

const report = '''# Blog Pulse, 23 Aug 2026

Checked at about 10:07 Europe/Rome. Sources I could open are linked. Locked sources are left blank, not guessed.

## Facts

### Today's posts went live

Two posts dated 23 Aug 2026 are on the blog index and in RSS. Both pages returned HTTP 200.

1. Herdr vs tmux vs Zellij. RSS `pubDate`: Sun, 23 Aug 2026 06:00:00 GMT
2. Scroll-driven CSS animations. RSS `pubDate`: Sun, 23 Aug 2026 05:00:00 GMT

### Posts that went live after yesterday's review

- A deep dive into Grok Bot, Sat, 22 Aug 2026
- How to group array items with Object.groupBy(), Sat, 22 Aug 2026
- Swamp tutorial: make AI agent work repeatable, Sat, 22 Aug 2026

## Traffic

Not checked. https://b.flaviocopes.com is still a Plausible login page. No numbers are guessed.

## Next actions

1. Sign in to Plausible in the Agent Computer.
2. Sign in to the Cloudflare Pages dashboard.
3. Add the GitHub plugin so I can read the private repo.
''';
