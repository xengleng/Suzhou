//! Fixed content: the plugin catalogue, suggested Bots, and the demo team.

use crate::model::*;

pub fn now_time() -> String {
    chrono::Local::now().format("%-I:%M %p").to_string()
}

pub fn today_stamp() -> String {
    format!("Today {}", now_time())
}

pub fn now_stamp_full() -> String {
    chrono::Local::now()
        .format("%a %-d %b, %-I:%M %p")
        .to_string()
}

pub struct Suggestion {
    pub name: &'static str,
    pub blurb: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub color: usize,
    pub shape: Shape,
}

/// Cards under "Suggestions" on the New Bot screen.
pub const SUGGESTIONS: &[Suggestion] = &[
    Suggestion {
        name: "Night Shift",
        blurb: "Works overnight and preps your morning digest",
        title: "Overnight researcher",
        description: "Works while you sleep. Reads what came in overnight, checks the sites and \
                      dashboards you care about, and leaves a short digest for the morning.",
        color: 2,
        shape: Shape::Hexagon,
    },
    Suggestion {
        name: "Inbox Triage",
        blurb: "Sorts your email and drafts replies in your voice",
        title: "Email assistant",
        description: "Sorts new email into Act, Read and Ignore, drafts replies in your voice, \
                      and never sends anything without asking first.",
        color: 8,
        shape: Shape::Cloud,
    },
    Suggestion {
        name: "Chief of Staff",
        blurb: "Keeps the team on track and runs your week",
        title: "Chief of staff",
        description: "Keeps a running list of what every other Bot is doing, chases what is \
                      stuck, and sends a short plan for the week every Monday.",
        color: 0,
        shape: Shape::Square,
    },
    Suggestion {
        name: "Deal Desk",
        blurb: "Watches the pipeline and preps every call",
        title: "Sales analyst",
        description: "Reads the CRM, preps a one-page brief before each call, and flags deals \
                      that have gone quiet.",
        color: 4,
        shape: Shape::Drop,
    },
];

/// Apps offered during first run ("What do you use for work?").
pub const WORK_TOOLS: &[(&str, &str)] = &[
    ("gmail", "Gmail"),
    ("gcal", "Google Calendar"),
    ("gdrive", "Google Drive"),
    ("slack", "Slack"),
    ("github", "GitHub"),
    ("notion", "Notion"),
    ("linear", "Linear"),
    ("figma", "Figma"),
];

pub const PLUGIN_CATEGORIES: &[&str] = &[
    "All",
    "Featured",
    "Team plugins",
    "Agent Orchestration",
    "Canvas",
    "Customer Support",
    "Data Analytics",
    "Design",
    "Finance And Legal",
    "Inbox And Collaboration",
    "Infrastructure",
    "MCP",
    "Payments",
    "Productivity",
    "Research",
    "Sales",
    "Scheduling",
];

