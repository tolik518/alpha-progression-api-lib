// Resolves Alpha Progression's apiKeys out of the obfuscated app bundle, mapped to
// the endpoint each one belongs to. Runs on node or deno, no dependencies.
//
//   node tools/extract-keys.mjs path/to/assets/public/assets/index-<hash>.js
//
// The bundle is obfuscated with string-array indirection: call sites read
// {'apiKey':_0x2a8f49(0x41e)} rather than holding the literal. Rather than trying to
// decode that by hand, this reuses the bundle's own decoder — it lifts the string
// array, the rotation preamble and the decoder function out of the file and re-runs
// just those three pieces in isolation.
import { readFileSync } from "node:fs";

const path = process.argv[2];
if (!path) {
  console.error("usage: extract-keys.mjs <index-<hash>.js>");
  process.exit(1);
}
const src = readFileSync(path, "utf8");

const URL_RE = /alphaprogression\.com(\/[\w\/-]+)/;

// Walks from an opening brace to its match, ignoring braces inside string literals
// (the string array is full of SVG path data and {{template}} markup).
function block(s, from) {
  let depth = 0, quote = null;
  for (let i = s.indexOf("{", from); i < s.length; i++) {
    const c = s[i];
    if (quote) {
      if (c === "\\") i++;
      else if (c === quote) quote = null;
    } else if (c === "'" || c === '"' || c === "`") quote = c;
    else if (c === "{") depth++;
    else if (c === "}" && --depth === 0) return s.slice(from, i + 1);
  }
  throw new Error("unbalanced braces");
}

function fn(name) {
  const at = src.indexOf(`function ${name}(`);
  if (at < 0) throw new Error(`missing function ${name}`);
  return block(src, at);
}

// function _0xA(index, key) { index = index - 0xOFFSET; var arr = _0xB(); ... }
const decoder = src.match(
  /function (_0x[0-9a-f]+)\((_0x[0-9a-f]+),_0x[0-9a-f]+\)\{\2=\2-0x[0-9a-f]+;var _0x[0-9a-f]+=(_0x[0-9a-f]+)\(\);/
);
if (!decoder) throw new Error("decoder function not found — bundle layout changed?");
const [, decName, , arrName] = decoder;

// Shuffles the array into its final order; the decoder returns garbage without it.
const rotation = src.match(
  new RegExp(`\\(function\\([^)]*\\)\\{[\\s\\S]*?\\}\\(${arrName},0x[0-9a-f]+\\)\\);`)
);
if (!rotation) throw new Error("rotation preamble not found — bundle layout changed?");

const decode = new Function(
  `${fn(arrName)}\n${fn(decName)}\n${rotation[0]}\nreturn ${decName};`
)();

// Calls look like post(URL, {'apiKey': lookup(0x…)}), so the endpoint is the last URL
// before the object literal — inline in some call sites, another lookup in others.
const keys = new Map();
for (const m of src.matchAll(/["']?apiKey["']?:_0x[0-9a-f]+\((0x[0-9a-f]+)\)/g)) {
  const key = decode(Number(m[1]));
  const lead = src.slice(Math.max(0, m.index - 200), m.index);

  const hits = [];
  for (const c of lead.matchAll(new RegExp(URL_RE, "g"))) hits.push([c.index, c[1]]);
  for (const c of lead.matchAll(/_0x[0-9a-f]+\((0x[0-9a-f]+)\)/g)) {
    const decoded = decode(Number(c[1]));
    const url = typeof decoded === "string" && decoded.match(URL_RE);
    if (url) hits.push([c.index, url[1]]);
  }
  if (!hits.length) continue; // not one of ours — Firebase config has an apiKey too

  if (!keys.has(key)) keys.set(key, new Set());
  keys.get(key).add(hits.sort((a, b) => a[0] - b[0]).at(-1)[1]);
}

const rows = [...keys]
  .map(([key, endpoints]) => [[...endpoints].sort().join(", "), key])
  .sort();
const width = Math.max(...rows.map(([endpoints]) => endpoints.length));
for (const [endpoints, key] of rows) console.log(endpoints.padEnd(width), key);

// The salt is not behind the string array — it's inline at the sha256 call.
const salt = src.match(/sha256\([^)]{0,80}'([0-9a-f]{64})'/);
console.log("\n" + "password salt".padEnd(width), salt ? salt[1] : "(not found)");
