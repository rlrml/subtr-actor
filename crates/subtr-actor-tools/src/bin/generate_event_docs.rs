use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::{Parser, ValueEnum};
use subtr_actor::analysis_graph::all_analysis_nodes;
use subtr_actor::{
    ALL_GOAL_TAG_DEFINITIONS, DetectionConfidence, EmittedEvent, EventDefinition,
    GoalTagDefinition, KnownIssueRef, ProducerDefinition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    /// Diff-friendly Markdown reference (the default).
    Markdown,
    /// Self-contained static HTML page with a synopsis table of contents,
    /// live search, and per-event detail sections (suitable for GitHub Pages).
    Html,
}

#[derive(Debug, Parser)]
#[command(about = "Generate event/stat documentation from static event definitions.")]
struct Args {
    /// Output format to render.
    #[arg(long, value_enum, default_value_t = Format::Markdown)]
    format: Format,

    /// Write generated output to this path. Prints to stdout when omitted.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Verify that --output already matches the generated output.
    #[arg(long)]
    check: bool,
}

#[derive(Debug)]
struct EventDoc {
    definition: &'static EventDefinition,
    producers: Vec<ProducerDefinition>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Walk the actual analysis nodes so docs are sourced from the same place
    // the runtime is: each node reports the events it emits. There is no
    // separate name-keyed producer registry to keep in sync.
    let nodes = all_analysis_nodes();
    let emitted_events: Vec<EmittedEvent> = nodes
        .iter()
        .flat_map(|node| node.emitted_events().iter().copied())
        .collect();
    let events = collect_events(&emitted_events)?;
    let rendered = match args.format {
        Format::Markdown => render_markdown(&events, ALL_GOAL_TAG_DEFINITIONS)?,
        Format::Html => render_html(&events, ALL_GOAL_TAG_DEFINITIONS)?,
    };

    match (args.output, args.check) {
        (Some(output), true) => {
            let existing = std::fs::read_to_string(&output)
                .with_context(|| format!("failed to read {}", output.display()))?;
            if existing != rendered {
                bail!("{} is out of date", output.display());
            }
        }
        (Some(output), false) => {
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            std::fs::write(&output, rendered)
                .with_context(|| format!("failed to write {}", output.display()))?;
        }
        (None, true) => {
            bail!("--check requires --output");
        }
        (None, false) => {
            print!("{rendered}");
        }
    }

    Ok(())
}

/// Collect emitted events into a stable, id-keyed map with de-duplicated,
/// sorted producers. Shared by every output format so they can never drift.
fn collect_events(
    emitted_events: &[EmittedEvent],
) -> anyhow::Result<BTreeMap<&'static str, EventDoc>> {
    let mut events = BTreeMap::<&'static str, EventDoc>::new();

    for emitted in emitted_events {
        let entry = events.entry(emitted.event.id).or_insert_with(|| EventDoc {
            definition: emitted.event,
            producers: Vec::new(),
        });
        if entry.definition != emitted.event {
            bail!("conflicting event definitions for {}", emitted.event.id);
        }
        entry.producers.push(emitted.producer);
    }

    for event in events.values_mut() {
        event.producers.sort_by_key(|producer| {
            (
                producer.node_name,
                producer.node_type,
                producer.calculator_type,
            )
        });
        event.producers.dedup();
    }

    Ok(events)
}

// ---------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------

fn render_markdown(
    events: &BTreeMap<&'static str, EventDoc>,
    goal_tags: &[GoalTagDefinition],
) -> anyhow::Result<String> {
    let mut markdown = String::new();
    markdown.push_str("# Stat Definitions\n\n");
    markdown.push_str("Generated from static Rust metadata. Do not edit by hand.\n\n");

    markdown.push_str("## Events\n\n");
    for event in events.values() {
        render_event_markdown(&mut markdown, event)?;
    }

    render_goal_tag_markdown(&mut markdown, goal_tags);
    Ok(markdown)
}

