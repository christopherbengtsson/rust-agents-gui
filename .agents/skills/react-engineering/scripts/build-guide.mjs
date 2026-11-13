import { readFileSync, readdirSync, writeFileSync, existsSync } from "node:fs";
import { dirname, resolve, relative, sep, isAbsolute } from "node:path";
import { fileURLToPath } from "node:url";

const skillRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = resolve(skillRoot, "../../..");
const output = resolve(skillRoot, "AGENTS.md");
const requiredFields = ["title", "impact", "impactDescription", "tags"];

// This authored format uses single-line scalars, not general-purpose YAML.
function document(source, label) {
  const match = source.match(/^---\n([\s\S]*?)\n---\n\n([\s\S]+)$/);
  if (!match) throw new Error(`${label}: expected frontmatter and Markdown body`);
  const metadata = {};
  for (const line of match[1].split("\n")) {
    const field = line.match(/^([A-Za-z]+): (.+)$/);
    if (!field || Object.hasOwn(metadata, field[1])) {
      throw new Error(`${label}: invalid or duplicate metadata field: ${line}`);
    }
    const raw = field[2];
    const value = raw.startsWith('"') ? JSON.parse(raw) : raw;
    if (typeof value !== "string" || !value.trim()) {
      throw new Error(`${label}: metadata values must be nonempty single-line strings`);
    }
    metadata[field[1]] = value;
  }
  return { metadata, body: match[2] };
}

function slug(heading) {
  return heading.toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu, "").replaceAll(" ", "-");
}

function headings(source) {
  const found = new Set();
  const occurrences = new Map();
  for (const match of source.matchAll(/^#{1,6} (.+)$/gm)) {
    const base = slug(match[1]);
    const count = occurrences.get(base) ?? 0;
    occurrences.set(base, count + 1);
    found.add(count ? `${base}-${count}` : base);
  }
  return found;
}

function markdownLinks(source) {
  // Rule sources use inline links without optional title syntax.
  return [...source.matchAll(/\[[^\]\n]+\]\(([^\s)]+)\)/g)].map(match => match[1]);
}

function checkLinks(source, from, generated) {
  for (const target of markdownLinks(source)) {
    if (/^[A-Za-z][A-Za-z\d+.-]*:/.test(target) || target.startsWith("//")) continue;
    const [path, fragment] = target.split("#");
    const destination = path ? resolve(dirname(from), decodeURIComponent(path)) : from;
    const withinRepository = relative(repositoryRoot, destination);
    if (withinRepository.startsWith(`..${sep}`) || isAbsolute(withinRepository)) {
      throw new Error(`${relative(skillRoot, from)}: link leaves repository: ${target}`);
    }
    const text = destination === output ? generated :
      existsSync(destination) ? readFileSync(destination, "utf8") : undefined;
    if (text === undefined) throw new Error(`${relative(skillRoot, from)}: missing link ${target}`);
    if (fragment && !headings(text).has(decodeURIComponent(fragment))) {
      throw new Error(`${relative(skillRoot, from)}: missing anchor ${target}`);
    }
  }
}

function rebaseLinks(source, from) {
  return source.replace(/(\[[^\]\n]+\]\()([^\s)]+)(\))/g, (whole, start, target, end) => {
    if (/^[A-Za-z][A-Za-z\d+.-]*:/.test(target) || target.startsWith("//")) return whole;
    const [path, fragment] = target.split("#");
    const destination = path ? resolve(dirname(from), decodeURIComponent(path)) : from;
    const rebased = relative(skillRoot, destination).split(sep).join("/");
    return `${start}${rebased}${fragment ? `#${fragment}` : ""}${end}`;
  });
}

