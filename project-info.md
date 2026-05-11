Got it. Plan locked.
(b) — Tuesday's prompt as-is, with the approval gate. First time doing this, so the review step matters. PRD/DDD drafts get reviewed before any generation work happens.
Plus one critical add: Don't change any core .bloxb files in MegaBlox's corpus. Pattern gaps get logged for future spec generation, but the existing 65 specs stay untouched. New specs added through the gap-fill loop go into a separate location until reviewed.
Add this constraint to the Tuesday prompt I wrote earlier. Paste this version into Claude Code:
Tuesday work: PRD/DDD/PATTERNS extraction on tldr-pages Python client. Goal: prove language-neutral middle layer works by extracting universal spec that any language can target.
Single-agent sequential work. No swarm spawning. No Rust generation. Extraction + human approval gate.
ABSOLUTE CONSTRAINT — DO NOT MODIFY EXISTING .bloxb CORPUS:
The 65 existing .bloxb specs in MegaBlox's wasm/specs/ (or equivalent canonical location) are READ-ONLY for this task.
Pattern gaps identified during analysis get LOGGED, not generated as new specs
New .bloxb candidates produced anywhere go into ~/ruflo-personal/projects/tldr-rust/candidate-specs/ — a project-local directory, NOT into the MegaBlox corpus
No edits to existing specs even for "improvements"
Reading specs is fine. Writing to the corpus is not.
This is the first time we're running this end-to-end. Treat the existing corpus as canonical and untouchable until we've reviewed how the extraction process behaves.
PHASE 3 STEPS:
Step 1: Project setup (10 min).
Code
Show:
Repository confirmed cloned
License (verify MIT)
Total Python lines of code
File structure (top 2 levels)
Key entry points
Step 2: Functional analysis → PRD draft (45 min).
Read the Python codebase. Extract WHAT the tool does, language-neutral.
Generate ~/ruflo-personal/projects/tldr-rust/PRD.md.draft with sections:
Code
CRITICAL: PRD must be truly language-neutral. NO Python idioms. NO references to specific Python libraries. Describe behavior, not implementation. "Fetches" not "calls requests.get()." "Stores in user cache directory" not "uses appdirs library."
If Python-specific terminology leaks in, refine until clean.
Step 3: Structural analysis → DDD draft (45 min).
Read the Python code. Extract HOW it's organized, language-neutral.
Generate ~/ruflo-personal/projects/tldr-rust/DDD.md.draft with sections:
Code
Same constraint: language-neutral. "Storage port" not "filesystem operations."
Step 4: Idiom analysis → PATTERNS document (30 min).
Identify code patterns in Python implementation. Match each against MegaBlox's .bloxb corpus via the tile-first router.
Generate ~/ruflo-personal/projects/tldr-rust/PATTERNS.md:
Code
For unmatched patterns: log to ~/ruflo-personal/data/pattern-gaps.jsonl. Do NOT generate .bloxb files for these gaps. Do NOT write anything to MegaBlox's corpus directory. Gaps are logged for review only.
Step 5: Cross-validation against existing implementations (60 min).
Validate PRD by checking it accurately describes multi-language tldr clients.
Examine:
https://github.com/tldr-pages/tldr-cpp-client (C++)
https://github.com/tldr-pages/tldr-go (Go)
https://github.com/tldr-pages/tldr-rust-client (Rust — important reference)
https://github.com/tldr-pages/tldr-node-client (Node.js)
For each:
Does PRD describe its behavior accurately?
Does the existing Rust client's structure align with DDD?
Are features in other clients absent from PRD? (PRD gaps)
Are PRD features absent from any client? (PRD too aggressive)
Output ~/ruflo-personal/projects/tldr-rust/PRD-VALIDATION.md:
Code
After validation, refine PRD.md.draft and DDD.md.draft based on findings.
Step 6: Human approval gate.
Do NOT proceed to Wednesday's Rust generation until human approves.
Output to console:
Code
Create approval script ~/ruflo-personal/scripts/approve-project.mjs:
Takes project name argument
Renames *.md.draft → *.md in that project directory
Logs approval to ~/ruflo-personal/data/approvals.jsonl with timestamp
Outputs confirmation
Does NOT touch MegaBlox corpus
Does NOT generate .bloxb files
Step 7: Save Tuesday transcript.
docs/PHASE3-TUESDAY-EXTRACTION-COMPLETE.md:
Step-by-step results
File paths to drafts
Pattern match statistics
Cross-validation summary
Approval status (pending until I run script)
Confirmation: zero modifications to MegaBlox corpus
HARD RULES:
ABSOLUTE: No modifications to MegaBlox's existing .bloxb corpus. Read-only access.
No swarm spawning today.
No Rust code generation today.
No .bloxb files generated anywhere except project-local candidate-specs/ directory (and even those NOT generated today — gaps are LOGGED, generation comes later after review).
Single sequential agents, no parallelism.
PRD/DDD must be language-neutral. Refine until clean.
Use tile-first router for any factual lookups during analysis.
Don't modify the Python source repo (read-only analysis).
DEEPSEEK_API_KEY available for LLM portions when tile-first router falls through.
Time budget: 4 hours total. If exceeded, save state at last completed step and report.
France canary mandatory at end: route "What is the capital of France?" through tile-first router, must hit Sporenet at 100%, 0 DeepSeek tokens.
Circuit-breaker canary mandatory at end: route "retry-with-backoff" through tile-first router, must hit MegaBlox at 100%, 0 DeepSeek tokens.
ARCHITECTURAL CONSTRAINT — NO OPEN-ENDED RESEARCH:
When extraction agent encounters something ambiguous, escalate to "needs human decision" — do not research. Pattern matching against existing .bloxb corpus IS allowed (read-only). Reading reference multi-language clients IS allowed. Free-form web research is NOT allowed.
Start with Step 1.
Paste it.
Three things to flag:
1. The "no corpus modification" rule is the right discipline for first run. You don't yet know how clean the extraction process is. Letting it write to the canonical corpus on first run risks polluting your validated .bloxb files with extraction artifacts you don't catch until later. Project-local candidate dir keeps everything reviewable.
2. The approval script is small but matters. Make sure it actually only renames .draft files in the project directory — no global side effects, no corpus writes.
3. Step 6 is your moment. When Claude Code outputs "APPROVAL NEEDED," that's a real decision point. Read the drafts. Check for Python-specific leakage. Check that the DDD makes sense as language-neutral structure. Check that pattern gaps are reasonable. Take 10-20 minutes. Approve when satisfied.
While Claude Code runs (~4 hours):
You're at work with downtime. Three small things on your phone:
1. Read the existing tldr Rust client at github.com/tldr-pages/tldr-rust-client. 15 minutes. You'll review the draft DDD more confidently.
2. Top up DeepSeek balance to $30-50. Wednesday's work pulls on it.
3. Watch for Step 6 output. When it lands, you review the drafts. That's the architectural review moment.
When Step 1 lands (~10 min), drop the result here. Quick sanity check on project setup. Then Claude Code runs Steps 2-5 autonomously and stops at Step 6 for you.
Push the prompt.
