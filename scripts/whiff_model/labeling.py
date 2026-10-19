"""Blinded vision-model labeling with an append-only provenance ledger.

Each call sends one window's rendered images (identified only by a neutral ID)
plus the rubric to a vision-capable model and records the raw response, its
hash, the parsed judgment, validation errors, latency, and reported cost.
Attempts are keyed by content (images, prompt, transport, model, settings) so
interrupted runs resume without re-labeling and repeated reviews never
overwrite earlier provenance.
"""

from __future__ import annotations

import concurrent.futures
import datetime as dt
import hashlib
import json
import os
import pathlib
import re
import subprocess
import tempfile
import threading
import time

PROMPT_VERSION = "whiff-rubric-v1"
CLASSES = (
    "clear_whiff",
    "disrupted_attempt",
    "successful_contact",
    "no_observable_attempt",
    "uncertain",
)
TRI_STATE = ("yes", "no", "unclear")
CONFIDENCE = ("low", "medium", "high")

RUBRIC = """\
You are labeling a Rocket League replay window for a research dataset. Judge only what the
evidence shows. Every window is 6 seconds long with time t measured from the window start.
The CORE INTERVAL is highlighted (yellow); only interactions whose closest approach between the
FOCAL car and the ball falls inside the core interval should be reported.

Colors: the FOCAL car is magenta (marked 'F' in 3-D views); its teammate is blue; opponents are
orange; the ball is white/grey. In 3-D views the white face of each car box is its NOSE (front).
Cars are drawn as their collision hitbox, the ball at its true radius (92.75 uu). Frames are only
drawn at real replay samples; there is no interpolation.

For EACH interaction in the core interval where the focal car moves toward, jumps/dodges/aerials
at, or passes close to the ball, answer three separate questions:
  attempt_observable: did the focal player visibly TRY to contact the ball (committed approach,
      jump, dodge, aerial or turn into the ball's path such that contact was plausible)?
  focal_contact: did the focal car actually touch the ball?
  disrupted_by_other_player: before the focal car could reach the ball, did ANOTHER player (teammate
      or opponent) touch the ball, or bump/demo the focal car, in a way that removed the opportunity?
Then derive exactly one label:
  clear_whiff            attempt=yes, contact=no, disrupted=no, and not plausibly a fake.
  disrupted_attempt      attempt=yes, contact=no, disrupted=yes (another player's touch/bump
                         took the ball or the car away first). Keep these separate from whiffs.
  successful_contact     the focal car touched the ball (any quality; a bad shot is still contact).
  no_observable_attempt  proximity without visible intent: shadowing, rotating past, positioning,
                         a deliberate fake/dummy, or incidental closeness.
  uncertain              the evidence cannot separate the options (e.g. possible fake vs whiff,
                         contact not determinable, intent unclear). Do not guess.
A missed follow-up after a successful touch is a separate interaction. If nothing in the core
interval comes close to an attempt, return an empty "interactions" list. Mark the window unusable
only if the rendering/telemetry is too broken to judge.

Return ONLY a JSON object, no prose, with this shape:
{"window_usable": true|false,
 "unusable_reason": string|null,
 "interactions": [
   {"closest_approach_t": number,      // window seconds, inside the core interval
    "attempt_start_t": number|null,     // when the approach/commitment became visible
    "commit_t": number|null,            // jump/dodge/aerial commitment time, if any
    "attempt_observable": "yes"|"no"|"unclear",
    "focal_contact": "yes"|"no"|"unclear",
    "disrupted_by_other_player": "yes"|"no"|"unclear",
    "possible_fake": "yes"|"no"|"unclear",
    "label": "clear_whiff"|"disrupted_attempt"|"successful_contact"|"no_observable_attempt"|"uncertain",
    "evidence": [string],               // e.g. "ball_cam_core t=3.40", "trajectory top view"
    "rationale": string,                // one or two sentences of observable evidence
    "uncertainty_reason": string|null,
    "confidence": "low"|"medium"|"high"}
 ]}
"""