function build() {
  const skillFile = resolve(skillRoot, "SKILL.md");
  const { body: skill } = document(readFileSync(skillFile, "utf8"), "SKILL.md");
  const before = skill.indexOf("## Rule index\n");
  const after = skill.indexOf("## Completion\n", before);
  if (before < 0 || after < 0) throw new Error("SKILL.md: missing index/completion boundaries");
  const index = skill.slice(before + "## Rule index\n".length, after);
  const groups = [];
  const indexed = new Set();
  let group;
  for (const line of index.split("\n")) {
    if (!line.trim()) continue;
    if (line.startsWith("### ")) {
      group = { title: line.slice(4), rules: [] };
      if (groups.some(item => item.title === group.title)) throw new Error("Duplicate index category");
      groups.push(group);
      continue;
    }
    const entry = line.match(/^- \[([a-z0-9-]+)\]\(rules\/([a-z0-9-]+)\.md\) — (.+)$/);
    if (!group || !entry || entry[1] !== entry[2]) throw new Error(`Invalid index entry: ${line}`);
    const id = entry[1];
    if (indexed.has(id)) throw new Error(`Duplicate index entry: ${id}`);
    indexed.add(id);
    group.rules.push({ id, summary: entry[3] });
  }
  if (!groups.length || groups.some(item => !item.rules.length)) throw new Error("Empty index category");
  const disk = readdirSync(resolve(skillRoot, "rules"));
  if (disk.some(file => !/^[a-z0-9]+(?:-[a-z0-9]+)*\.md$/.test(file))) {
    throw new Error("rules/: expected only named Markdown rule files");
  }
  const missing = [...indexed].filter(id => !disk.includes(`${id}.md`));
  const unindexed = disk.filter(file => !indexed.has(file.slice(0, -3)));
  if (missing.length || unindexed.length) {
    throw new Error(`Index mismatch; missing files: ${missing.join(", ")}; unindexed: ${unindexed.join(", ")}`);
  }
  const sources = [];
  const sections = groups.map(category => {
    const rules = category.rules.map(({ id, summary }) => {
      const path = resolve(skillRoot, "rules", `${id}.md`);
      const source = readFileSync(path, "utf8");
      const { metadata, body } = document(source, id);
      if (Object.keys(metadata).length !== requiredFields.length ||
          requiredFields.some(field => !Object.hasOwn(metadata, field)) ||
          !["LOW", "MEDIUM", "HIGH", "CRITICAL"].includes(metadata.impact) ||
          !/^[A-Za-z0-9-]+(?:, [A-Za-z0-9-]+)*$/.test(metadata.tags)) {
        throw new Error(`${id}: invalid rule metadata`);
      }
      if (!body.startsWith(`# ${metadata.title}\n`) ||
          !body.includes("**Incorrect (") || !body.includes("**Correct (") ||
          !/\nSources?: \[/.test(body)) {
        throw new Error(`${id}: expected matching title, example pair, and sources`);
      }
      sources.push({ path, source: body });
      const expanded = rebaseLinks(body, path).replace(/^(#{1,3}) /gm, "###$1 ");
      return `### ${id}\n\n${summary}\n\nImpact: ${metadata.impact} — ${metadata.impactDescription}.\n\nSource: [rules/${id}.md](rules/${id}.md)\n\n${expanded.trim()}`;
    });
    return `## ${category.title}\n\n${rules.join("\n\n")}`;
  });
  const overview = skill.slice(0, before).replace(/^# React engineering\n\n/, "").trim();
  const generated = `# React engineering — full guide\n\n<!-- Generated by scripts/build-guide.mjs. Edit SKILL.md and rules/*.md; regenerate. -->\n\n${overview}\n\n${sections.join("\n\n")}\n\n${skill.slice(after).trim()}\n`;
  for (const { path, source } of sources) checkLinks(source, path, generated);
  checkLinks(skill, skillFile, generated);
  checkLinks(generated, output, generated);
  return { generated, count: indexed.size };
}

try {
  if (process.argv.slice(2).some(arg => arg !== "--check") || process.argv.length > 3) {
    throw new Error("Usage: node scripts/build-guide.mjs [--check]");
  }
  const { generated, count } = build();
  if (process.argv.includes("--check")) {
    if (!existsSync(output) || readFileSync(output, "utf8") !== generated) {
      throw new Error("AGENTS.md is missing or stale; run scripts/build-guide.mjs");
    }
    console.log(`Checked ${count} rules, index, local links/anchors, and generated AGENTS.md`);
  } else {
    writeFileSync(output, generated);
    console.log(`Generated AGENTS.md from ${count} rules`);
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
