# Technical Design: World-Class UX

> Implementation approach for the premium UX with virtualized list, split preview, and smooth animations.

---

## Overview

The UX overhaul focuses on:
1. Virtualized list rendering for performance under load
2. Split preview pane for detailed item viewing
3. Smooth animations using CSS transitions and GPU acceleration
4. Visual treatment for sensitive items
5. Context chips for quick identification

---

## Affected Files

| File | Change |
|------|--------|
| `crates/applet/src/main.rs` | Major refactor for new UI structure |
| `crates/applet/src/list.rs` | New virtualized list component |
| `crates/applet/src/preview.rs` | New preview pane component |
| `crates/applet/src/chips.rs` | New context chip components |
| `crates/applet/src/empty_state.rs` | New empty state components |
| `crates/applet/src/shortcuts_overlay.rs` | New keyboard shortcuts overlay |
| `crates/applet/src/animation.rs` | Animation utilities |
| `crates/hypr-picker/src/main.rs` | Similar refactor for Hyprland picker |

---

## Implementation Details

### Virtualized List

```rust
// In applet/src/list.rs

use std::sync::Arc;

/// A virtualized list that only renders visible items
pub struct VirtualizedList<T> {
    items: Vec<T>,
    visible_range: Range<usize>,
    item_height: f32,
    scroll_offset: f32,
    container_height: f32,
}

impl<T: Clone> VirtualizedList<T> {
    /// Calculate which items should be visible given scroll offset
    fn calculate_visible_range(&self) -> Range<usize> {
        let start = (self.scroll_offset / self.item_height) as usize;
        let visible_count = (self.container_height / self.item_height).ceil() as usize + 2;
        let end = (start + visible_count).min(self.items.len());
        start..end
    }

    /// Get only the items that should be rendered
    fn get_visible_items(&self) -> Vec<&T> {
        self.items[self.visible_range.clone()].iter().collect()
    }

    /// Scroll to a specific index
    fn scroll_to_index(&mut self, index: usize) {
        self.scroll_offset = (index as f32) * self.item_height;
        self.visible_range = self.calculate_visible_range();
    }
}
```

### Split Layout

```rust
// In applet/src/layout.rs

use cosmic::widget::Column;

pub struct SplitLayout {
    pub list_pane: widget::Container,
    pub preview_pane: widget::Container,
    pub splitter: widget::Splitter,
}

impl SplitLayout {
    /// Default widths: 50% list, 50% preview
    /// User can drag splitter to resize
    pub fn new(list_pane: widget::Container, preview_pane: widget::Container) -> Self {
        let splitter = widget::Splitter::new()
            .direction(horizontal)
            .on_resize(|position| {
                // Update list and preview widths
            });

        Self {
            list_pane,
            preview_pane,
            splitter,
        }
    }
}
```

### Preview Pane

```rust
// In applet/src/preview.rs

pub enum PreviewContent {
    Text {
        content: String,
        syntax_highlighted: bool,
        language: Option<String>,
    },
    Image {
        path: PathBuf,
        width: u32,
        height: u32,
        size_bytes: u64,
    },
    Html {
        content: String,
        sandboxed: bool,
    },
    Files {
        files: Vec<FileInfo>,
    },
    Sensitive {
        masked_content: String,
        detection_reason: String,
    },
}

pub struct PreviewPane {
    content: PreviewContent,
    animation_state: AnimationState,
}
```

### Context Chips

```rust
// In applet/src/chips.rs

pub struct ContextChip {
    pub text: String,
    pub icon: Option<String>,
    pub color: ChipColor,
    pub on_click: Option<Box<dyn Fn()>>,
}

pub enum ChipColor {
    SourceApp,     // Blue
    Age,          // Gray
    ContentType,  // Green
    Sensitive,    // Red
    Pinned,       // Green
    Starred,      // Yellow
}

impl ContextChip {
    pub fn new(text: &str, color: ChipColor) -> Self {
        Self {
            text: text.to_string(),
            icon: None,
            color,
            on_click: None,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
}
```

### Animation Utilities

```rust
// In applet/src/animation.rs

use std::time::Duration;

/// Transitions with sensible defaults
pub fn fade_in(duration: Duration) -> cosmic::widget::Animator {
    cosmic::widget::Animator::new()
        .transition(cosmic::widget::Transition::Fade)
        .duration(duration)
        .easing(cosmic::widget::Easing::EaseOut)
}

/// Slide in from left
pub fn slide_in_left(duration: Duration) -> cosmic::widget::Animator {
    cosmic::widget::Animator::new()
        .transition(cosmic::widget::Transition::Slide {
            direction: Direction::LeftToRight,
        })
        .duration(duration)
        .easing(cosmic::widget::Easing::EaseOut)
}

/// Scale bounce for pin/star actions
pub fn bounce(duration: Duration) -> cosmic::widget::Animator {
    cosmic::widget::Animator::new()
        .transition(cosmic::widget::Transition::Scale {
            from: 1.2,
            to: 1.0,
        })
        .duration(duration)
        .easing(cosmic::widget::Easing::Spring)
}
```

---

## Performance Optimizations

### GPU Acceleration

```css
/* Use transform for animations (GPU accelerated) */
.list-item {
    will-change: transform, opacity;
}

.preview-content {
    will-change: transform;
}
```

### Lazy Loading

```rust
// Only load full content when item is selected
fn on_item_selected(item: &ClipboardItem) {
    match item.content_type {
        ContentType::Image => {
            // Load full image only when in preview
            load_full_image_async(item.image_path());
        }
        ContentType::Html => {
            // Parse and sandbox HTML only when in preview
            parse_html_sandboxed(item.content());
        }
        _ => {
            // Text content is already loaded
        }
    }
}
```

### List Item Recycling

```rust
// Reuse list item widgets instead of creating new ones
fn get_or_create_list_item(index: usize) -> &mut ListItem {
    if index < self.reused_items.len() {
        // Reuse existing item
        self.reused_items[index].reset();
    } else {
        // Create new item
        self.reused_items.push(ListItem::new());
    }
    &mut self.reused_items[index]
}
```

---

## State Management

### App State Changes

```rust
// In applet/src/main.rs

pub struct AppState {
    // ... existing fields ...

    // New fields for world-class UX
    pub selected_item_preview: Option<PreviewContent>,
    pub list_virtualization: VirtualizationState,
    pub animation_queue: Vec<Animation>,
    pub showing_shortcuts_overlay: bool,
}

struct VirtualizationState {
    pub scroll_offset: f32,
    pub visible_range: Range<usize>,
    pub item_height: f32,
}

struct Animation {
    pub target_element: String,
    pub animation_type: AnimationType,
    pub start_time: Instant,
    pub duration: Duration,
}
```

---

## Testing Strategy

1. Performance testing with 1000 items (verify 60fps)
2. Memory profiling with large history
3. Animation smoothness testing
4. Visual regression tests for sensitive item treatment
5. Empty state rendering tests

---

**Last Updated**: Phase 15