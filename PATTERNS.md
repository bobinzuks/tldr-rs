# tldr Client — Pattern Analysis (Language-Neutral Mappings)

This document extracts implementation patterns from the Python client and expresses them in language-neutral terms suitable for any implementation.

## 1. CLI Argument Parsing

### Python Pattern
```python
parser = ArgumentParser(prog="tldr", usage="...")
parser.add_argument('-v', '--version', action='version', ...)
parser.add_argument('command', type=str, nargs='*')
options = parser.parse_args()
```

### Language-Neutral Pattern
```
CLI_PARSER:
  - Define program metadata (name, usage, description)
  - Define flags with short/long forms
  - Define positional arguments with cardinality
  - Parse argv into structured options object
  - Generate shell completions
```

### Key Behaviors
- Mutually exclusive flags (--short-options vs --long-options can combine)
- Optional positional arguments (command not required with --list, --update)
- Multi-value positional (command can be "git commit" → ["git", "commit"])
- Version string includes client and specification versions

---

## 2. Configuration via Environment

### Python Pattern
```python
PAGES_SOURCE = os.environ.get('TLDR_PAGES_SOURCE_LOCATION', default)
USE_NETWORK = int(os.environ.get('TLDR_NETWORK_ENABLED', '1')) > 0
```

### Language-Neutral Pattern
```
ENV_CONFIG:
  - Read string value with default fallback
  - Parse boolean as "0"/"1" integer string
  - Parse integer (hours for cache age)
  - Expand paths (~/ handling)
  - Chain multiple variables (TLDR_LANGUAGE → LANGUAGE → LANG)
```

### Key Behaviors
- Defaults are hardcoded constants
- Boolean environment uses "0"/"1" string convention
- Paths support ~ expansion
- Invalid values use defaults (no error)

---

## 3. Path Resolution (XDG Compliance)

### Python Pattern
```python
def get_cache_dir() -> Path:
    if os.environ.get('XDG_CACHE_HOME', False):
        return Path(os.environ.get('XDG_CACHE_HOME')) / 'tldr'
    if os.environ.get('HOME', False):
        return Path(os.environ.get('HOME')) / '.cache' / 'tldr'
    return Path.home() / '.cache' / 'tldr'
```

### Language-Neutral Pattern
```
XDG_PATH_RESOLUTION:
  cache_dir:
    1. $XDG_CACHE_HOME/tldr if XDG_CACHE_HOME set
    2. $HOME/.cache/tldr if HOME set
    3. ~/.cache/tldr fallback

  system_data_dir:
    1. First existing $XDG_DATA_DIRS entry with /tldr subdirectory
    2. /usr/share/tldr fallback
```

### Key Behaviors
- Environment variables take precedence
- Create directories on write operations
- System cache is read-only
- Path separators are platform-appropriate

---

## 4. Multi-Level Fallback Resolution

### Python Pattern
```python
for platform in platforms:
    for language in languages:
        try:
            return get_page_for_platform(command, platform, language)
        except CacheNotExist:
            continue
```

### Language-Neutral Pattern
```
FALLBACK_CHAIN:
  sources: [user_cache, system_cache, network]
  platforms: [current_platform, common, ...others]
  languages: [preferred, ..., en]

  for each source:
    for each platform:
      for each language:
        try fetch
        if success: return
        if not_found: continue
        if error: store error, continue

  if any_results: return first
  if stored_error: raise error
  return not_found
```

### Key Behaviors
- Short-circuit on first success
- HTTP 404 means try next, not error
- Non-404 errors are stored and raised if all else fails
- Results ordered by priority (platform, then language)

---

## 5. HTTP Request Pattern

### Python Pattern
```python
data = urlopen(
    Request(url, headers={'User-Agent': 'tldr-python-client'}),
    timeout=10,
    context=ssl_context
).read()
```

### Language-Neutral Pattern
```
HTTP_FETCH:
  - Set User-Agent header for identification
  - Timeout: 10 seconds
  - TLS context: system default or custom CA or insecure
  - Read entire response body to bytes
  - Handle URLError, HTTPError separately
```

