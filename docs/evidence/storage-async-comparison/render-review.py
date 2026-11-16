#!/usr/bin/env python3
"""Render the bounded storage comparison as a static, portable HTML report."""

import argparse
import html
import json
from pathlib import Path


def escape(value):
    return html.escape(str(value), quote=True)


def validate(data):
    if not isinstance(data, dict):
        raise ValueError("data must be a JSON object")
    for key in ("date", "recommendation", "recommendation_reason", "sqlx_status", "bridge_status"):
        if not isinstance(data.get(key), str):
            raise ValueError(f"{key} must be a string")
    if data["recommendation"] not in {"SQLx", "rusqlite bridge", "Conditional"}:
        raise ValueError("recommendation must be SQLx, rusqlite bridge, or Conditional")
    statuses = {"Strong", "Worth exploring", "Speculative"}
    if data["sqlx_status"] not in statuses or data["bridge_status"] not in statuses:
        raise ValueError("candidate status must be Strong, Worth exploring, or Speculative")
    if not isinstance(data.get("metrics"), list):
        raise ValueError("metrics must be an array")
    for row in data["metrics"]:
        if not isinstance(row, dict) or any(
            not isinstance(row.get(key), (str, int, float)) or isinstance(row.get(key), bool)
            for key in ("metric", "sqlx", "bridge")
        ):
            raise ValueError("each metric needs metric, sqlx, and bridge display values")
    if not isinstance(data.get("limits"), list) or not all(isinstance(item, str) for item in data["limits"]):
        raise ValueError("limits must be an array of strings")
    paths = data.get("report_path", [])
    if isinstance(paths, str):
        paths = [paths]
    if not isinstance(paths, list) or not all(isinstance(item, str) for item in paths):
        raise ValueError("report_path must be a string or array of strings")
    return paths


def badge(status):
    tone = {"Strong": "strong", "Worth exploring": "explore", "Speculative": "speculative"}[status]
    return f'<span class="badge {tone}">{escape(status)}</span>'


def before_diagram(name):
    """SVG labels are static; the identifier is selected by this script only."""
    return f'''<svg viewBox="0 0 420 280" role="img" aria-labelledby="{name}-before-title">
      <title id="{name}-before-title">Current storage has useful depth, with blocking dispatch at callers</title>
      <defs><marker id="{name}-arrow" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0 0 L7 3.5 L0 7" fill="#64748b"/></marker></defs>
      <g font-family="system-ui,sans-serif" text-anchor="middle" font-size="13">
        <rect x="15" y="12" width="184" height="42" rx="8" fill="#fff" stroke="#94a3b8"/>
        <text x="107" y="38">Workspace requests</text>
        <rect x="219" y="12" width="186" height="42" rx="8" fill="#fff" stroke="#94a3b8"/>
        <text x="312" y="38">Provider persistence</text>
        <path d="M107 54 V83 H210 V105 M312 54 V83 H210" fill="none" stroke="#64748b" stroke-width="1.5" marker-end="url(#{name}-arrow)"/>
        <text x="210" y="76" font-size="11" fill="#64748b">caller-owned blocking dispatch</text>
        <path d="M25 112 H395" stroke="#94a3b8" stroke-dasharray="5 5"/>
        <rect x="55" y="127" width="310" height="90" rx="10" fill="#172b36"/>
        <text x="210" y="153" fill="#fff" font-weight="600">Existing deep storage module</text>
        <text x="210" y="176" fill="#d4e4eb" font-size="12">private opening · durable transactions</text>
        <text x="210" y="196" fill="#d4e4eb" font-size="12">saved identity · decoding · errors</text>
        <path d="M210 217 V238" stroke="#64748b" marker-end="url(#{name}-arrow)"/>
        <text x="210" y="261" fill="#475569">SQLite · synchronous connection</text>
      </g></svg>'''


def sqlx_after():
    return '''<div class="after-stack" aria-label="SQLx candidate storage implementation">
      <div class="diagram-box">Workspace / conversation / runtime owners</div>
      <div class="down-arrow">↓ <span>existing behavior-shaped seam</span></div>
      <div class="deep-box"><b>Storage module · async implementation</b>
        <div class="inside">SQLx queries + transactions<br>Maintained pool + bounded driver buffers</div>
      </div>
      <div class="down-arrow">↓</div>
      <div class="diagram-box">SQLite workers · explicit close</div>
      <p class="diagram-note">Retain admission, identity and completion policy.</p>
    </div>'''