fn render_event_markdown(markdown: &mut String, event: &EventDoc) -> anyhow::Result<()> {
    let definition = event.definition;
    markdown.push_str("### ");
    markdown.push_str(definition.label);
    markdown.push_str(" (`");
    markdown.push_str(definition.id);
    markdown.push_str("`)\n\n");

    markdown.push_str("- Category: `");
    markdown.push_str(&serialized_token(&definition.category)?);
    markdown.push_str("`\n");
    render_confidence_markdown(markdown, &definition.confidence)?;

    markdown.push_str("- Producers:");
    if event.producers.is_empty() {
        markdown.push_str(" none\n");
    } else {
        markdown.push('\n');
        for producer in &event.producers {
            markdown.push_str("  - `");
            markdown.push_str(producer.node_name);
            markdown.push_str("` via `");
            markdown.push_str(producer.node_type);
            markdown.push_str("` / `");
            markdown.push_str(producer.calculator_type);
            markdown.push_str("`\n");
        }
    }

    render_text_section_markdown(markdown, "Summary", definition.summary);
    render_list_section_markdown(markdown, "Approach", definition.approach);
    render_list_section_markdown(markdown, "Limitations", definition.limitations);
    render_known_issues_markdown(markdown, definition.confidence.known_issues);
    markdown.push('\n');
    Ok(())
}

fn render_goal_tag_markdown(markdown: &mut String, goal_tags: &[GoalTagDefinition]) {
    markdown.push_str("## Goal Tags\n\n");
    for definition in goal_tags {
        markdown.push_str("### ");
        markdown.push_str(definition.label);
        markdown.push_str(" (`");
        markdown.push_str(definition.id);
        markdown.push_str("`)\n\n");

        render_text_section_markdown(markdown, "Summary", definition.summary);
        render_list_section_markdown(markdown, "Approach", definition.approach);
        markdown.push('\n');
    }
}

fn render_confidence_markdown(
    markdown: &mut String,
    confidence: &DetectionConfidence,
) -> anyhow::Result<()> {
    markdown.push_str("- Confidence:\n");
    markdown.push_str("  - Approach: `");
    markdown.push_str(&serialized_token(&confidence.approach)?);
    markdown.push_str("`\n");
    markdown.push_str("  - True positive evidence: `");
    markdown.push_str(&serialized_token(&confidence.true_positive_evidence)?);
    markdown.push_str("`\n");
    markdown.push_str("  - False positive evidence: `");
    markdown.push_str(&serialized_token(&confidence.false_positive_evidence)?);
    markdown.push_str("`\n");
    markdown.push_str("  - False negative evidence: `");
    markdown.push_str(&serialized_token(&confidence.false_negative_evidence)?);
    markdown.push_str("`\n");
    markdown.push_str("  - Testing: `");
    markdown.push_str(&serialized_token(&confidence.testing)?);
    markdown.push_str("`\n");
    Ok(())
}

fn render_text_section_markdown(markdown: &mut String, heading: &str, value: &str) {
    markdown.push('\n');
    markdown.push_str("**");
    markdown.push_str(heading);
    markdown.push_str("**\n\n");
    markdown.push_str(value);
    markdown.push('\n');
}

fn render_list_section_markdown(markdown: &mut String, heading: &str, values: &[&str]) {
    markdown.push('\n');
    markdown.push_str("**");
    markdown.push_str(heading);
    markdown.push_str("**\n\n");
    if values.is_empty() {
        markdown.push_str("_None documented._\n");
    } else {
        for value in values {
            markdown.push_str("- ");
            markdown.push_str(value);
            markdown.push('\n');
        }
    }
}

fn render_known_issues_markdown(markdown: &mut String, issues: &[KnownIssueRef]) {
    markdown.push_str("\n**Known Issues**\n\n");
    if issues.is_empty() {
        markdown.push_str("_None documented._\n");
    } else {
        for issue in issues {
            markdown.push_str("- `");
            markdown.push_str(issue.id);
            markdown.push_str("`: ");
            markdown.push_str(issue.summary);
            if let Some(url) = issue.url {
                markdown.push_str(" (");
                markdown.push_str(url);
                markdown.push(')');
            }
            markdown.push('\n');
        }
    }
}

// ---------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------

