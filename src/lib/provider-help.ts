import { openUrl } from "@tauri-apps/plugin-opener";
export type ProviderHelpTopic = "install" | "authentication";
const destinations = {
  install: "https://developers.openai.com/codex/cli",
  authentication: "https://developers.openai.com/codex/auth",
} as const;
export function openProviderHelp(topic: ProviderHelpTopic) {
  return openUrl(destinations[topic]);
}
