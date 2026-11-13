// Injected via the actual WKWebView Inspector console for opt-in qualification.
// Observes native trusted tab clicks; never replaces IPC, data or React behavior.
(() => {
  window.__inspectionQualification?.dispose?.();
  const records = [];
  const observers = new Set();
  const click = (event) => {
    const tab = event.target.closest('[role="tab"]');
    if (
      !tab ||
      tab.getAttribute("aria-selected") === "true" ||
      records.length >= 12
    )
      return;
    const record = {
      sample: records.length + 1,
      target: tab.textContent.trim(),
      trusted: event.isTrusted,
      input: performance.now(),
      eventTime: event.timeStamp,
      rowsBefore: document.querySelectorAll(".diff-row").length,
      viewport: [innerWidth, innerHeight],
    };
    records.push(record);
    console.timeStamp(`05c.nav.${record.sample}.input`);
    const observer = new MutationObserver(() => {
      if (
        document
          .querySelector('[role="tab"][aria-selected="true"]')
          ?.textContent.trim() !== record.target
      )
        return;
      observer.disconnect();
      observers.delete(observer);
      record.commit = performance.now();
      record.rowsAfter = document.querySelectorAll(".diff-row").length;
      console.timeStamp(`05c.nav.${record.sample}.commit`);
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          record.secondFrame = performance.now();
          console.timeStamp(`05c.nav.${record.sample}.secondFrame`);
          if (records.length === 12)
            requestAnimationFrame(() =>
              requestAnimationFrame(() => console.profileEnd("05c-navigation")),
            );
        }),
      );
    });
    observers.add(observer);
    observer.observe(document.querySelector(".inspection-tabs"), {
      subtree: true,
      childList: true,
      attributes: true,
    });
  };
  document.addEventListener("click", click, true);
  window.__inspectionQualification = {
    records,
    userAgent: navigator.userAgent,
    timeOrigin: performance.timeOrigin,
    assets: [...document.scripts].map((script) => script.src),
    dispose() {
      document.removeEventListener("click", click, true);
      observers.forEach((observer) => observer.disconnect());
    },
  };
  console.profile("05c-navigation");
  return "05c navigation observation installed";
})();
