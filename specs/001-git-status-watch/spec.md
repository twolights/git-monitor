# Feature Specification: Git Status Watcher

**Feature Branch**: `001-git-status-watch`  
**Created**: 2026-06-28  
**Status**: Draft  
**Input**: User description: "CLI tool to watch git repository status in a specified directory with a real-time terminal UI showing the current branch, repository status (untracked files, staged changes, unstaged changes), and recent commits, displayed in a split view. First version is a simple, read-only implementation."

## Clarifications

### Session 2026-06-28

- Q: How should the split view be divided between status and recent commits? → A: Horizontal divider — status pane on top, recent commits on the bottom (stacked).
- Q: How many recent commits, and what detail per commit? → A: 10 most recent commits; each shows short hash + summary (first line) + relative date + author.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Monitor repository status in real time (Priority: P1)

A developer working in a git repository wants to keep an eye on the state of their working tree without repeatedly typing `git status`. They launch the tool, which opens a live terminal view that continuously reflects the current branch, untracked files, staged changes, and unstaged changes. As the developer edits, stages, or discards files in another terminal or editor, the view updates automatically to mirror the real state of the repository.

**Why this priority**: This is the core value of the tool — a continuously updated, glanceable view of working-tree state. Without it there is no product. It is the minimum viable slice that delivers standalone value.

**Independent Test**: Launch the tool inside a git repository, then create a new file, stage another file, and modify a tracked file from a separate shell. Confirm the live view reflects each change (untracked, staged, unstaged) within a short, perceptible delay, and shows the current branch name — all without restarting the tool.

**Acceptance Scenarios**:

1. **Given** a git repository with a clean working tree, **When** the user launches the tool, **Then** the view displays the current branch name and indicates there are no pending changes.
2. **Given** the tool is running, **When** the user creates a new untracked file, **Then** the file appears in the view's untracked-files section within a short delay without user interaction.
3. **Given** the tool is running, **When** the user stages a modified file, **Then** the file moves from the unstaged/changes section to the staged ("changes to be committed") section in the view.
4. **Given** the tool is running, **When** the user discards or commits changes, **Then** the corresponding entries disappear from the relevant sections.

---

### User Story 2 - View recent commit history alongside status (Priority: P2)

A developer wants context about recent work while watching the repository. The tool presents a split view: one region shows the live working-tree status and the other shows a list of the most recent commits on the current branch, so the developer can see both the current state and the recent history at a glance.

**Why this priority**: Recent history adds important context (what was just committed, current position on the branch) and is a stated requirement, but the status view alone is usable without it, so it ranks below P1.

**Independent Test**: Launch the tool in a repository with several commits and confirm the commits region lists the most recent commits with identifying information (e.g., short hash and message). Make a new commit from another shell and confirm it appears at the top of the list.

**Acceptance Scenarios**:

1. **Given** a repository with commit history, **When** the user launches the tool, **Then** a list of the most recent commits is shown in its own region of the split view.
2. **Given** the tool is running, **When** a new commit is created on the current branch, **Then** the new commit appears at the top of the recent-commits list.
3. **Given** the terminal window, **When** the tool is displayed, **Then** the status region and the recent-commits region are visually separated in a split layout.

---

### User Story 3 - Watch a repository other than the current directory (Priority: P3)

A developer wants to monitor a repository that is not their current working directory (for example, a project in another folder). They launch the tool with an option specifying the target directory, and the tool watches that repository instead of the current one. When no directory is specified, the tool defaults to the current working directory.

**Why this priority**: Convenience that broadens applicability, but the default-to-current-directory behavior already covers the most common case, so directory selection is the lowest priority of the three.

**Independent Test**: Run the tool with a directory argument pointing at a different git repository and confirm the displayed branch, status, and commits correspond to that repository, not the current directory. Run the tool with no argument and confirm it watches the current directory.

**Acceptance Scenarios**:

1. **Given** the user runs the tool with no directory option, **When** the tool starts, **Then** it watches the git repository in the current working directory.
2. **Given** the user runs the tool with a directory option pointing to a valid git repository, **When** the tool starts, **Then** it watches that repository.
3. **Given** the user runs the tool with a directory option pointing to a path that is not a git repository, **When** the tool starts, **Then** it reports a clear error and exits rather than displaying an empty or misleading view.

---

### Edge Cases

