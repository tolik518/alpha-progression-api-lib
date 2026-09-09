#!/usr/bin/env python3
import base64, os, re

PKG      = "alpha-progression-wasm/pkg"
HTML_SRC = "example/index.html"
HTML_OUT = "example/index.bundle.html"

with open(f"{PKG}/alpha_progression_wasm_bg.wasm", "rb") as f:
    wasm_b64 = base64.b64encode(f.read()).decode()

with open(f"{PKG}/alpha_progression_wasm.js") as f:
    wasm_js = f.read()

with open(HTML_SRC) as f:
    html = f.read()

app_js_start = html.index('<script type="module">')
app_js_end   = html.index('</script>', app_js_start) + len('</script>')
app_js_inner = html[app_js_start + len('<script type="module">') : app_js_end - len('</script>')]

app_js_inner = re.sub(r"\s*import init.*?alpha_progression_wasm\.js.*?;\s*\n", "\n", app_js_inner)
app_js_inner = app_js_inner.replace(
    "  await init();\n  const client = new Client();",
    "  if (!window.__wasm_ready) await new Promise(r => window.addEventListener('wasm-ready', r, {once: true}));\n"
    "  const client = new wasm_bindgen.Client();"
)

app_script = f"<script>\n(async () => {{{app_js_inner}}})();\n</script>"

loader_script = f"""<script>
{wasm_js}
</script>
<script>
(async () => {{
  const b64   = "{wasm_b64}";
  const bytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
  await wasm_bindgen({{ module_or_path: bytes.buffer }});
  window.__wasm_ready = true;
  window.dispatchEvent(new Event('wasm-ready'));
}})();
</script>"""

result = html[:app_js_start] + loader_script + "\n" + app_script + html[app_js_end:]

with open(HTML_OUT, "w") as f:
    f.write(result)

print(f"=> {HTML_OUT}  ({os.path.getsize(HTML_OUT) // 1024} KB)")