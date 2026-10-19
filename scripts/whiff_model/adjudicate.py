"""Combine independent judgments into qualified labels with explicit tiers.

Two independent first-pass judgments (different model families, same blinded
evidence) are matched interaction by interaction. Agreement is accepted;
disagreements and telemetry contradictions go to at most two re-reviews that
see close-up evidence and the anonymized competing answers. A re-review that
sides with one first-pass answer resolves the item; otherwise it stays
`uncertain`. Agreement between models is not ground truth, so every final
label records its tier and the attempts behind it.
"""

from __future__ import annotations

import numpy as np

from telemetry import ReplayTelemetry

MATCH_TOLERANCE_SECONDS = 0.4
ABSENT = "no_observable_attempt"


def match_interactions(a: list[dict], b: list[dict], tolerance: float = MATCH_TOLERANCE_SECONDS):
    """Greedy one-to-one time matching; unmatched items pair with None."""
    pairs = []
    used_b = set()
    order = sorted(range(len(a)), key=lambda i: a[i]["closest_approach_t"])
    for i in order:
        best, best_dt = None, tolerance
        for j, item in enumerate(b):
            if j in used_b:
                continue
            delta = abs(item["closest_approach_t"] - a[i]["closest_approach_t"])
            if delta <= best_dt:
                best, best_dt = j, delta
        if best is not None:
            used_b.add(best)
        pairs.append((a[i], b[best] if best is not None else None))
    pairs.extend((None, b[j]) for j in range(len(b)) if j not in used_b)
    return pairs


def label_of(item: dict | None) -> str:
    return ABSENT if item is None else item["label"]


def telemetry_flags(r: ReplayTelemetry, window: dict, label: str, t_window: float) -> list[str]:
    """Contradictions between a label and replay telemetry (which can itself be wrong)."""
    track = r.players[window["player_index"]]
    t = window["window_start_time"] + t_window
    lo, hi = r.index_at_time(t - 0.3), r.index_at_time(t + 0.3)
    segment = track.contact_gap[lo : hi + 1]
    min_gap = float(np.nanmin(segment)) if np.isfinite(segment).any() else np.inf
    focal_touch = any(
        touch.get("player_id") == track.player_id and lo <= r.touch_index(touch) <= hi
        for touch in r.touches
    )
    flags = []
    if label == "successful_contact" and not focal_touch and min_gap > 50:
        flags.append("contact_label_without_telemetry_contact")
    if label in ("clear_whiff", "disrupted_attempt") and focal_touch and min_gap <= 5:
        flags.append("miss_label_with_detected_focal_touch")
    if label == "clear_whiff" and min_gap > 700:
        flags.append("whiff_label_far_from_ball")
    return flags


def summarize(item: dict | None) -> str:
    if item is None:
        return "reported no interaction at this time (equivalent to no_observable_attempt)"
    return (
        f"label={item['label']}, attempt={item['attempt_observable']}, contact={item['focal_contact']}, "
        f"disrupted={item['disrupted_by_other_player']}, possible_fake={item['possible_fake']}, "
        f"closest_approach_t={item['closest_approach_t']:.2f}; rationale: {item.get('rationale', '')}"
    )


def review_context(t: float, first: dict | None, second: dict | None) -> str:
    return (
        f"\nTwo earlier reviewers disagreed about (or a telemetry check questioned) the interaction "
        f"near t={t:.2f}s. Their answers, in random order:\n"
        f"  Reviewer A: {summarize(first)}\n  Reviewer B: {summarize(second)}\n"
        "Either may be wrong. Re-examine all images, especially the close-up sheets, and give your own "
        "judgment of every interaction in the core interval using the same JSON format.\n"
    )


def resolve(first_label: str, second_label: str, reviews: list[str]) -> tuple[str, str]:
    """Final (label, tier) from the two first-pass labels and up to two review labels."""
    if first_label == second_label:
        return first_label, "agreed"
    for review in reviews:
        if review in (first_label, second_label):
            return review, "adjudicated"
    return "uncertain", "unresolved"