REPRESENTATION_NOTES = {
    "replay": "Images: pseudo-3-D replay contact sheets (ball-cam behind the focal car and a fixed "
    "overview camera at ~10 frames/s over the core interval, plus a 2 frames/s overview of the "
    "whole window).",
    "trajectory": "Image: timestamped trajectory plots (zoomed top view, side view, ball position in "
    "the focal car's own frame, and whole-field context). Numbers along paths are window times.",
    "compact": "Images: timestamped trajectory plots and a telemetry strip. In the telemetry strip "
    "positions/velocities are measured, the hitbox gap is an estimate, and touch markers come from a "
    "heuristic touch detector that can be wrong; weigh them against the trajectory evidence.",
    "combined": "Images: pseudo-3-D replay contact sheets, trajectory plots, and a telemetry strip. "
    "In the telemetry strip positions/velocities are measured, the hitbox gap is an estimate, and "
    "touch markers come from a heuristic touch detector that can be wrong; weigh them against the "
    "visual evidence.",
}

RESPONSE_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["window_usable", "unusable_reason", "interactions"],
    "properties": {
        "window_usable": {"type": "boolean"},
        "unusable_reason": {"type": ["string", "null"]},
        "interactions": {
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": [
                    "closest_approach_t",
                    "attempt_start_t",
                    "commit_t",
                    "attempt_observable",
                    "focal_contact",
                    "disrupted_by_other_player",
                    "possible_fake",
                    "label",
                    "evidence",
                    "rationale",
                    "uncertainty_reason",
                    "confidence",
                ],
                "properties": {
                    "closest_approach_t": {"type": "number"},
                    "attempt_start_t": {"type": ["number", "null"]},
                    "commit_t": {"type": ["number", "null"]},
                    "attempt_observable": {"type": "string", "enum": list(TRI_STATE)},
                    "focal_contact": {"type": "string", "enum": list(TRI_STATE)},
                    "disrupted_by_other_player": {"type": "string", "enum": list(TRI_STATE)},
                    "possible_fake": {"type": "string", "enum": list(TRI_STATE)},
                    "label": {"type": "string", "enum": list(CLASSES)},
                    "evidence": {"type": "array", "items": {"type": "string"}},
                    "rationale": {"type": "string"},
                    "uncertainty_reason": {"type": ["string", "null"]},
                    "confidence": {"type": "string", "enum": list(CONFIDENCE)},
                },
            },
        },
    },
}


def build_prompt(
    representation: str, image_paths: list[pathlib.Path], core: list[float], extra: str = ""
) -> str:
    listing = "\n".join(f"- {path}" for path in image_paths)
    return (
        f"{RUBRIC}\n{REPRESENTATION_NOTES[representation]}\n"
        f"Core interval for this window: t={core[0]:.2f}s to t={core[1]:.2f}s.\n"
        f"{extra}"
        f"Read every image file below before answering:\n{listing}\n"
    )


def extract_json(text: str) -> dict:
    text = text.strip()
    fenced = re.search(r"```(?:json)?\s*(\{.*\})\s*```", text, re.S)
    if fenced:
        text = fenced.group(1)
    start = text.find("{")
    end = text.rfind("}")
    if start < 0 or end <= start:
        raise ValueError("no JSON object in response")
    return json.loads(text[start : end + 1])