pub fn plugins() -> Vec<Plugin> {
    let p = |id: &'static str,
             name: &'static str,
             description: &'static str,
             categories: &'static [&'static str],
             glyph: &'static str,
             color: u32,
             featured: bool,
             team: bool| Plugin {
        id,
        name,
        description,
        categories,
        glyph,
        color,
        featured,
        team,
        installed: false,
    };
    let mut list = vec![
        p(
            "gmail",
            "Gmail",
            "Search, read, draft, and manage email.",
            &["Inbox And Collaboration", "Productivity"],
            "M",
            0xEA4335,
            true,
            false,
        ),
        p(
            "gcal",
            "Google Calendar",
            "Search events and schedule meetings.",
            &["Scheduling", "Productivity"],
            "31",
            0x1A73E8,
            true,
            false,
        ),
        p(
            "gdrive",
            "Google Drive",
            "Search, read, create, and share files.",
            &["Productivity"],
            "D",
            0x0F9D58,
            true,
            false,
        ),
        p(
            "granola",
            "Granola",
            "Your meetings in your workflow. Granola notes, transcripts and action items.",
            &["Productivity", "Research"],
            "G",
            0x9DBB3A,
            true,
            false,
        ),
        p(
            "slack",
            "Slack",
            "Read channels, search threads and post updates.",
            &["Inbox And Collaboration"],
            "#",
            0x611F69,
            false,
            false,
        ),
        p(
            "github",
            "GitHub",
            "Read repositories, open pull requests and triage issues.",
            &["Infrastructure"],
            "GH",
            0x24292F,
            false,
            false,
        ),
        p(
            "notion",
            "Notion",
            "Search and edit pages and databases.",
            &["Productivity", "Canvas"],
            "N",
            0x191919,
            false,
            false,
        ),
        p(
            "linear",
            "Linear",
            "Create, update and triage issues and projects.",
            &["Productivity", "Infrastructure"],
            "L",
            0x5E6AD2,
            false,
            false,
        ),
        p(
            "figma",
            "Figma",
            "Read designs, comments and components.",
            &["Design", "Canvas"],
            "F",
            0xA259FF,
            false,
            false,
        ),
        p(
            "cloudflare",
            "Cloudflare",
            "Workers, Pages deployments and DNS.",
            &["Infrastructure"],
            "CF",
            0xF38020,
            false,
            false,
        ),
        p(
            "stripe",
            "Stripe",
            "Look up payments, refunds and subscriptions.",
            &["Payments", "Finance And Legal"],
            "S",
            0x635BFF,
            false,
            false,
        ),
        p(
            "hubspot",
            "HubSpot",
            "Contacts, deals and pipeline reports.",
            &["Sales", "Customer Support"],
            "H",
            0xFF7A59,
            false,
            false,
        ),
        p(
            "zendesk",
            "Zendesk",
            "Read and answer support tickets.",
            &["Customer Support"],
            "Z",
            0x03363D,
            false,
            false,
        ),
        p(
            "plausible",
            "Plausible",
            "Privacy-friendly site analytics.",
            &["Data Analytics"],
            "P",
            0x5850EC,
            false,
            false,
        ),
        p(
            "bigquery",
            "BigQuery",
            "Run SQL over your warehouse.",
            &["Data Analytics", "Infrastructure"],
            "BQ",
            0x4285F4,
            false,
            false,
        ),
        p(
            "perplexity",
            "Deep Research",
            "Long-form web research with sources.",
            &["Research"],
            "R",
            0x20808D,
            false,
            false,
        ),
        p(
            "mcp",
            "Custom MCP server",
            "Connect any Model Context Protocol server by URL.",
            &["MCP"],
            "{}",
            0x444444,
            false,
            false,
        ),
        p(
            "orchestrator",
            "Team Lead",
            "Lets one Bot hand work to the others in its group.",
            &["Agent Orchestration"],
            "TL",
            0x0EA5E9,
            false,
            false,
        ),
        p(
            "quickbooks",
            "QuickBooks",
            "Invoices, bills and reconciliation.",
            &["Finance And Legal"],
            "QB",
            0x2CA01C,
            false,
            false,
        ),
        p(
            "migration",
            "claude-opus-4-5-migration",
            "Migrate your code and prompts from Sonnet 4.x and Opus 4.1 to Opus 4.5.",
            &["Team plugins"],
            "C",
            0x9A9A9A,
            false,
            true,
        ),
        p(
            "frontend",
            "frontend-design",
            "Create distinctive, production-grade frontend interfaces.",
            &["Team plugins", "Design"],
            "F",
            0x9A9A9A,
            false,
            true,
        ),
    ];
    for id in [
        "cloudflare",
        "plausible",
        "github",
        "linear",
        "slack",
        "notion",
    ] {
        if let Some(plugin) = list.iter_mut().find(|p| p.id == id) {
            plugin.installed = true;
        }
    }
    list
}

pub fn default_skills() -> Vec<Skill> {
    vec![
        Skill {
            name: "daily-site-review".into(),
            description: "Check posts, RSS, deploys and traffic; write a dated report.".into(),
        },
        Skill {
            name: "summarize-inbox".into(),
            description: "Group unread email into Act, Read and Ignore.".into(),
        },
        Skill {
            name: "weekly-plan".into(),
            description: "Turn this week's calendar and tasks into a one-page plan.".into(),
        },
        Skill {
            name: "research-brief".into(),
            description: "Research a topic and write a sourced one-page brief.".into(),
        },
    ]
}

