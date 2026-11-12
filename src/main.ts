import { invoke } from "@tauri-apps/api/core";
import "./style.css";

interface SupervisorStatus {
  state: "ready";
  supervisor_pid: number;
  application_version: string;
}

const status = document.querySelector<HTMLParagraphElement>("#status")!;
const details = document.querySelector<HTMLDListElement>("#details")!;
const refresh = document.querySelector<HTMLButtonElement>("#refresh")!;

async function refreshStatus(): Promise<void> {
  refresh.disabled = true;
  status.textContent = "Connecting…";
  details.replaceChildren();
  try {
    const snapshot = await invoke<{
      status: SupervisorStatus;
      cursor: { owner: string; revision: number };
      reset: boolean;
    }>("supervisor_status");
    const observation = snapshot.status;
    status.textContent = "Connected · Supervisor ready";
    status.dataset.state = observation.state;
    for (const [label, value] of [
      ["Supervisor process", observation.supervisor_pid],
      ["Desktop process", await invoke<number>("desktop_pid")],
      ["Application version", observation.application_version],
      ["Owner identity", snapshot.cursor.owner],
      ["Snapshot revision", snapshot.cursor.revision],
    ]) {
      const term = document.createElement("dt");
      term.textContent = String(label);
      const description = document.createElement("dd");
      description.textContent = String(value);
      details.append(term, description);
    }
  } catch (error) {
    status.dataset.state = "error";
    status.textContent = String(error);
  } finally {
    refresh.disabled = false;
  }
}

refresh.addEventListener("click", () => void refreshStatus());
void refreshStatus();
