# Agents Directory - Master Index

Organization system for agent workflows, documentation, and knowledge management.

> **📁 PROJECT STRUCTURE:** This is a graph visualization application built with egui/eframe:
> - `src/` - Main application source code
> - `src/app.rs` - Main application state and UI logic
> - `src/vis/` - Graph visualization components
> - `src/read/` - Graph reading context

## Directory Structure

```
agents/
├── guides/           # How-to guides and troubleshooting patterns
├── plans/            # Task plans (before execution)
├── implemented/      # Completed feature documentation
├── bug-reports/      # Known issues and problem analyses
├── analysis/         # Algorithm analysis and comparisons
└── tmp/              # Temporary analysis files (never commit)
```

## File Naming Convention (CRITICAL)

**All agent-generated files MUST include a timestamp prefix for chronological ordering:**

- **Format:** `YYYYMMDD_<FILENAME>.md` (e.g., `20260131_FEATURE_NAME.md`)
- **Benefits:**
  - Files sorted newest-to-oldest alphabetically (descending date order)
  - File age immediately visible without checking git history
  - Easy tracking of document evolution over time
  - Prevents filename collisions across time periods
  
**Examples:**
- ✅ `20260131_EGUI_INTEGRATION_GUIDE.md`
- ✅ `20260131_PLAN_ASYNC_GRAPH_LOADING.md`
- ✅ `20260131_VISUALIZATION_REFACTOR_COMPLETE.md`
- ❌ `EGUI_INTEGRATION_GUIDE.md` (missing timestamp)

**When to use:**
- Always for new files in `guides/`, `plans/`, `implemented/`, `bug-reports/`, `analysis/`
- Not required for `INDEX.md` files (special case)
- Not required for `tmp/` files (temporary, not committed)

---

## When to Use Each Directory

### `agents/guides/` 📚
**Purpose:** Persistent how-to guides and troubleshooting patterns

**What goes here:**
- Pattern guides (how to do X correctly)
- Common mistakes and fixes
- Migration checklists
- API usage examples
- Troubleshooting workflows

**When to add:**
- After solving a confusing problem
- When documenting a pattern that will recur
- After user clarifies unclear behavior
- When establishing best practices

**Format:** `YYYYMMDD_<TOPIC>_GUIDE.md`

**Index:** `agents/guides/INDEX.md` (tag-based search)

**⚠️ REQUIRED:** Add entry to INDEX.md with summary, tags, what it solves, and confidence:
- 🟢 High - Verified, current, complete
- 🟡 Medium - Mostly accurate, may have gaps
- 🔴 Low - Outdated or incomplete

---

### `agents/plans/` 📋
**Purpose:** Task plans before implementation (research phase)

**What goes here:**
- Multi-file refactoring plans
- Large feature implementation strategies
- Architecture change proposals

**When to add:** >5 files affected, >100 lines changed, or unclear scope

**Workflow:**
1. Create `YYYYMMDD_PLAN_<task_name>.md` using template
2. Gather ALL context before planning
3. Document: Objective, Context, Analysis, Steps, Risks, Validation
4. Execute in separate session (fresh context)
5. Create summary in `agents/implemented/` + update INDEX.md
6. Archive plan (rename to `YYYYMMDD_PLAN_<task>_DONE.md`) or delete if obsolete

**Format:** `YYYYMMDD_PLAN_<task_name>.md`

**Template:** `agents/plans/PLAN_TEMPLATE.md`

---

### `agents/implemented/` ✅
**Purpose:** Completed feature documentation

**What goes here:**
- Feature completion summaries
- Refactoring results
- Migration completions
- Enhancement documentation

**When to add:** After completing a significant feature or refactoring

**Format:** `YYYYMMDD_<FEATURE_NAME>_COMPLETE.md`

**Index:** `agents/implemented/INDEX.md`

---

### `agents/bug-reports/` 🐛
**Purpose:** Known issues and problem analyses

**What goes here:**
- Bug reports with root cause analysis
- Problem investigations
- Workarounds and temporary fixes

**When to add:** When identifying a bug that needs tracking or investigation

**Format:** `YYYYMMDD_BUG_<issue_name>.md`

**Index:** `agents/bug-reports/INDEX.md`

---

### `agents/analysis/` 🔬
**Purpose:** Algorithm analysis and architectural decisions

**What goes here:**
- Algorithm comparisons
- Architecture analysis documents
- Design decision rationale
- Performance analysis

**When to add:** When making significant architectural decisions

**Format:** `YYYYMMDD_<TOPIC>_ANALYSIS.md`

**Index:** `agents/analysis/INDEX.md`

---

### `agents/tmp/` 📝
**Purpose:** Temporary working files

**What goes here:**
- Scratch files during investigation
- Temporary notes
- Work-in-progress analysis

**⚠️ NEVER COMMIT:** Add to `.gitignore`

---

## Cross-References

- **Main README:** `../README.md` - Project overview
- **Context Engine:** `../../context-engine/` - Graph context framework (submodule)
- **Egui:** `../../egui/` - UI framework (submodule)