### Key Behaviors
- Identify as tldr client via User-Agent
- Support custom CA certificate path
- Support insecure mode (skip verification)
- Timeout prevents indefinite hang
- Response is raw bytes (not decoded)

---

## 6. ZIP Archive Extraction

### Python Pattern
```python
zipfile = ZipFile(BytesIO(response.read()))
for entry in zipfile.namelist():
    match = pattern.match(entry)
    if match:
        content = zipfile.read(entry)
```

### Language-Neutral Pattern
```
ZIP_EXTRACT:
  - Download archive to memory (BytesIO equivalent)
  - Iterate entry names
  - Filter entries by path pattern
  - Extract matching entries
  - Parse path for platform/command names
```

### Key Behaviors
- Archive URL: `tldr-pages.{lang}.zip`
- Entry pattern: `pages/platform/command.md`
- Extract to cache directory structure
- Report count per language

---

## 7. Cache Freshness Check

### Python Pattern
```python
last_modified = datetime.fromtimestamp(path.stat().st_mtime)
hours_passed = (datetime.now() - last_modified).total_seconds() / 3600
return hours_passed <= MAX_CACHE_AGE
```

### Language-Neutral Pattern
```
CACHE_FRESHNESS:
  - Read file modification timestamp
  - Calculate hours since modification
  - Compare against MAX_CACHE_AGE (default: 168 hours / 1 week)
  - Handle file-not-found as not-fresh
```

### Key Behaviors
- Uses filesystem mtime, not separate metadata
- Age measured in hours
- Configurable via environment
- Missing file = not fresh

---

## 8. Markdown Parsing (tldr Format)

### Python Pattern
```python
for line in page:
    line = line.decode('utf-8')
    if line[0] == '#':     # Command name
    elif line[0] == '>':   # Description
    elif line[0] == '-':   # Example text
    elif line[0] == '`':   # Command template
```

### Language-Neutral Pattern
```
TLDR_MARKDOWN_PARSE:
  line_types:
    '#' prefix  → command_name (strip '# ')
    '>' prefix  → description (strip '>', '<')
    '-' prefix  → example_text (may contain `backticks`)
    '`' prefix  → command_template (strip surrounding backticks)
    empty       → skip

  placeholder_patterns:
    {{description}}           → parameter placeholder
    {{[-s|--short]}}          → option alternatives
    \{\{ and \}\}             → escaped braces (literal)
