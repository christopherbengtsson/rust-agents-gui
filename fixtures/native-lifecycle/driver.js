// Injected only by the feature-gated qualification build into its actual WebView.
// All requests use production Tauri commands; provider traffic is synthetic.
(async () => {
  const cfg = window.nativeFixture;
  const invoke = window.__TAURI_INTERNALS__.invoke;
  const record = (stage, details = {}) =>
    invoke("record", { value: { stage, ...details } });
  const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const until = async (read, accepts, label) => {
    const deadline = Date.now() + 25000;
    while (Date.now() < deadline) {
      const value = await read();
      if (accepts(value)) return value;
      await pause(30);
    }
    throw new Error(`Fixture deadline: ${label}`);
  };
  try {
    const snapshot = await until(
      () => invoke("core_status"),
      (value) =>
        value.status.state === "ready" || value.status.state === "failed",
      "core startup",
    );
    await record("startup", { snapshot, pid: await invoke("desktop_pid") });
    if (cfg.mode === "compete") {
      if (snapshot.status.state !== "failed")
        throw new Error("Competing owner admitted");
      await record("excluded", { snapshot });
      return;
    }
    if (snapshot.status.state !== "ready")
      throw new Error(snapshot.diagnostic ?? "Core not ready");
    const incarnation = snapshot.cursor.incarnation;
    const request = async (command, request) => {
      const reply = await invoke(command, {
        expectedIncarnation: incarnation,
        request,
      });
      if (reply.incarnation !== incarnation)
        throw new Error("Serving incarnation mismatch");
      if (reply.result.Err) throw new Error(JSON.stringify(reply.result.Err));
      return reply.result.Ok;
    };
    const repo = (value) => request("repository_request", value);
    const workspace = (value) => request("workspace_request", value);
    const provider = (value) => request("provider_request", value);
    const { target } = await provider({ operation: "target" });
    const observe = async () =>
      (
        await provider({
          operation: "observe",
          target_id: target.id,
          refresh: false,
        })
      ).observation;
    if (cfg.mode === "reopen") {
      const observation = await observe();
      if (
        incarnation === cfg.previousIncarnation ||
        observation.login?.id !== cfg.attempt ||
        observation.login.phase !== "cleanup_uncertain"
      )
        throw new Error("Interrupted login recovery lost identity/fence");
      const rejected = await invoke("provider_request", {
        expectedIncarnation: incarnation,
        request: {
          operation: "start_browser_login",
          target_id: target.id,
          attempt_id: crypto.randomUUID(),
          expected_revision: observation.revision,
        },
      });
      if (!rejected.result.Err)
        throw new Error("Surviving-work fence admitted replacement");
      await record("recovered", {
        incarnation,
        observation,
        rejected,
        repositories: await repo({
          operation: "list",
          after: null,
          refresh: false,
        }),
      });
      return;
    }
    if (cfg.mode === "replacement") {
      const observation = await observe();
      if (
        incarnation !== cfg.previousIncarnation ||
        observation.login?.id !== cfg.attempt ||
        observation.login.phase !== "pending"
      )
        throw new Error("WebView replacement changed native work");
      await record("reattached", {
        incarnation,
        observation,
        text: document.body.innerText.slice(0, 1800),
      });
      history.pushState(null, "", "/settings/general");
      dispatchEvent(new PopStateEvent("popstate"));
      await pause(150);
      const returnLink = await until(
        async () =>
          Array.from(document.querySelectorAll("a")).find((link) =>
            link.textContent.includes("Codex sign-in"),
          ),
        Boolean,
        "visible progress return link",
      );
      returnLink.click();
      const cancel = await until(
        async () =>
          Array.from(document.querySelectorAll("button")).find(
            (button) =>
              button.textContent.trim() === "Cancel sign-in" &&
              !button.disabled,
          ),
        Boolean,
        "visible cancel control",
      );
      cancel.click();
      const cancelled = await until(
        observe,
        (value) => value.login?.phase === "cancelled",
        "cancel exact attempt",
      );
      await record("cancelled", {
        incarnation,
        observation: cancelled,
        text: document.body.innerText.slice(0, 2000),
      });
      return;
    }
    if (cfg.mode === "initial") {
      const retention = new Promise((resolve) =>
        window.addEventListener(
          "native-retention-result",
          (event) => resolve(event.detail),
          { once: true },
        ),
      );
      await pause(200);
      window.dispatchEvent(new Event("native-retention-measure"));
      const retained = await Promise.race([
        retention,
        pause(2000).then(() => {
          throw new Error("Native retention instrumentation unavailable");
        }),
      ]);
      if (retained.inactive > 128)
        throw new Error("Inactive retention exceeded count policy");
      await record("retention", {
        retained,
        heap: performance.memory?.usedJSHeapSize ?? null,
        heapLimit:
          "WebKit heap API unavailable when null; count is not a byte guarantee",
      });
      const mismatch = await invoke("repository_request", {
        expectedIncarnation: "retired",
        request: { operation: "list", after: null, refresh: false },
      }).then(
        () => null,
        (error) => error,
      );
      if (mismatch?.error?.code !== "incarnation_changed")
        throw new Error("Stale incarnation was not rejected");
      const action_id = crypto.randomUUID();
      await repo({
        operation: "register",
        request: {
          action_id,
          path: cfg.repository,
          create_initial_workspace: true,
        },
      });
      const registration = (
        await until(
          () => repo({ operation: "action", action_id }),
          (value) => value.registration?.state === "completed",
          "registration",
        )
      ).registration;
      const repository_id = registration.repository_id;
      const initial = await until(
        () =>
          workspace({
            operation: "get",
            id: registration.initial_workspace_id,
          }),
        (value) => value.workspace?.status?.state === "ready",
        "first workspace",
      );
      const createIds = [crypto.randomUUID(), crypto.randomUUID()];
      const concurrent = await Promise.all([
        ...createIds.map((action_id) =>
          workspace({ operation: "create", repository_id, action_id }),
        ),
        workspace({
          operation: "changes",
          workspace_id: initial.workspace.id,
          refresh: true,
          comparison_id: null,
          after: null,
        }),
      ]);
      const created = [];
      for (const action_id of createIds)
        created.push(
          await until(
            () => workspace({ operation: "action", action_id }),
            (value) => value.workspace?.status?.state === "ready",
            "concurrent creation",
          ),
        );
      await record("workspaces", {
        registration,
        initial,
        concurrent,
        created,
      });
    }
    // The fixture is captured at launch. No real provider or authentication is used.
    history.pushState(null, "", "/settings/providers?provider=codex");
    dispatchEvent(new PopStateEvent("popstate"));
    await provider({
      operation: "observe",
      target_id: target.id,
      refresh: true,
    });
    const readiness = await until(
      observe,
      (value) => value.phase === "observed",
      "readiness",
    );
    const attempt_id = crypto.randomUUID();
    await provider({
      operation: "start_browser_login",
      target_id: target.id,
      attempt_id,
      expected_revision: readiness.revision,
    });
    const pending = await until(
      observe,
      (value) => value.login?.phase === "pending",
      "synthetic login pending",
    );
    if (cfg.mode === "active-quit") {
      await record("active-quit-ready", {
        incarnation,
        attempt: attempt_id,
        observation: pending,
      });
      return;
    }
    if (cfg.mode === "crash") {
      await record("kill-ready", {
        incarnation,
        attempt: attempt_id,
        observation: pending,
      });
      return;
    }
    history.pushState(null, "", "/settings/general");
    dispatchEvent(new PopStateEvent("popstate"));
    await pause(200);
    await record("replace-ready", {
      incarnation,
      attempt: attempt_id,
      observation: pending,
      text: document.body.innerText.slice(0, 2000),
    });
    await pause(2500);
    await invoke("replace_webview");
  } catch (error) {
    await record("failed", { error: String(error), stack: error.stack });
  }
})();