def validate_judgment(judgment: dict, core: list[float], tolerance: float = 0.25) -> list[str]:
    """Schema plus rubric-consistency errors; an empty list means valid."""
    errors = []
    if not isinstance(judgment.get("window_usable"), bool):
        errors.append("window_usable must be boolean")
    interactions = judgment.get("interactions")
    if not isinstance(interactions, list):
        return errors + ["interactions must be a list"]
    for n, item in enumerate(interactions):
        prefix = f"interactions[{n}]"
        if not isinstance(item, dict):
            errors.append(f"{prefix} must be an object")
            continue
        t = item.get("closest_approach_t")
        if not isinstance(t, (int, float)):
            errors.append(f"{prefix}.closest_approach_t must be a number")
        elif not (core[0] - tolerance <= t <= core[1] + tolerance):
            errors.append(f"{prefix}.closest_approach_t {t} outside core interval")
        for key in (
            "attempt_observable",
            "focal_contact",
            "disrupted_by_other_player",
            "possible_fake",
        ):
            if item.get(key) not in TRI_STATE:
                errors.append(f"{prefix}.{key} invalid")
        label = item.get("label")
        if label not in CLASSES:
            errors.append(f"{prefix}.label invalid")
            continue
        attempt = item.get("attempt_observable")
        contact = item.get("focal_contact")
        disrupted = item.get("disrupted_by_other_player")
        rules = {
            "clear_whiff": attempt == "yes" and contact == "no" and disrupted == "no",
            "disrupted_attempt": attempt == "yes" and contact != "yes" and disrupted == "yes",
            "successful_contact": contact == "yes",
            "no_observable_attempt": attempt in ("no", "unclear") and contact != "yes",
            "uncertain": True,
        }
        if not rules[label]:
            errors.append(f"{prefix}.label {label} inconsistent with answers")
        if label == "clear_whiff" and item.get("possible_fake") == "yes":
            errors.append(f"{prefix}.label clear_whiff but possible_fake=yes")
    return errors


def window_outcome(judgment: dict) -> str:
    """Coarse window-level outcome used for agreement summaries."""
    if not judgment.get("window_usable", True):
        return "unusable"
    labels = [item.get("label") for item in judgment.get("interactions", [])]
    for label in ("clear_whiff", "disrupted_attempt", "uncertain", "successful_contact"):
        if label in labels:
            return label
    return "no_observable_attempt"


class Transport:
    name = "base"

    def __init__(self, model: str, timeout: float = 300.0):
        self.model = model
        self.timeout = timeout

    def settings(self) -> dict:
        return {"transport": self.name, "model": self.model}

    def call(self, prompt: str, image_paths: list[pathlib.Path], work_dir: pathlib.Path) -> dict:
        raise NotImplementedError


class ClaudeCliTransport(Transport):
    """`claude -p` with only the Read tool, a minimal system prompt, and no MCP servers."""

    name = "claude-cli"

    def call(self, prompt, image_paths, work_dir):
        dirs = sorted({str(path.parent) for path in image_paths})
        command = [
            "claude",
            "-p",
            "--model",
            self.model,
            "--output-format",
            "json",
            "--tools",
            "Read",
            "--allowedTools",
            "Read",
            "--max-turns",
            "12",
            "--strict-mcp-config",
            "--no-session-persistence",
            "--disable-slash-commands",
            "--no-chrome",
            "--setting-sources",
            "",
            "--exclude-dynamic-system-prompt-sections",
            "--system-prompt",
            "You are a careful, skeptical visual annotator. Read every listed "
            "image with the Read tool, then answer with only the requested JSON object.",
        ]
        for directory in dirs:
            command += ["--add-dir", directory]
        started = time.monotonic()
        proc = subprocess.run(
            command,
            cwd=work_dir,
            capture_output=True,
            text=True,
            timeout=self.timeout,
            input=prompt,
        )
        latency = time.monotonic() - started
        try:
            envelope = json.loads(proc.stdout)
        except json.JSONDecodeError:
            return {
                "ok": False,
                "error": f"bad envelope: {proc.stderr[-500:]}",
                "latency": latency,
                "raw": proc.stdout,
            }
        usage = envelope.get("usage") or {}
        return {
            "ok": not envelope.get("is_error", False),
            "raw": envelope.get("result", ""),
            "latency": latency,
            "cost_usd": envelope.get("total_cost_usd"),
            "num_turns": envelope.get("num_turns"),
            "tokens": {
                key: usage.get(key)
                for key in (
                    "input_tokens",
                    "output_tokens",
                    "cache_read_input_tokens",
                    "cache_creation_input_tokens",
                )
            },
            "model_usage": list((envelope.get("modelUsage") or {}).keys()),
            "error": None if not envelope.get("is_error") else envelope.get("result"),
        }