- What happens when the target directory is not a git repository (or has no commits yet)? The tool should report a clear message rather than crashing or showing a misleading empty view.
- How does the tool handle a repository with a very large number of changed or untracked files? The view should remain responsive and gracefully handle content that exceeds the visible area (e.g., truncation or scrolling indication).
- How does the tool handle a detached HEAD state (no branch name)? The view should indicate the detached/HEAD state rather than showing a blank branch.
- How does the tool behave when the terminal window is very small or is resized while running? The layout should adapt or degrade gracefully without corrupting the display.
- What happens if the repository is deleted, moved, or becomes inaccessible while the tool is running? The tool should surface an error state rather than displaying stale data silently.
- How does the tool handle a repository in the middle of an operation such as a merge or rebase? At minimum it should not crash; reflecting such states is a nice-to-have.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tool MUST run as a command-line program that opens a terminal-based view when launched.
- **FR-002**: The tool MUST watch the git repository in the current working directory by default when no target directory is provided.
- **FR-003**: The tool MUST provide an option to specify a different directory to watch instead of the current working directory.
- **FR-004**: The tool MUST display the name of the current branch of the watched repository.
- **FR-005**: The tool MUST display the repository status, including untracked files, changes staged for commit, and changes not staged for commit, in clearly distinguished groups.
- **FR-006**: The tool MUST display a list of the 10 most recent commits on the current branch; each entry MUST show the short commit hash, the commit summary (first line of the message), a relative date (e.g., "2 hours ago"), and the author. When fewer than 10 commits exist, all available commits are shown.
- **FR-007**: The tool MUST present the status information and the recent-commits information in a split view with a horizontal divider — the status region on top and the recent-commits region on the bottom — both visible at the same time.
- **FR-008**: The tool MUST continuously monitor the repository and update the displayed status and commit information to reflect changes, without requiring the user to manually refresh or restart.
- **FR-009**: The tool MUST operate in read-only mode in this version: it MUST NOT modify the repository, its files, or its history.
- **FR-010**: The tool MUST report a clear error and exit when the target path is not a valid git repository.
- **FR-011**: The tool MUST provide a way for the user to exit the live view and return to the shell.
- **FR-012**: The tool MUST indicate when the working tree is clean (no untracked, staged, or unstaged changes).
- **FR-013**: The tool MUST handle terminal display gracefully when content exceeds the available space (e.g., truncate or otherwise avoid corrupting the layout).

### Key Entities *(include if feature involves data)*

- **Watched Repository**: The git repository being monitored, identified by its directory path; the source of all displayed information.
- **Branch**: The current branch (or detached HEAD state) of the watched repository; provides the branch-name display and the scope for recent commits.
- **Working-Tree Status**: The set of file-level states grouped as untracked files, staged changes, and unstaged changes at a given moment.
- **Commit**: A recent entry in the branch history, characterized by a short hash, a summary (first line of the message), a relative date, an author, and ordering by recency (most recent first, up to 10 shown).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can launch the tool and see the current branch, working-tree status, and recent commits for the current directory within 2 seconds of starting it.
- **SC-002**: After a change is made to the repository (file created, staged, modified, or committed), the displayed view reflects that change within 2 seconds without any user interaction.
- **SC-003**: A user can determine the complete working-tree state (untracked, staged, unstaged, or clean) and the current branch in a single glance, without scrolling, for a repository with up to 20 changed files.
- **SC-004**: A user can point the tool at any valid git repository on their system, or rely on the current-directory default, and see correct information for that repository in 100% of cases.
- **SC-005**: When pointed at an invalid or non-git path, the tool informs the user of the problem in 100% of cases instead of displaying empty or misleading data.
- **SC-006**: The tool never alters the watched repository — repository contents, index, and history are byte-for-byte identical before and after a session in 100% of cases.

## Assumptions

- The tool runs in an interactive terminal that supports a full-screen text UI; output redirection to a non-interactive context is out of scope for this version.
- "Recent commits" shows a fixed count of 10 (see Clarifications); making this count configurable is out of scope for this version.
- Real-time updating is achieved by polling the repository at a short interval (assumed sub-second to ~1 second) and/or reacting to filesystem changes; the exact mechanism is an implementation detail, but updates must feel near-immediate per SC-002.
- The watched repository is a standard local git repository; submodules, worktrees, and bare repositories are not specially handled in this version.
- Single-repository monitoring only; watching multiple repositories simultaneously is out of scope for v1.
- No interactive git operations (staging, committing, branching, etc.), configuration files, or persistent settings are included in this version (read-only, simple-first scope).
- The user has git available/installed in the environment; the tool reads standard git repository state.
