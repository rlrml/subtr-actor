use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::Parser;
use subtr_actor::analysis_graph::all_analysis_nodes;
use subtr_actor::{
    ALL_GOAL_TAG_DEFINITIONS, DetectionConfidence, EmittedEvent, EventDefinition,
    GoalTagDefinition, KnownIssueRef, ProducerDefinition,
};

#[derive(Debug, Parser)]
#[command(about = "Generate Markdown documentation from static event definitions.")]
struct Args {
    /// Write generated Markdown to this path. Prints to stdout when omitted.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Verify that --output already matches the generated Markdown.
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
    let markdown = render_docs(&emitted_events, ALL_GOAL_TAG_DEFINITIONS)?;

    match (args.output, args.check) {
        (Some(output), true) => {
            let existing = std::fs::read_to_string(&output)
                .with_context(|| format!("failed to read {}", output.display()))?;
            if existing != markdown {
                bail!("{} is out of date", output.display());
            }
        }
        (Some(output), false) => {
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            std::fs::write(&output, markdown)
                .with_context(|| format!("failed to write {}", output.display()))?;
        }
        (None, true) => {
            bail!("--check requires --output");
        }
        (None, false) => {
            print!("{markdown}");
        }
    }

    Ok(())
}

fn render_docs(
    emitted_events: &[EmittedEvent],
    goal_tags: &[GoalTagDefinition],
) -> anyhow::Result<String> {
    let mut markdown = String::new();
    markdown.push_str("# Stat Definitions\n\n");
    markdown.push_str("Generated from static Rust metadata. Do not edit by hand.\n\n");
    render_event_docs(&mut markdown, emitted_events)?;
    render_goal_tag_docs(&mut markdown, goal_tags);
    Ok(markdown)
}

fn render_event_docs(markdown: &mut String, emitted_events: &[EmittedEvent]) -> anyhow::Result<()> {
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

    markdown.push_str("## Events\n\n");

    for event in events.values_mut() {
        event.producers.sort_by_key(|producer| {
            (
                producer.node_name,
                producer.node_type,
                producer.calculator_type,
            )
        });
        event.producers.dedup();

        render_event(markdown, event)?;
    }

    Ok(())
}

fn render_event(markdown: &mut String, event: &EventDoc) -> anyhow::Result<()> {
    let definition = event.definition;
    markdown.push_str("### ");
    markdown.push_str(definition.label);
    markdown.push_str(" (`");
    markdown.push_str(definition.id);
    markdown.push_str("`)\n\n");

    markdown.push_str("- Category: `");
    markdown.push_str(&serialized_token(&definition.category)?);
    markdown.push_str("`\n");
    render_confidence(markdown, &definition.confidence)?;

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

    render_text_section(markdown, "Summary", definition.summary);
    render_list_section(markdown, "Approach", definition.approach);
    render_list_section(markdown, "Limitations", definition.limitations);
    render_known_issues(markdown, definition.confidence.known_issues);
    markdown.push('\n');
    Ok(())
}

fn render_goal_tag_docs(markdown: &mut String, goal_tags: &[GoalTagDefinition]) {
    markdown.push_str("## Goal Tags\n\n");
    for definition in goal_tags {
        markdown.push_str("### ");
        markdown.push_str(definition.label);
        markdown.push_str(" (`");
        markdown.push_str(definition.id);
        markdown.push_str("`)\n\n");

        render_text_section(markdown, "Summary", definition.summary);
        render_list_section(markdown, "Approach", definition.approach);
        markdown.push('\n');
    }
}

fn render_confidence(
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

fn render_text_section(markdown: &mut String, heading: &str, value: &str) {
    markdown.push('\n');
    markdown.push_str("**");
    markdown.push_str(heading);
    markdown.push_str("**\n\n");
    markdown.push_str(value);
    markdown.push('\n');
}

fn render_list_section(markdown: &mut String, heading: &str, values: &[&str]) {
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

fn render_known_issues(markdown: &mut String, issues: &[KnownIssueRef]) {
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

fn serialized_token(value: &impl serde::Serialize) -> anyhow::Result<String> {
    let serialized = serde_json::to_string(value)?;
    Ok(serialized.trim_matches('"').to_owned())
}

#[allow(dead_code)]
fn _assert_event_is_copy(_event: EmittedEvent) {}
