# Docker Manager Context Menu System Design Specification

## 1. Overview
Implement a comprehensive, context-aware menu system that follows the macOS native aesthetic. The system will provide tailored actions for containers, Compose projects, and global workspace areas.

## 2. Visual Design (macOS Native Style)
- **Container**: Floating panel with `10px` corner radius.
- **Material**: `backdrop-filter: blur(20px)` with semi-transparent background (`rgba(255, 255, 255, 0.7)` for light, `rgba(30, 30, 30, 0.7)` for dark).
- **Shadow**: Soft, multi-layered shadow to provide depth.
- **Menu Items**:
    - Height: `32px`.
    - Corner Radius: `6px`.
    - Hover: Background turns to `#007AFF` (SF Blue) and text turns white.
    - Icons: Small, low-contrast icons (using `@vicons/ionicons5`) aligned to the left.
- **Separators**: `0.5px` subtle lines between functional groups.

## 3. Contextual Scenarios

### 3.1 Scenario A: Container Actions
- **Group 1: Lifecycle**: Start, Restart, Stop (Dynamic enable/disable based on status).
- **Group 2: Interaction**: 
    - Open Terminal (Submenu: As User, As Root).
    - View Logs.
- **Group 3: Management**: Rename, Export Config, Delete (with confirmation).
- **Group 4: Info**: Copy ID, Copy Image Name, Inspect Data.

### 3.2 Scenario B: Compose Project Actions
- **Group 1: Orchestration**: Up, Down, Restart, Build.
- **Group 2: File System**: Open in Finder, Edit `docker-compose.yml`.
- **Group 3: Maintenance**: Prune Unused Resources.

### 3.3 Scenario C: Global Background Actions
- **Group 1: Refresh**: Reload all projects/containers.
- **Group 2: Creation**: New Container (Wizard), Import Compose Project.
- **Group 3: View**: Toggle Compact Mode, Zoom In/Out.

## 4. Technical Architecture
- **Base Component**: Custom-styled `n-dropdown` from Naive UI to handle positioning and z-index.
- **State Integration**: Menu item availability (`disabled` property) tied to the `store` state of the target object.
- **Boundary Detection**: Ensure menus flip if they would overflow the viewport edges.
- **Hook-based API**: Provide a `useContextMenu` hook to trigger menus from any component easily.

## 5. Implementation Roadmap (Task 1 of 4)
1.  **Style Overrides**: Define the macOS-style CSS for `n-dropdown`.
2.  **Logic Hook**: Create `useContextMenu.ts` to manage menu state and options generation.
3.  **Scene Integration**:
    - Bind right-click events in `ComposeProjectList.vue`.
    - Bind right-click events in `ContainerDetail.vue` (header).
    - Bind right-click events to the empty area of `Compose.vue`.
4.  **Action Handlers**: Connect menu keys to store actions (restart, stop, etc.).