/// The team shown with `--demo`, built from the reference screenshots.
pub fn demo_bots(next_id: &mut u64) -> Vec<Bot> {
    let mut id = || {
        *next_id += 1;
        *next_id
    };
    let msg = |id: u64, kind: MessageKind, time: &str| Message {
        id,
        kind,
        time: time.into(),
        reactions: Vec::new(),
        reply_to: None,
        fresh: false,
    };

    let mut pulse = Bot::new(id(), "Blog Pulse", 7, Shape::Pill);
    pulse.title = "Daily site reviewer".into();
    pulse.description = "Reviews flaviocopes.com every morning: new posts, RSS, deploys and \
                         traffic. Separates verified facts from interpretation and links every \
                         claim to its source."
        .into();
    pulse.pinned = true;
    pulse.skills = vec![default_skills().remove(0)];
    let routine_id = id();
    pulse.routines.push(Routine {
        id: routine_id,
        name: "Daily blog review".into(),
        instruction: "Run the daily flaviocopes.com review as Blog Pulse. Today is the user's \
                      local date in Europe/Rome.\n\nDo the full review, then send it in chat and \
                      save it to /workspace/blog-pulse/YYYY-MM-DD.md."
            .into(),
        schedules: vec!["Every day at 10:01 AM".into()],
        active: true,
        history: Vec::new(),
    });
    pulse.messages = vec![
        msg(id(), MessageKind::Stamp("Today 10:01 AM".into()), ""),
        msg(id(), MessageKind::Event { text: "Renamed to Blog Pulse".into(), icon: None, link: None, routine: None }, ""),
        msg(id(), MessageKind::User { text: "You are Blog Pulse, the daily reviewer for https://flaviocopes.com.\nYour job is to prepare a daily review of the site.\nUse the Plausible analytics dashboard, the Cloudflare Pages deployments page, the GitHub repository, and the live site.\nFor every report:\n1. List posts published since the last report.\n2. Check the RSS feed includes them.\n3. Compare traffic with the previous day.\n4. Check that the latest deployment finished without errors.\n5. Separate verified facts from your interpretation.\n6. Link every claim to its source.\n7. Save the report as /workspace/blog-pulse/YYYY-MM-DD.md.\nEnd with:\n- what changed\n- what may require attention\n- what you checked\n- what you could not check\n- three suggested next actions\nYou may read connected systems and prepare drafts.\nDo not publish posts, push to the repository, send emails, close issues, or make purchases without approval.\nIf a source is unavailable, do not guess. Record the missing source and continue with the remaining checks.".into(), attachments: vec![] }, "10:01 AM"),
        msg(id(), MessageKind::Bot { text: "On it. I'll start today's review of flaviocopes.com and check which sources I can actually reach first.".into(), streaming: false }, "10:02 AM"),
        msg(id(), MessageKind::Activity { label: "Opened flaviocopes.com, /rss.xml and the Cloudflare dashboard".into(), done: true }, ""),
        msg(id(), MessageKind::Bot { text: "`/index.xml` is a 404. The live feed is `/rss.xml`.".into(), streaming: false }, "10:05 AM"),
        msg(id(), MessageKind::Bot { text: "I could not get traffic or the Pages deploy list. Plausible at https://b.flaviocopes.com is a login wall. The Cloudflare Pages sign-in was dismissed. The private GitHub repo is also closed without `gh` or the GitHub plugin.\n\nCloudflare Builds did connect. Worker `flaviocopes-daily-redeploy` is the scheduled rebuild. It POSTs the deploy hook at 07:15, 09:15, 11:15, and 15:15 Europe/Rome. It has no Workers Builds history, which is expected for a cron Worker. `flaviocopes.pages.dev` is serving the same two posts. That is not the same as seeing a green Pages deployment.".into(), streaming: false }, "10:06 AM"),
        msg(id(), MessageKind::Bot { text: "What changed: two new posts, RSS path confirmed, rebuild Worker identified.\n\nWhat may need attention: no visitor comparison, no proof the 07:15 or 09:15 rebuilds finished cleanly, and the 11:15 slot has not fired yet.\n\nWhat I checked: blog index, RSS, both post URLs, Plausible homepage, Cloudflare Workers list/code, `flaviocopes.pages.dev`, public GitHub.\n\nWhat I could not check: Plausible stats, top pages and sources, Pages deployments, the private repo.\n\nNext: sign in to Plausible, sign in to the Pages dashboard, and connect GitHub. I will not guess those numbers.".into(), streaming: false }, "10:07 AM"),
        msg(id(), MessageKind::File { name: "2026-08-22.md".into(), size: "4.6 KB".into(), content: REPORT.into() }, ""),
        msg(id(), MessageKind::Event { text: "Created routine".into(), icon: Some("icons/clock.svg".into()), link: Some("Daily blog review".into()), routine: Some(routine_id) }, ""),
        msg(id(), MessageKind::Bot { text: "I also set a daily review at 10:01 Europe/Rome, all 7 days, since posts go out on weekends too.".into(), streaming: false }, "10:07 AM"),
    ];

    let mut scout = Bot::new(id(), "Scout", 4, Shape::Hexagon);
    scout.title = "Learning scout".into();
    scout.description = "Finds DataCamp courses that fit my weekly goal and time budget, and \
                         reports durations as ranges."
        .into();
    scout.messages = vec![
        msg(id(), MessageKind::User { text: "Plan this week's learning: 4 hours, intermediate, focus on AI agents.".into(), attachments: vec![] }, "8:00 AM"),
        msg(id(), MessageKind::Activity { label: "Browsed datacamp.com/courses/introduction-to-ai-agents".into(), done: true }, ""),
        msg(id(), MessageKind::Snapshot { title: "Introduction to AI Agents".into(), url: "datacamp.com/courses/introduction-to-ai-agents".into() }, ""),
        msg(id(), MessageKind::Bot { text: "Monday DataCamp Learning Plan is on. It runs Mondays at 8:00 AM and uses the saved Skill. I won't fire it now.".into(), streaming: false }, "8:04 AM"),
    ];
    let scout_routine = id();
    scout.routines.push(Routine {
        id: scout_routine,
        name: "Monday DataCamp Learning Plan".into(),
        instruction: "Read the current learner goal, level, weekly time budget, format preference, and completed-resource list.\n\nRun the \"Weekly DataCamp Learning Plan\" Skill using those inputs.".into(),
        schedules: vec!["Every Monday at 8:00 AM".into()],
        active: true,
        history: vec![RunRecord { when: "Mon 29 Sep, 8:00 AM".into(), status: RunStatus::Succeeded }],
    });

    let mut inbox = Bot::new(id(), "Inbox Triage", 8, Shape::Cloud);
    inbox.title = "Email assistant".into();
    inbox.description = SUGGESTIONS[1].description.into();
    inbox.messages = vec![
        msg(id(), MessageKind::Bot { text: "Morning. 14 new emails: 3 need you, 6 to read, 5 I archived. Drafts are ready for the three.".into(), streaming: false }, "7:30 AM"),
        msg(id(), MessageKind::Approval { action: "Send 3 drafted replies".into(), detail: "To: Marta (invoice), Jon (Thursday call), Priya (contract v2). Each draft is in your Gmail drafts folder.".into(), state: ApprovalState::Pending }, "7:31 AM"),
    ];
    inbox.status = BotStatus::NeedsYou;
    inbox.unread = true;

    let mut night = Bot::new(id(), "Night Shift", 2, Shape::Hexagon);
    night.title = SUGGESTIONS[0].title.into();
    night.description = SUGGESTIONS[0].description.into();
    night.messages = vec![msg(
        id(),
        MessageKind::Bot {
            text: "Your morning digest is ready: 2 deploys, 1 failed check, 9 new issues.".into(),
            streaming: false,
        },
        "6:00 AM",
    )];

    vec![pulse, inbox, scout, night]
}

