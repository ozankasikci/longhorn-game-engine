#!/bin/bash

# Test script for drag-drop remote commands with diagnostic logging
SOCKET="/tmp/longhorn-editor.sock"

echo "========================================="
echo "Testing Scene Tree Drag-Drop via Remote Commands"
echo "========================================="

# Get all entities first
echo ""
echo "1. Getting current entities..."
RESPONSE=$(echo '{"action": "get_entities"}' | nc -U $SOCKET)
echo "$RESPONSE" | jq .

# Extract entity IDs
ENTITIES=$(echo "$RESPONSE" | jq -r '.data[].id')
ENTITY_ARRAY=($ENTITIES)

if [ ${#ENTITY_ARRAY[@]} -lt 2 ]; then
    echo "Error: Need at least 2 entities for testing"
    exit 1
fi

ENTITY1=${ENTITY_ARRAY[0]}
ENTITY2=${ENTITY_ARRAY[1]}
ENTITY1_NAME=$(echo "$RESPONSE" | jq -r ".data[] | select(.id==$ENTITY1) | .name")
ENTITY2_NAME=$(echo "$RESPONSE" | jq -r ".data[] | select(.id==$ENTITY2) | .name")

echo ""
echo "Entity 1: ID=$ENTITY1, Name=$ENTITY1_NAME"
echo "Entity 2: ID=$ENTITY2, Name=$ENTITY2_NAME"

# Test 1: Simulate dragging Entity2 onto Entity1
echo ""
echo "========================================="
echo "Test 1: Drag $ENTITY2_NAME ($ENTITY2) onto $ENTITY1_NAME ($ENTITY1)"
echo "========================================="
echo ""
echo "Sending simulate_scene_tree_drag command..."
echo "{\"action\": \"simulate_scene_tree_drag\", \"dragged_entity_id\": $ENTITY2, \"target_entity_id\": $ENTITY1}" | nc -U $SOCKET | jq .

echo ""
echo "Verifying hierarchy (checking entities)..."
echo '{"action": "get_entities"}' | nc -U $SOCKET | jq .

echo ""
echo "Checking logs for drag-drop activity..."
echo '{"action": "get_log_tail", "lines": 20}' | nc -U $SOCKET | jq '.data.entries[] | select(.message | contains("DND") or contains("reparent"))'

# Test 2: Drag entity to root
echo ""
echo "========================================="
echo "Test 2: Drag $ENTITY2_NAME ($ENTITY2) to root"
echo "========================================="
echo ""
echo "Sending simulate_scene_tree_drag_to_root command..."
echo "{\"action\": \"simulate_scene_tree_drag_to_root\", \"entity_id\": $ENTITY2}" | nc -U $SOCKET | jq .

echo ""
echo "Verifying hierarchy (should be root again)..."
echo '{"action": "get_entities"}' | nc -U $SOCKET | jq .

echo ""
echo "Checking logs for drag-drop activity..."
echo '{"action": "get_log_tail", "lines": 20}' | nc -U $SOCKET | jq '.data.entries[] | select(.message | contains("DND") or contains("reparent"))'

echo ""
echo "========================================="
echo "Tests complete!"
echo "========================================="
echo ""
echo "Summary:"
echo "  - Test 1: Simulated drag-drop via remote command (bypasses UI)"
echo "  - Test 2: Simulated drag-to-root via remote command (bypasses UI)"
echo ""
echo "These tests verify the backend hierarchy system works."
echo "If these pass but manual drag-drop fails, the issue is in the UI layer."
