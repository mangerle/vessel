# Docker Manager UI Redesign Specification (Apple/macOS Style)

## 1. Project Overview
Refactor the current Docker Manager UI to achieve a "simple but not simplistic, elegant but not ugly" aesthetic, specifically following the **Apple / macOS Native App** style. This involves a shift from standard web-component-heavy design to a refined, layered card-based interface.

## 2. Core Aesthetic Principles
- **Style**: macOS "Layered Look".
- **Visual Hierarchy**: Defined by depth, shadows, and spacing rather than hard lines.
- **Interactions**: Smooth transitions, spring animations, and subtle hover effects.

## 3. Visual System
### 3.1 Color Palette
- **Backgrounds**:
  - Global Light: `#F5F5F7`
  - Global Dark: `#161617`
- **Cards**:
  - Light: `#FFFFFF`
  - Dark: `#1C1C1E`
- **Accents**:
  - Primary: `#007AFF` (SF Blue)
  - Success: `#28CD41` (Emerald Green)
  - Warning: `#FF9500` (Orange)
  - Error: `#FF3B30` (Coral Red)
  - Neutral/Stopped: `#8E8E93` (Stone Gray)

### 3.2 Materials & Shapes
- **Corner Radius**: `12px` for main cards, `8px` for inner elements.
- **Border**: `0.5px solid rgba(0, 0, 0, 0.1)` (Light) / `0.5px solid rgba(255, 255, 255, 0.1)` (Dark).
- **Shadow**: `box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05)`.
- **Blur**: `backdrop-filter: blur(20px)` for sidebar and modals.

## 4. Layout Structure (Three-Column Card Layout)

### 4.1 Sidebar (Navigation)
- **Width**: 200px (Fixed or Collapsible).
- **Background**: Transparent with Backdrop Blur.
- **Items**: Rounded selection blocks, SF Symbols-like icons.

### 4.2 List Column (Projects/Containers)
- **Structure**: A floating vertical card.
- **Header**: Integrated Search bar with subtle border.
- **List Items**: 
  - Height: `64px`.
  - Content: Title (Bold), Subtitle (Image/ID), Status Indicator (4px vertical bar on the left).

### 4.3 Detail Column (Main Workspace)
- **Structure**: The largest floating card.
- **Toolbar**: 
  - Top-aligned.
  - Left: Container Name & Status badge.
  - Right: Capsule-style Action Buttons (Restart, Stop, Term, etc.).
- **Navigation**: **Segmented Control** (Sliding pill) for switching between:
  - Overview
  - Logs
  - Terminal
  - Stats (Dashboard)

## 5. Key Components Detail
- **Segmented Control**: A sliding background block that follows the selection with a spring animation.
- **Zen Dashboard**: ECharts with `smooth: true`, area gradients, and hidden grid lines/axes.
- **Blurred Terminal**: Xterm.js inside a semi-transparent dark container with increased padding.

## 6. Implementation Roadmap
1.  **Base Layout Refactor**: Implement the three-column grid with the new background and card styles.
2.  **Component Styling**: Create a custom CSS/Theme override for Naive UI components.
3.  **Detailed View Overhaul**: Implement the Segmented Control and the new Toolbar.
4.  **Dashboard & Terminal Polish**: Apply "Zen" styling to charts and terminal.
5.  **Micro-interactions**: Add hover effects and transition animations.
