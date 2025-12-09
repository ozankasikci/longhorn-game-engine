# Manual Test for Drag-and-Drop Reparenting

## Setup
1. Start the editor: `cargo run --bin longhorn-editor`
2. Ensure there are at least 2 entities in the scene (Player and Enemy exist by default)

## Test Case 1: Basic Drag and Drop
1. In the Scene Tree panel, click and hold on "Enemy" entity
2. Drag the mouse over "Player" entity
3. Release the mouse button

**Expected Result:**
- Visual highlight appears on "Player" row during drag hover
- After release, "Enemy" becomes a child of "Player"
- "Enemy" appears indented under "Player" in the tree
- Console log shows: "Reparented entity [enemy_id] to [player_id]"

## Test Case 2: Drag to Root Zone
1. Click and hold on the child entity (Enemy, if it's now a child of Player)
2. Drag to the bottom drop zone labeled "Drop here to make root entity"
3. Release

**Expected Result:**
- Entity becomes a root entity (no parent)
- Entity appears at top level (no indentation)
- Console log shows: "Cleared parent for entity [entity_id]"

## Test Case 3: Self-Drop Protection
1. Click and hold on "Player" entity
2. Drag and drop on "Player" itself
3. Release

**Expected Result:**
- Nothing happens (no reparenting)
- No error messages

## Verification via Remote Commands
After manual drag-and-drop, verify hierarchy using:
```bash
echo '{"action": "get_entities"}' | nc -U /tmp/longhorn-editor.sock | jq .
```

This should show the parent-child relationships established by drag-and-drop.

## Root Cause
The issue was that `dnd_release_payload()` was being called on `outer_response` (the horizontal container) instead of on `label_response` (the inner label that had the drag payload). The fix aligns with the design document by checking drops on the `.inner` response.