def bridge_after():
    return '''<div class="after-stack" aria-label="Maintained rusqlite bridge candidate implementation">
      <div class="diagram-box">Workspace / conversation / runtime owners</div>
      <div class="down-arrow">↓ <span>existing behavior-shaped seam</span></div>
      <div class="deep-box"><b>Storage module · async implementation</b>
        <div class="inside">Whole rusqlite transaction closure<br>tokio-rusqlite connection worker</div>
      </div>
      <div class="down-arrow">↓</div>
      <div class="diagram-box">SQLite · existing opening flags retained</div>
      <p class="diagram-note">Bound submission; retain task and close outcomes.</p>
    </div>'''


def candidate(name, title, status, problem, solution, wins, after, extra=""):
    win_html = "".join(f"<li>{escape(win)}</li>" for win in wins)
    return f'''<article id="{name}" class="candidate">
      <div class="candidate-heading"><h2>{escape(title)}</h2>{badge(status)}</div>
      <p class="files"><b>Files</b> · crates/core/src/storage.rs · workspaces.rs ·
        connection.rs · runtimes/provider_connection/{{login,selection}}.rs · supervisor/src/main.rs</p>
      <div class="diagrams"><figure><figcaption>Before · useful depth, scattered dispatch</figcaption>{before_diagram(name)}</figure>
        <figure><figcaption>After · candidate implementation</figcaption>{after}</figure></div>
      <dl class="argument"><div><dt>Problem</dt><dd>{escape(problem)}</dd></div>
        <div><dt>Solution</dt><dd>{escape(solution)}</dd></div></dl>
      <ul class="wins">{win_html}</ul>{extra}
    </article>'''