class CodexCliTransport(Transport):
    """`codex exec` with attached images and a JSON output schema, read-only sandbox."""

    name = "codex-cli"

    def __init__(self, model: str | None, timeout: float = 600.0, reasoning: str = "medium"):
        super().__init__(model or "default", timeout)
        self.reasoning = reasoning

    def settings(self) -> dict:
        return {**super().settings(), "reasoning_effort": self.reasoning}

    def call(self, prompt, image_paths, work_dir):
        schema_path = work_dir / "response_schema.json"
        if not schema_path.exists():
            schema_path.write_text(json.dumps(RESPONSE_SCHEMA))
        with tempfile.NamedTemporaryFile("r", dir=work_dir, suffix=".txt", delete=False) as handle:
            out_path = pathlib.Path(handle.name)
        command = [
            "codex",
            "exec",
            "--skip-git-repo-check",
            "--ephemeral",
            "-s",
            "read-only",
            "-c",
            f'model_reasoning_effort="{self.reasoning}"',
            "--output-schema",
            str(schema_path),
            "-o",
            str(out_path),
        ]
        if self.model != "default":
            command += ["-m", self.model]
        for path in image_paths:
            command += ["-i", str(path)]
        command += [
            "--",
            prompt.replace(
                "Read every image file below before answering", "The attached images are, in order"
            ),
        ]
        started = time.monotonic()
        proc = subprocess.run(
            command,
            cwd=work_dir,
            capture_output=True,
            text=True,
            timeout=self.timeout,
            stdin=subprocess.DEVNULL,
        )
        latency = time.monotonic() - started
        raw = out_path.read_text() if out_path.exists() else ""
        out_path.unlink(missing_ok=True)
        model_line = re.search(r"^model: (.+)$", proc.stdout + proc.stderr, re.M)
        tokens = re.search(r"tokens used\s*\n\s*([\d,]+)", proc.stdout + proc.stderr)
        return {
            "ok": proc.returncode == 0 and bool(raw.strip()),
            "raw": raw,
            "latency": latency,
            "cost_usd": None,
            "tokens": {"total": int(tokens.group(1).replace(",", "")) if tokens else None},
            "model_usage": [model_line.group(1).strip()] if model_line else [],
            "error": None if proc.returncode == 0 else (proc.stderr or proc.stdout)[-800:],
        }


def make_transport(spec: str) -> Transport:
    kind, _, model = spec.partition(":")
    if kind == "claude":
        return ClaudeCliTransport(model or "claude-opus-5-5")
    if kind == "codex":
        return CodexCliTransport(model or None)
    raise ValueError(f"unknown transport {spec}")


class Ledger:
    """Append-only JSONL ledger of labeling attempts."""

    def __init__(self, path: pathlib.Path):
        self.path = path
        self.lock = threading.Lock()
        path.parent.mkdir(parents=True, exist_ok=True)

    def records(self) -> list[dict]:
        if not self.path.exists():
            return []
        return [json.loads(line) for line in self.path.read_text().splitlines() if line.strip()]

    def append(self, record: dict):
        # One write() per record on an O_APPEND descriptor keeps concurrent
        # writers from interleaving lines.
        data = (json.dumps(record) + "\n").encode()
        with self.lock:
            fd = os.open(self.path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o600)
            try:
                os.write(fd, data)
                os.fsync(fd)
            finally:
                os.close(fd)


