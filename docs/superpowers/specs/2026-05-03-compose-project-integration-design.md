# Compose Project Integration & Editing Design Specification

## 1. Overview
Transform the current container-centric view into a project-aware management system. This involves a tree-like navigation structure and a dedicated project workspace featuring a YAML editor for `docker-compose.yml`.

## 2. UI/UX Design

### 2.1 Navigation (Left Column)
- **Structure**: Hierarchical Tree View.
    - **Level 1 (Project)**: Compose project name. Status icon indicates overall health (All Running, Partial, Stopped).
    - **Level 2 (Container)**: Individual containers belonging to the project.
- **Actions**: Clicking a Project node opens the Project Workspace; clicking a Container node opens the Container Detail.

### 2.2 Project Workspace (Right Column)
- **Dashboard Tab**:
    - Aggregated resource metrics (Total CPU, Mem, Net).
    - Quick Action Toolbar: `Up`, `Down`, `Restart`, `Build`.
- **Editor Tab**:
    - **Component**: Monaco Editor or CodeMirror for YAML editing.
    - **Controls**:
        - `Save`: Persists the file to disk.
        - `Save & Apply`: Persists the file and triggers `docker compose up -d`.
    - **Execution Console**: A slide-up or bottom panel showing real-time output from `docker compose` commands.

## 3. Backend Architecture (Rust)

### 3.1 File Operations
- `get_compose_config(project_id)`: Reads the `docker-compose.yml` from the `working_dir` stored in the database.
- `save_compose_config(project_id, content)`: Writes the updated content back to the file.

### 3.2 Command Execution Engine
- **Command**: `tokio::process::Command` to invoke `docker compose`.
- **Context**: Execution happens in the `working_dir` of the project.
- **Output Streaming**:
    - Capture `stdout` and `stderr` asynchronously.
    - Broadcast lines to the frontend via Tauri Events (`compose-cmd-output`).
- **Validation**: Before saving, run `docker compose config` to check for YAML syntax and structure errors.

## 4. Data Model Integration
- Leverage existing `compose_projects` table.
- Ensure `ContainerInfo` correctly maps to `ComposeProject` in the frontend store.

## 5. Implementation Roadmap (Task 2 of 4)
1.  **Backend Commands**: Implement file I/O and process execution in `docker.rs`.
2.  **Frontend Store**: Update `compose.ts` to handle project file content and command execution states.
3.  **UI Refactor**:
    - Replace `ComposeProjectList.vue` list with a tree structure.
    - Enhance `ContainerDetail.vue` to support "Project View" mode.
4.  **Editor Integration**: Add the editor component and the execution console.
5.  **Integration**: Connect right-click "Edit" action to the project workspace.
