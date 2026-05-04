# Images, Networks, and Volumes UI Redesign Specification

## 1. Overview
Refactor the remaining core resource views (Images, Networks, Volumes) to align with the macOS-style three-column card layout. This redesign moves away from basic tables to a sophisticated "List + Detail" interaction model, providing resource-specific insights such as image layers, network topology, and volume usage.

## 2. Global UI Framework
- **Layout**: Three-column architecture.
    - **Sidebar**: App-wide navigation (existing).
    - **List Column**: Narrow floating card (`320px`) with a search/action header and vertical resource list.
    - **Detail Column**: Large floating card (`flex: 1`) with a toolbar and segmented control for tabbed content.
- **Aesthetics**: macOS "Layered Look" using `var(--macos-shadow)`, `12px` corner radius, and SF-style typography.

## 3. Resource Specific Designs

### 3.1 Images View
- **List Column**:
    - **Pull Header**: Integrated `n-auto-complete` for image pulling.
        - **Feature**: Real-time suggestions via `search_images` backend command.
        - **Visuals**: Show stars and "Official" badges in the dropdown.
    - **List Item**: Title is `Repository:Tag`. Subtitle is `Short ID` · `Size` · `Created Ago`.
- **Detail Column Tabs**:
    - **Overview**: Basic metadata (ID, Arch, OS, Created, Size).
    - **Layers**: A vertical timeline (`n-timeline`) showing `docker history` data (Command, Size per layer).
    - **Config**: List of environment variables, exposed ports, and the default CMD/Entrypoint.

### 3.2 Networks View
- **List Column**:
    - **Header**: Prune action and Refresh button.
    - **List Item**: Title is `Network Name`. Subtitle is `Driver (Scope)`.
- **Detail Column Tabs**:
    - **Overview**: IPAM configuration (Subnet, Gateway), ID, and Scope.
    - **Connected Containers**: A table listing containers attached to this network.
        - **Columns**: Container Name, IPv4 Address, IPv6 Address, MAC Address.
        - **Action**: "Disconnect" button per row.

### 3.3 Volumes View
- **List Column**:
    - **Header**: Prune action and Refresh button.
    - **List Item**: Title is `Volume Name`. Subtitle is `Driver`.
- **Detail Column Tabs**:
    - **Overview**: Displays the `Mountpoint`.
        - **Action**: "Open in File Explorer" button for local mounts.
    - **Users**: A list of containers currently mounting this volume, showing their mount destination (Source -> Target).

## 4. Backend Requirements (Rust)
- **`search_images(term)`**: New command to invoke Docker Search API.
- **`get_image_history(id)`**: Fetch layer information for the timeline.
- **`get_network_details(id)`**: Ensure connected container IPs/MACs are retrieved.
- **`open_volume_path(path)`**: Platform-specific command to open the host directory.

## 5. Implementation Roadmap
1. **Backend Extensions**: Implement missing Docker commands in `docker.rs`.
2. **Component Refactor**:
    - Migrate `Images.vue`, `Networks.vue`, and `Volumes.vue` to the three-column template.
    - Implement the `n-auto-complete` pull logic.
3. **Detail Modules**: Build the specialized tab components (Timeline, Container Table, Usage List).
4. **Interactions**: Add the "Open in Explorer" and "Disconnect Network" logic.
