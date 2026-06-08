# UI Flow: Advanced Filtering & Saved Searches

> User interaction flows and UI behavior for the advanced filtering system.

---

## Main Search Flow

```
[User opens picker]
        |
        v
[Search box focused] --> [Type text or filters]
        |                        |
        v                        v
[Show autocomplete] <-- [Detect filter pattern]
        |                        |
        v                        v
[Select suggestion or type more]
        |
        v
[Press Enter or click search icon]
        |
        v
[Execute search via IPC]
        |
        v
[Display results with active filter chips shown above]
```

---

## Chip UI Interaction

### Adding a Filter Chip

1. User types filter prefix (e.g., `type:`)
2. Autocomplete shows available values
3. User clicks a value or presses Tab
4. Chip appears in the search box: `type:text`
5. Chip is highlighted (active state)

### Removing a Filter Chip

1. User clicks on a chip (it highlights)
2. User presses Delete/Backspace
3. Chip is removed, search text remains

### Editing a Filter Chip

1. User clicks on a chip
2. Chip becomes editable
3. User changes value or presses Escape to cancel

---

## Filter Chip States

| State | Appearance | Behavior |
|-------|------------|----------|
| Default | Gray background | Click to select |
| Selected | Blue background, white text | Click to edit or remove |
| Hover | Lighter background | Shows tooltip with value |
| Error | Red border | Invalid filter value |

---

## Saved Search Flow

```
[User clicks "Saved" button in search box]
        |
        v
[Dropdown shows saved searches]
        |
        v
[User selects a saved search]
        |
        v
[Search box populated with query]
        |
        v
[Search executes automatically]
```

### Saving a Search

1. User types a search query
2. User clicks the "Save" icon (bookmark)
3. Dialog appears: "Save search as..."
4. User enters a name
5. User clicks "Save"
6. Search is saved and appears in dropdown

### Managing Saved Searches

1. User clicks "Saved" button
2. User right-clicks a saved search
3. Context menu shows: "Edit", "Rename", "Delete", "Move to top"
4. User selects an action

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `/` or `Ctrl+F` | Focus search box |
| `Enter` | Execute search |
| `Escape` | Clear search and close autocomplete |
| `Tab` | Accept autocomplete suggestion |
| `Backspace` | Remove last chip if at end |
| `Ctrl+S` | Open save dialog |
| `Ctrl+,` | Open saved searches dropdown |

---

## Filter Autocomplete

### type: filter
```
type:[text|image|html|files]
```
Shows content type options with icons.

### age: filter
```
age:[today|week|month|<number>s/m/h/d]
```
Shows relative time options.

### app: filter
```
app:[kitty|firefox|vscode|...]
```
Shows recently seen source apps, sorted by frequency.

### pinned: / sensitive: / starred: filters
```
pinned:[true|false]
sensitive:[true|false]
starred:[true|false]
```
Shows boolean options.

### in: filter (collection)
```
in:[prompts|deploy|db|...]
```
Shows available collection names.

---

## Empty States

### No Results
- Icon: magnifying glass with X
- Text: "No items match your search"
- Suggestions: "Try removing filters", "Search for something else"

### No Saved Searches
- Icon: bookmark outline
- Text: "No saved searches yet"
- Hint: "Press Ctrl+S to save your current search"

### No Search History
- Icon: clock outline
- Text: "No recent searches"
- Hint: "Your search history will appear here"

---

## Error States

### Invalid Filter
- Chip shows red border
- Tooltip shows: "Invalid value for type filter"
- Autocomplete shows valid options

### Daemon Not Running
- Search box disabled
- Icon: cloud with X
- Text: "Daemon not running"

---

**Last Updated**: Phase 15