```

### Key Behaviors
- Input is bytes, decode to UTF-8
- Leading character determines line type
- Backticks in example text rendered specially
- Placeholders use double braces
- Option alternatives separated by pipe

---

## 9. ANSI Color Output

### Python Pattern
```python
from termcolor import colored
line = colored(text, 'green', attrs=['bold'])
sys.stdout.buffer.write(line.encode('utf-8'))
```

### Language-Neutral Pattern
```
ANSI_OUTPUT:
  elements:
    command_name  → bold (customizable)
    description   → default (customizable)
    example_text  → green (customizable)
    command       → red (customizable)
    parameter     → default (customizable)
    inline_code   → yellow + italics (fixed)

  escapes:
    italics_start → \x1B[3m
    italics_end   → \x1B[23m

  output:
    - Write to stdout buffer (not print)
    - Encode as UTF-8
    - Leading spaces for indentation (2 chars default)
```

### Key Behaviors
- Colors configurable via TLDR_COLOR_* environment
- Windows requires colorama initialization
- Plain mode bypasses all styling
- Buffer write for Unicode safety

---

## 10. Option Length Alternatives

### Python Pattern
```python
if display_option_length == "short":
    line = re.sub(r'{{\[([^|]+)\|[^|]+?\]}}', r'\1', line)
elif display_option_length == "long":
    line = re.sub(r'{{\[[^|]+\|([^|]+?)\]}}', r'\1', line)
```

### Language-Neutral Pattern
```
OPTION_EXTRACTION:
  pattern: {{[-s|--short]}} or {{[short|long]}}

  display_mode:
    short → extract first alternative (before |)
    long  → extract second alternative (after |)
    both  → keep entire placeholder
```

### Key Behaviors
- Regex-based extraction
- First group = short form
- Second group = long form
- Affects only `[x|y]` patterns inside `{{}}`

---

## 11. Error Message Patterns

### Python Pattern
```python
sys.exit((
    "`{cmd}` documentation is not available.\n"
    "If you want to contribute it, feel free to"
    " send a pull request to: https://github.com/tldr-pages/tldr"
).format(cmd=command))
```

### Language-Neutral Pattern
```
ERROR_MESSAGES:
  not_found:
    - Include command name in backticks
    - Provide contribution URL
    - Exit with non-zero code

  network_error:
    - Include error details
    - Exit with non-zero code

  warning (not error):
    - Platform mismatch: "showing page from platform X"
    - Color warning text yellow
```

### Key Behaviors
- Contribution invitation on not-found
- Warnings don't cause non-zero exit
- Errors written to stderr
- Warnings written to stdout with color

---

## 12. Keyboard Interrupt Handling

### Python Pattern
```python
def cli():
    try:
        main()
    except KeyboardInterrupt:
        print("\nExited on keyboard interrupt.")
```

### Language-Neutral Pattern
```
SIGNAL_HANDLING:
  - Catch interrupt signal (SIGINT / Ctrl+C)
  - Print clean exit message
  - Exit with zero code (graceful)
```

### Key Behaviors
- Top-level wrapper around main
- Newline prefix for clean formatting
- Not an error condition

---

## 13. Locale Chain Resolution

### Python Pattern
```python
tldr_language = get_language_code(os.environ.get('TLDR_LANGUAGE', ''))
languages = os.environ.get('LANGUAGE', '').split(':')
languages = list(map(get_language_code, filter(...)))
if tldr_language:
    languages.remove(tldr_language)
    languages.insert(0, tldr_language)
if 'en' not in languages:
    languages.append('en')
```

### Language-Neutral Pattern
```
LOCALE_CHAIN:
  1. Parse TLDR_LANGUAGE → insert at front
  2. Parse LANGUAGE (colon-separated) → append each
  3. Parse LANG → append if not present
  4. Ensure 'en' at end
  5. Remove duplicates (preserve order)
  6. Normalize codes (strip encoding, map pt→pt_PT)
```

### Key Behaviors
- TLDR_LANGUAGE has highest priority
- English always present as fallback
- Regional variants preserved (pt_BR ≠ pt_PT)
- "C" and "POSIX" ignored

---

## 14. Command Name Normalization

### Python Pattern
```python
command = '-'.join(options.command).lower()
```

### Language-Neutral Pattern
```
COMMAND_NORMALIZE:
  - Join multi-word input with hyphens
  - Lowercase entire string
  - Example: ["Git", "Commit"] → "git-commit"
```

### Key Behaviors
- Happens before lookup
- Supports "git commit" style input
- Case insensitive matching

---

## 15. Search Pattern

### Python Pattern
```python
search_term = options.search.lower()
for command in commands:
    if search_term in command.lower():
        similar_commands.append(command)
```

### Language-Neutral Pattern
```
COMMAND_SEARCH:
  - Require cached commands (prompt update if empty)
  - Case-insensitive substring match
  - Return all matching command names
  - Report "no matches" if empty result
```

### Key Behaviors
- Operates on cache only (no network)
- Case-insensitive
- Substring, not exact match
- Exit 1 if no matches

---

## 16. File Rendering Pattern

### Python Pattern
```python
if file_path.exists():
    with file_path.open(encoding='utf-8') as f:
        output(f.read().encode('utf-8').splitlines(), ...)
```

### Language-Neutral Pattern
```
LOCAL_RENDER:
  - Verify file exists
  - Read as UTF-8 text
  - Encode to bytes (for consistency with network pages)
  - Split into lines
  - Pass to same render pipeline as fetched pages
```

### Key Behaviors
- Same rendering as remote pages
- Supports plain mode
- Reports file error if not found
