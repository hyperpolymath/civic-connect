// Deno-only JS: produce docs/justfile-cookbook.adoc from just --list + Nickel export
const tasks = await Deno.readTextFile(".tmp/just_tasks.txt");
let spec = {};
try { spec = JSON.parse(await Deno.readTextFile(".tmp/scripts.json")); } catch {}
const header = `= Justfile Cookbook
:project-name: IndieWeb2

== 📖 Tasks
`;
const taskSection = "----\n" + tasks.trim() + "\n----\n";
const commandsSection = spec.commands ? `
== 🛠️ Generated Commands
${spec.commands.map(c=>`- ${c.name}: ${c.description}`).join("\n")}
` : "";
await Deno.writeTextFile("docs/justfile-cookbook.adoc", header + taskSection + commandsSection);
console.log("✅ Wrote docs/justfile-cookbook.adoc");
