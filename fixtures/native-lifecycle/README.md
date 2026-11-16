# Native lifecycle qualification driver

`driver.js` is the synthetic production-command/WebView driver retained from SQLx ticket 06 (`423c1fd`), promoted into an explicit current qualification entry point by ticket 07. It is injected only by the desktop `qualification` feature. It uses the unchanged Codex readiness fixture with provenance in `fixtures/codex-readiness/README.md`: Codex 0.169.0, source `d32063f96c2dff1dc96af839b3952abb6acfeda4`. All provider messages and login pages are synthetic; no authentication or chat is permitted.

Run the separately prepared qualification bundle through `corepack pnpm qualify:native -- <qualified.app> <new-report.json>`. The Python owner retains disposable roots/results, uses a 180-second total bound and terminates only retained child handles in failed-run cleanup. Recovery SIGKILL is never scored as completed Quit.