pub const REPORT: &str = "# Blog Pulse, 23 Aug 2026\n\nChecked at about 10:07 Europe/Rome. Sources I could open are linked. Locked sources are left blank, not guessed.\n\n## Facts\n\n### Today's posts went live\n\nTwo posts dated 23 Aug 2026 are on the blog index and in RSS. Both pages returned HTTP 200.\n\n1. Herdr vs tmux vs Zellij. RSS `pubDate`: Sun, 23 Aug 2026 06:00:00 GMT, 08:00 Europe/Rome\n2. Scroll-driven CSS animations. RSS `pubDate`: Sun, 23 Aug 2026 05:00:00 GMT, 07:00 Europe/Rome\n\n### Posts that went live after yesterday's 10:01 review\n\nYesterday's morning report only saw the 07:00 and 08:00 Europe/Rome posts. Three more dated 22 Aug are now live, all HTTP 200:\n\n- A deep dive into Grok Bot, Sat, 22 Aug 2026 08:00:00 GMT\n- How to group array items with Object.groupBy(), Sat, 22 Aug 2026 09:00:00 GMT\n- Swamp tutorial: make AI agent work repeatable, Sat, 22 Aug 2026 13:00:00 GMT\n\nThat matches the 09:15, 11:15, and 15:15 Europe/Rome rebuild slots from the scheduled Worker.\n\n## Traffic\n\nNot checked. https://b.flaviocopes.com is still a Plausible login page. No numbers are guessed.\n\n## Next actions\n\n1. Sign in to Plausible in the Agent Computer.\n2. Sign in to the Cloudflare Pages dashboard.\n3. Add the GitHub plugin so I can read the private repo.\n";