def cache_key(task: dict, transport: Transport) -> str:
    material = json.dumps(
        {
            "neutral_id": task["neutral_id"],
            "pass": task["pass"],
            "representation": task["representation"],
            "image_sha256": task["image_sha256"],
            "prompt_version": PROMPT_VERSION,
            "prompt_sha256": hashlib.sha256(task["prompt"].encode()).hexdigest(),
            "settings": transport.settings(),
        },
        sort_keys=True,
    )
    return hashlib.sha256(material.encode()).hexdigest()


def run_tasks(
    tasks: list[dict],
    transport: Transport,
    ledger: Ledger,
    work_dir: pathlib.Path,
    max_calls: int,
    max_cost_usd: float,
    workers: int,
    max_retries: int = 2,
) -> dict:
    """Label tasks not yet validly labeled under the same cache key."""
    work_dir.mkdir(parents=True, exist_ok=True)
    existing = ledger.records()
    done = {r["cache_key"] for r in existing if r.get("valid")}
    failures: dict[str, int] = {}
    for r in existing:
        transport_failure = any(e.startswith("transport:") for e in r.get("validation_errors", []))
        if not r.get("valid") and not transport_failure:
            failures[r["cache_key"]] = failures.get(r["cache_key"], 0) + 1
    pending = []
    for task in tasks:
        key = cache_key(task, transport)
        if key in done or failures.get(key, 0) > max_retries:
            continue
        pending.append((key, task))
    stats = {"pending": len(pending), "calls": 0, "valid": 0, "invalid": 0, "cost_usd": 0.0}
    budget_lock = threading.Lock()

    def one(item):
        key, task = item
        with budget_lock:
            if stats["calls"] >= max_calls or stats["cost_usd"] >= max_cost_usd:
                return None
            stats["calls"] += 1
        attempts = failures.get(key, 0)
        while True:
            result = _call_once(task, transport, work_dir)
            record = _record(task, transport, key, result, attempts)
            ledger.append(record)
            with budget_lock:
                stats["cost_usd"] += result.get("cost_usd") or 0.0
                stats["valid" if record["valid"] else "invalid"] += 1
            attempts += 1
            if record["valid"] or attempts > max_retries:
                return record

    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        for _ in pool.map(one, pending):
            pass
    return stats


def _call_once(task, transport, work_dir):
    try:
        return transport.call(
            task["prompt"], [pathlib.Path(p) for p in task["image_paths"]], work_dir
        )
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": "timeout", "raw": "", "latency": transport.timeout}


def _record(task, transport, key, result, attempt_number):
    raw = result.get("raw") or ""
    parsed, errors = None, []
    if result.get("ok"):
        try:
            parsed = extract_json(raw)
            errors = validate_judgment(parsed, task["core_interval"])
        except (ValueError, json.JSONDecodeError) as error:
            errors = [f"parse: {error}"]
    else:
        errors = [f"transport: {result.get('error')}"]
    schema_errors = [e for e in errors if "inconsistent" not in e and "possible_fake" not in e]
    return {
        "record_type": "label_attempt",
        "attempt_id": hashlib.sha256(
            f"{key}:{attempt_number}:{time.time_ns()}".encode()
        ).hexdigest()[:20],
        "cache_key": key,
        "created_at": dt.datetime.now(dt.UTC).isoformat(),
        "neutral_id": task["neutral_id"],
        "pass": task["pass"],
        "representation": task["representation"],
        "transport": transport.settings(),
        "reported_models": result.get("model_usage"),
        "prompt_version": PROMPT_VERSION,
        "prompt_sha256": hashlib.sha256(task["prompt"].encode()).hexdigest(),
        "renderer_version": task["renderer_version"],
        "image_sha256": task["image_sha256"],
        "raw_response": raw,
        "raw_response_sha256": hashlib.sha256(raw.encode()).hexdigest(),
        "judgment": parsed,
        "validation_errors": errors,
        "valid": parsed is not None and not schema_errors,
        "consistent": parsed is not None and not errors,
        "latency_s": result.get("latency"),
        "cost_usd": result.get("cost_usd"),
        "tokens": result.get("tokens"),
        "retry_index": attempt_number,
    }
