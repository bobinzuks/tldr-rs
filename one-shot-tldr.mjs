import { readFile, writeFile } from 'fs/promises';
import { execSync } from 'child_process';

const KEY = process.env.DEEPSEEK_API_KEY;
const SOURCE = './source-python/tldr.py';

const py = await readFile(SOURCE, 'utf8');

const prompt = `Convert this Python CLI tool to Rust.

REQUIREMENTS:
- Use clap for CLI args
- Use reqwest::blocking for HTTP
- Use colored for terminal output  
- Cache pages at ~/.cache/tldr/
- Commands: tldr <page>, tldr --update, tldr --list
- No async needed, blocking is fine
- Single src/main.rs file

PYTHON SOURCE:
${py.slice(0, 3000)}

Output ONLY a rust code block for src/main.rs`;

const resp = await fetch('https://api.deepseek.com/v1/chat/completions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + KEY },
  body: JSON.stringify({ model: 'deepseek-chat', messages: [{ role: 'user', content: prompt }], max_tokens: 3000 })
});

const data = await resp.json();
const content = data.choices?.[0]?.message?.content || '';
const code = content.match(/```rust\n([\s\S]*?)```/)?.[1] || content;

await writeFile('src/main.rs', code);
console.log('Written src/main.rs (' + code.length + ' chars)');
console.log('Building...');

try { execSync('cargo build --release', { stdio: 'inherit' }); }
catch {}