const HTML_STYLE: &str = r#"
:root {
  --bg: #ffffff;
  --fg: #1b1f24;
  --muted: #5b6573;
  --border: #d7dce2;
  --card: #f6f8fa;
  --accent: #2f6feb;
  --code-bg: #eef1f4;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0d1117;
    --fg: #e6edf3;
    --muted: #9aa6b2;
    --border: #2a313a;
    --card: #161b22;
    --accent: #6ea8ff;
    --code-bg: #1f262e;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  font: 15px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--fg);
  background: var(--bg);
}
.wrap { max-width: 980px; margin: 0 auto; padding: 2rem 1.25rem 5rem; }
h1 { font-size: 1.9rem; margin: 0 0 .25rem; }
h2 { font-size: 1.35rem; margin: 2.5rem 0 .75rem; border-bottom: 1px solid var(--border); padding-bottom: .35rem; }
h3 { font-size: 1.1rem; margin: 0; }
.subtitle { color: var(--muted); margin: 0 0 1.5rem; }
code, .mono { font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace; font-size: .85em; }
code { background: var(--code-bg); padding: .1em .35em; border-radius: 4px; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
.search {
  width: 100%;
  padding: .6rem .8rem;
  font-size: 1rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--fg);
  margin-bottom: 1.5rem;
}
table.toc { width: 100%; border-collapse: collapse; }
table.toc th, table.toc td { text-align: left; padding: .5rem .6rem; border-bottom: 1px solid var(--border); vertical-align: top; }
table.toc th { color: var(--muted); font-weight: 600; font-size: .8rem; text-transform: uppercase; letter-spacing: .03em; }
table.toc td.summary { color: var(--muted); }
.cat-group { margin-top: 1rem; }
.cat-group > .cat-head { font-size: .8rem; text-transform: uppercase; letter-spacing: .03em; color: var(--muted); margin: 1.25rem 0 .35rem; font-weight: 700; }
.badge {
  display: inline-block;
  font-size: .72rem;
  font-weight: 600;
  padding: .12em .55em;
  border-radius: 999px;
  background: var(--card);
  border: 1px solid var(--border);
  color: var(--muted);
  white-space: nowrap;
}
.event {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 1rem 1.1rem;
  margin: .9rem 0;
  background: var(--card);
  scroll-margin-top: 1rem;
}
.event .head { display: flex; align-items: baseline; gap: .5rem; flex-wrap: wrap; }
.event .head .id { color: var(--muted); }
.event .summary { margin: .6rem 0 .25rem; }
.section-label { font-size: .78rem; text-transform: uppercase; letter-spacing: .03em; color: var(--muted); font-weight: 700; margin: .85rem 0 .3rem; }
ul.tight { margin: .2rem 0; padding-left: 1.2rem; }
ul.tight li { margin: .15rem 0; }
.conf { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .3rem; }
.conf .badge b { color: var(--fg); font-weight: 700; }
.producers code { font-size: .8em; }
.muted { color: var(--muted); }
.toplink { float: right; font-size: .8rem; }
.count { color: var(--muted); font-weight: 400; font-size: .9rem; }
"#;

const HTML_SCRIPT: &str = r#"
const search = document.getElementById('search');
const rows = Array.from(document.querySelectorAll('[data-search]'));
const groups = Array.from(document.querySelectorAll('[data-group]'));
function applyFilter() {
  const q = search.value.trim().toLowerCase();
  for (const el of rows) {
    const hit = !q || el.getAttribute('data-search').includes(q);
    el.style.display = hit ? '' : 'none';
  }
  for (const g of groups) {
    const anyVisible = Array.from(g.querySelectorAll('[data-search]'))
      .some(el => el.style.display !== 'none');
    g.style.display = anyVisible ? '' : 'none';
  }
}
search.addEventListener('input', applyFilter);
"#;