def render(data):
    paths = validate(data)
    metrics = "".join(
        f'<tr><th scope="row">{escape(row["metric"])}</th><td>{escape(row["sqlx"])}</td><td>{escape(row["bridge"])}</td></tr>'
        for row in data["metrics"]
    ) or '<tr><td colspan="3">No measured results supplied.</td></tr>'
    limits = "".join(f"<li>{escape(item)}</li>" for item in data["limits"])
    sources = "".join(f"<li><code>{escape(path)}</code></li>" for path in paths)
    selected = data["recommendation"]
    anchor = {"SQLx": "sqlx", "rusqlite bridge": "bridge", "Conditional": "evidence"}[selected]
    heading = {"SQLx": "Choose SQLx", "rusqlite bridge": "Choose the maintained rusqlite bridge", "Conditional": "Resolve the remaining qualification gates"}[selected]
    sqlx = candidate(
        "sqlx", "SQLx · maintained async composition", data["sqlx_status"],
        "Synchronous persistence makes callers carry database dispatch; connection reuse and async composition are separate maintenance work.",
        "Keep the storage interface and domain owners; place SQLx query, transaction, pool and close mechanics inside its implementation.",
        ["Locality: connection mechanics stay inside", "Leverage: awaited persistence across callers", "Controlled outcomes preserve policy tests"], sqlx_after(),
        '<p class="callout">Opening gate: SQLx 0.9 has no supported NOFOLLOW option. Preserving the current guard needs upstream support or explicit design judgment; this is not a blanket safety verdict.</p>',
    )
    bridge = candidate(
        "bridge", "rusqlite · maintained async bridge", data["bridge_status"],
        "Direct synchronous calls spread blocking dispatch while hand-built async workers would duplicate a maintained mechanism.",
        "Keep whole transaction closures behind storage; use tokio-rusqlite and retain bounded admission, operation results and explicit drain.",
        ["Locality: closure details stay internal", "Leverage: reuse existing transaction semantics", "Controlled outcomes preserve policy tests"], bridge_after(),
        '<p class="callout neutral">The bridge queue is unbounded. External admission must cap submitted work; retained operation failures remain visible even when close reports success.</p>',
    )
    return '''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Async storage · architecture comparison</title>
<script src="https://cdn.tailwindcss.com"></script>
<script type="module">import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs"; mermaid.initialize({startOnLoad:true,theme:"neutral",securityLevel:"strict"});</script>
<style>
:root{color-scheme:light;--ink:#172b36;--muted:#5f6e75;--line:#d8dfdf;--paper:#f7f7f3;--accent:#116859}*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:var(--paper);color:var(--ink);font:15px/1.55 system-ui,-apple-system,sans-serif}main{max-width:1120px;margin:auto;padding:52px 28px 72px}header{margin-bottom:32px}.eyebrow,figcaption,dt{font-size:11px;text-transform:uppercase;letter-spacing:.1em;font-weight:700}.eyebrow{color:var(--accent)}h1,h2,h3,p{margin-top:0}h1{font:clamp(34px,5vw,54px)/1.08 Georgia,serif;letter-spacing:-.035em;margin-bottom:17px}h2{font:27px/1.2 Georgia,serif;margin-bottom:0}h3{font-size:18px}.legend{display:flex;gap:20px;flex-wrap:wrap;color:var(--muted);font-size:12px}.legend span{display:flex;align-items:center;gap:7px}.legend i{display:inline-block;width:22px;height:13px;border:1px solid #94a3b8}.legend .seam{height:0;border:0;border-top:2px dashed #94a3b8}.legend .deep{background:var(--ink);border:3px solid var(--ink)}.candidate,.panel,.recommendation{border:1px solid var(--line);border-radius:14px;background:white;padding:28px;margin-bottom:28px}.candidate-heading{display:flex;justify-content:space-between;align-items:center;gap:14px}.badge{white-space:nowrap;border-radius:99px;font-size:11px;font-weight:700;padding:5px 10px}.strong{color:#075942;background:#d9f3e7}.explore{color:#83540b;background:#fff0cf}.speculative{color:#475569;background:#edf0f3}.files{font:11px/1.7 ui-monospace,monospace;color:var(--muted);margin:15px 0 22px;overflow-wrap:anywhere}.diagrams{display:grid;grid-template-columns:1fr 1fr;gap:18px}figure{margin:0;border:1px solid #e0e5e5;border-radius:9px;background:#f8faf9;padding:16px;min-width:0}figcaption{color:var(--muted);margin-bottom:12px}svg{width:100%;height:280px;display:block}.after-stack{height:280px;text-align:center;display:flex;flex-direction:column;justify-content:center;gap:0}.diagram-box{font-size:12px;background:#fff;border:1px solid #aab8bf;border-radius:7px;padding:10px}.down-arrow{font-size:18px;color:#64748b;line-height:1.5}.down-arrow span{font-size:10px;margin-left:7px}.deep-box{background:var(--ink);border:3px solid var(--ink);border-radius:9px;padding:12px 10px;color:#fff;font-size:12px}.inside{color:#d4e4eb;font-size:11px;margin-top:8px}.diagram-note{font-size:10px;color:var(--muted);margin:8px 0 0}.argument{margin:22px 0 16px;display:grid;grid-template-columns:1fr 1fr;gap:22px}.argument dd{margin:4px 0 0;font-size:13px}.wins{list-style:none;display:flex;gap:8px 20px;flex-wrap:wrap;padding:0;margin:0;color:var(--accent);font-size:12px}.wins li:before{content:"+ ";font-weight:700}.callout{font-size:12px;background:#fff6df;border-left:3px solid #d6a13d;padding:11px 13px;margin:20px 0 0}.callout.neutral{background:#eef4f3;border-color:#6e9691}.panel h2{margin-bottom:18px}.table-scroll{overflow:auto}table{width:100%;border-collapse:collapse;font-size:12px}th,td{border-bottom:1px solid #e0e5e5;padding:12px;text-align:left;vertical-align:top}thead th{color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.06em}tbody th{font-weight:600;width:30%}.note{font-size:12px;color:var(--muted);margin:16px 0 0}.probe-list,.limits{padding-left:20px;font-size:13px}.probe-list li,.limits li{margin:8px 0}.recommendation{border:2px solid var(--accent);background:#eff7f2}.recommendation h2{margin:8px 0 12px}.recommendation p{max-width:80ch;margin-bottom:16px}a{color:var(--accent);text-decoration:underline;text-underline-offset:3px}details{margin-top:20px;font-size:12px}summary{cursor:pointer;color:var(--muted)}.mermaid{overflow:auto;margin:14px 0 0;max-width:100%}.mermaid svg{height:auto;max-height:300px}footer{color:var(--muted);font-size:11px}footer ul{list-style:none;padding:0}footer code{overflow-wrap:anywhere}@media(max-width:760px){main{padding:30px 16px}.candidate,.panel,.recommendation{padding:20px}.candidate-heading{align-items:flex-start;flex-direction:column}.diagrams,.argument{grid-template-columns:1fr}.legend{gap:10px}h2{font-size:24px}}@media print{body{background:white}main{max-width:none;padding:0}.candidate,.panel,.recommendation{break-inside:avoid}details{display:none}}
</style></head><body class="bg-stone-50 text-slate-900"><main>
<header><p class="eyebrow">Agent Workspace · ''' + escape(data["date"]) + '''</p><h1>Async storage, with ownership intact.</h1>
<div class="legend"><span><i></i> module</span><span><i class="seam"></i> seam</span><span><i class="deep"></i> deep module</span></div></header>
''' + sqlx + bridge + '''
<section class="panel" id="evidence"><h2>Measured comparison</h2><div class="table-scroll"><table><thead><tr><th>Measure / gate</th><th>SQLx</th><th>rusqlite bridge</th></tr></thead><tbody>''' + metrics + '''</tbody></table></div>
<p class="note">Read each figure with its cohort, engine, topology and durability settings. Application units, isolated SQLite evidence, fixture desktop checks and native qualification are distinct.</p>
<ul class="limits">''' + limits + '''</ul></section>
<section class="panel"><h2>Keep policy; delegate mechanisms.</h2><div class="table-scroll"><table><thead><tr><th>Responsibility</th><th>SQLx</th><th>rusqlite bridge</th></tr></thead><tbody>
<tr><th>Worker and reply delivery</th><td>Maintained SQLite driver</td><td>Maintained tokio-rusqlite worker</td></tr>
<tr><th>Reuse / reader acquisition</th><td>Maintained pool</td><td>Reusable handles; multi-reader selection remains owned if needed</td></tr>
<tr><th>Transaction mechanics</th><td>Async transaction ownership</td><td>Whole synchronous transaction closure</td></tr>
<tr><th>Buffering</th><td>Bounded command / row buffers</td><td>Unbounded internal queue; cap submissions externally</td></tr>
<tr><th>Current private opening guard</th><td>Supported NOFOLLOW equivalent unresolved</td><td>Supported flags retained</td></tr>
<tr><th>Admission, bytes and fairness</th><td colspan="2">Meaningful application policy; pool acquisition is not workspace fairness</td></tr>
<tr><th>Durable action identity / recovery</th><td colspan="2">Owning Rust modules; no automatic replay of uncertain effects</td></tr>
<tr><th>Physical completion</th><td colspan="2">Retain admitted operations, fence admission, drain and close; distinguish effect completion from OS-thread exit</td></tr>
<tr><th>Checked queries / migrations</th><td>Optional tooling; metadata and preparation cost</td><td>Runtime queries; selected schema initialization remains owned</td></tr>
</tbody></table></div><p class="note">Prefer maintained built-ins strongly when they fit the required behavior. Their benefit is net code and guarantee ownership, not unconditional delegation of domain policy.</p>
<details><summary>Shared completion flow · mechanism graph</summary><pre class="mermaid">flowchart LR
  A[Owning core module] --> B[Bounded admission]
  B --> C[Retained storage operation]
  C --> D[Maintained SQLite worker]
  D --> E[Recorded outcome]
  E --> F[Drain then close]
  classDef deep fill:#172b36,color:#fff,stroke:#172b36;
  class C deep;
</pre></details></section>
<section class="panel"><h2>Future direction · probes, not promises</h2><ul class="probe-list">
<li><b>Local PR self-review:</b> exact comparison identity, draft revisions and retained human edits.</li>
<li><b>Automatic review handoff:</b> independent context, editable findings and explicit publication authority.</li>
<li><b>Chained sessions / shared context / coordinators:</b> durable trigger identity, versioned context and separate execution outcomes.</li>
<li><b>Unified inbox / Jira / GitHub sync:</b> remote identities, cursors and uncertain-write reconciliation stay with their owner.</li>
<li><b>Manual and agent controls:</b> one domain action policy, regardless of the initiator.</li>
</ul><p class="note">These probes assess locality and leverage. They add no production interfaces, workflow engine or feature commitments.</p></section>
<section class="recommendation" id="top-recommendation"><p class="eyebrow">Top recommendation</p><h2>''' + escape(heading) + '''</h2><p>''' + escape(data["recommendation_reason"]) + '''</p><a href="#''' + anchor + '''">''' + ("Review remaining evidence" if selected == "Conditional" else "Explore this candidate") + ''' →</a></section>
<footer><b>Evidence and source notes</b><ul>''' + sources + '''</ul><p>The existing storage module earns its depth. Blocking I/O alone does not make its interface shallow; the comparison concerns the implementation and mechanism ownership behind its seam.</p></footer>
</main></body></html>'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True, help="Comparison data JSON")
    parser.add_argument("--output", type=Path, required=True, help="Destination HTML, normally in the OS temp directory")
    args = parser.parse_args()
    try:
        data = json.loads(args.data.read_text(encoding="utf-8"))
        document = render(data)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    output = args.output.expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(document, encoding="utf-8")
    print(json.dumps({"output": str(output), "bytes": output.stat().st_size}))


if __name__ == "__main__":
    main()
