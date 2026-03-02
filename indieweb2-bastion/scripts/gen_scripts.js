// Deno-only JS generator: reads .tmp/scripts.json (from Nickel export)
// Writes per-shell scripts from templates

const j = JSON.parse(await Deno.readTextFile(".tmp/scripts.json"));
function joinSteps(shell, steps) {
  if (shell === "powershell" || shell === "cmd") return steps.join("\n");
  return steps.join("\n");
}
for (const t of j.targets) {
  await Deno.mkdir(`scripts/${t.shell}`, { recursive: true });
  for (const c of j.commands) {
    const path = `scripts/${t.shell}/${c.name}${t.ext}`;
    const tpl = j.templates[t.template] || j.templates["bash"];
    const body = tpl
      .replaceAll("$shebang", t.shebang)
      .replaceAll("$name", c.name)
      .replaceAll("$description", c.description)
      .replaceAll("$steps", joinSteps(t.shell, c.steps));
    await Deno.writeTextFile(path, body);
    if (t.shell !== "cmd") await Deno.chmod(path, 0o755).catch(() => {});
  }
}
console.log("✅ Generated scripts for shells:", j.targets.map(t => t.shell).join(", "));