fn render_html(
    events: &BTreeMap<&'static str, EventDoc>,
    goal_tags: &[GoalTagDefinition],
) -> anyhow::Result<String> {
    // Group events by category for the table of contents, preserving the
    // id-sorted order within each category.
    let mut by_category: BTreeMap<String, Vec<&EventDoc>> = BTreeMap::new();
    for event in events.values() {
        let category = serialized_token(&event.definition.category)?;
        by_category.entry(category).or_default().push(event);
    }

    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>subtr-actor — Stat &amp; Event Definitions</title>\n");
    html.push_str("<style>");
    html.push_str(HTML_STYLE);
    html.push_str("</style>\n</head>\n<body>\n<div class=\"wrap\">\n");

    html.push_str("<h1>Stat &amp; Event Definitions</h1>\n");
    writeln!(
        html,
        "<p class=\"subtitle\">Generated from static Rust metadata — do not edit by hand. \
         <span class=\"count\">{} events</span></p>",
        events.len()
    )?;

    html.push_str(
        "<input id=\"search\" class=\"search\" type=\"search\" placeholder=\"Filter events by name, id, category, or summary…\" autocomplete=\"off\">\n",
    );

    // Table of contents: one synopsis row per event, grouped by category.
    html.push_str("<h2 id=\"contents\">Contents</h2>\n");
    for (category, group) in &by_category {
        writeln!(
            html,
            "<div class=\"cat-group\" data-group><div class=\"cat-head\">{}</div>",
            escape_html(&title_case(category))
        )?;
        html.push_str("<table class=\"toc\"><thead><tr><th>Event</th><th>ID</th><th>Summary</th></tr></thead><tbody>\n");
        for event in group {
            let definition = event.definition;
            let search_blob = event_search_blob(definition, category);
            writeln!(
                html,
                "<tr data-search=\"{}\"><td><a href=\"#event-{}\">{}</a></td>\
                 <td><code>{}</code></td><td class=\"summary\">{}</td></tr>",
                escape_html(&search_blob),
                escape_html(definition.id),
                escape_html(definition.label),
                escape_html(definition.id),
                escape_html(synopsis(definition.summary)),
            )?;
        }
        html.push_str("</tbody></table></div>\n");
    }

    // Full per-event detail sections, grouped by category.
    html.push_str("<h2 id=\"events\">Events</h2>\n");
    for (category, group) in &by_category {
        writeln!(
            html,
            "<div class=\"cat-group\" data-group><div class=\"cat-head\">{}</div>",
            escape_html(&title_case(category))
        )?;
        for event in group {
            render_event_html(&mut html, event, category)?;
        }
        html.push_str("</div>\n");
    }

    // Goal tags.
    html.push_str("<h2 id=\"goal-tags\">Goal Tags</h2>\n");
    for definition in goal_tags {
        let search_blob = format!(
            "{} {} {}",
            definition.label, definition.id, definition.summary
        )
        .to_lowercase();
        writeln!(
            html,
            "<div class=\"event\" id=\"goal-{}\" data-search=\"{}\">\n\
             <div class=\"head\"><h3>{}</h3><code class=\"id\">{}</code>\
             <a class=\"toplink\" href=\"#contents\">top ↑</a></div>\n\
             <p class=\"summary\">{}</p>",
            escape_html(definition.id),
            escape_html(&search_blob),
            escape_html(definition.label),
            escape_html(definition.id),
            escape_html(definition.summary),
        )?;
        render_list_html(&mut html, "Approach", definition.approach);
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n<script>");
    html.push_str(HTML_SCRIPT);
    html.push_str("</script>\n</body>\n</html>\n");
    Ok(html)
}

fn render_event_html(html: &mut String, event: &EventDoc, category: &str) -> anyhow::Result<()> {
    let definition = event.definition;
    let search_blob = event_search_blob(definition, category);

    write!(
        html,
        "<div class=\"event\" id=\"event-{}\" data-search=\"{}\">\n\
         <div class=\"head\"><h3>{}</h3><code class=\"id\">{}</code>\
         <span class=\"badge\">{}</span>",
        escape_html(definition.id),
        escape_html(&search_blob),
        escape_html(definition.label),
        escape_html(definition.id),
        escape_html(&title_case(category)),
    )?;
    if definition.hidden_from_review {
        html.push_str("<span class=\"badge\">hidden from review</span>");
    }
    html.push_str("<a class=\"toplink\" href=\"#contents\">top ↑</a></div>\n");

    writeln!(
        html,
        "<p class=\"summary\">{}</p>",
        escape_html(definition.summary)
    )?;

    render_confidence_html(html, &definition.confidence)?;
    render_list_html(html, "Approach", definition.approach);
    render_list_html(html, "Limitations", definition.limitations);
    render_known_issues_html(html, definition.confidence.known_issues);
    render_producers_html(html, &event.producers);

    html.push_str("</div>\n");
    Ok(())
}

fn render_confidence_html(
    html: &mut String,
    confidence: &DetectionConfidence,
) -> anyhow::Result<()> {
    html.push_str("<div class=\"section-label\">Confidence</div>\n<div class=\"conf\">\n");
    let entries = [
        ("Approach", serialized_token(&confidence.approach)?),
        (
            "True positive",
            serialized_token(&confidence.true_positive_evidence)?,
        ),
        (
            "False positive",
            serialized_token(&confidence.false_positive_evidence)?,
        ),
        (
            "False negative",
            serialized_token(&confidence.false_negative_evidence)?,
        ),
        ("Testing", serialized_token(&confidence.testing)?),
    ];
    for (label, value) in entries {
        writeln!(
            html,
            "<span class=\"badge\">{} <b>{}</b></span>",
            escape_html(label),
            escape_html(&title_case(&value)),
        )?;
    }
    html.push_str("</div>\n");
    Ok(())
}

fn render_list_html(html: &mut String, heading: &str, values: &[&str]) {
    let _ = writeln!(html, "<div class=\"section-label\">{heading}</div>");
    if values.is_empty() {
        html.push_str("<p class=\"muted\">None documented.</p>\n");
    } else {
        html.push_str("<ul class=\"tight\">\n");
        for value in values {
            let _ = writeln!(html, "<li>{}</li>", escape_html(value));
        }
        html.push_str("</ul>\n");
    }
}

fn render_known_issues_html(html: &mut String, issues: &[KnownIssueRef]) {
    html.push_str("<div class=\"section-label\">Known issues</div>\n");
    if issues.is_empty() {
        html.push_str("<p class=\"muted\">None documented.</p>\n");
        return;
    }
    html.push_str("<ul class=\"tight\">\n");
    for issue in issues {
        html.push_str("<li><code>");
        html.push_str(&escape_html(issue.id));
        html.push_str("</code>: ");
        html.push_str(&escape_html(issue.summary));
        if let Some(url) = issue.url {
            let _ = write!(
                html,
                " (<a href=\"{}\">{}</a>)",
                escape_html(url),
                escape_html(url)
            );
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n");
}

fn render_producers_html(html: &mut String, producers: &[ProducerDefinition]) {
    html.push_str("<div class=\"section-label\">Producers</div>\n");
    if producers.is_empty() {
        html.push_str("<p class=\"muted\">None.</p>\n");
        return;
    }
    html.push_str("<ul class=\"tight producers\">\n");
    for producer in producers {
        let _ = writeln!(
            html,
            "<li><code>{}</code> via <code>{}</code> / <code>{}</code></li>",
            escape_html(producer.node_name),
            escape_html(producer.node_type),
            escape_html(producer.calculator_type),
        );
    }
    html.push_str("</ul>\n");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn event_search_blob(definition: &EventDefinition, category: &str) -> String {
    format!(
        "{} {} {} {}",
        definition.label, definition.id, category, definition.summary
    )
    .to_lowercase()
}

/// First sentence (or the whole thing if short) for the synopsis column.
fn synopsis(summary: &str) -> &str {
    match summary.find(". ") {
        Some(idx) => &summary[..=idx],
        None => summary,
    }
}

/// Turn a snake_case serde token into Title Case for display.
fn title_case(token: &str) -> String {
    token
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn serialized_token(value: &impl serde::Serialize) -> anyhow::Result<String> {
    let serialized = serde_json::to_string(value)?;
    Ok(serialized.trim_matches('"').to_owned())
}

#[allow(dead_code)]
fn _assert_event_is_copy(_event: EmittedEvent) {}
