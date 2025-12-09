#!/bin/bash

# Test script for hierarchy remote commands
SOCKET="/tmp/longhorn-editor.sock"

echo "Testing hierarchy commands..."

# Get all entities first
echo "1. Getting entities..."
echo '{"action": "get_entities"}' | nc -U $SOCKET | jq .

# Get entity IDs (assuming we have at least 2 entities)
ENTITIES=$(echo '{"action": "get_entities"}' | nc -U $SOCKET | jq -r '.data[].id')
ENTITY_ARRAY=($ENTITIES)

if [ ${#ENTITY_ARRAY[@]} -lt 2 ]; then
    echo "Error: Need at least 2 entities. Creating them..."
    echo '{"action": "create_entity", "name": "Parent"}' | nc -U $SOCKET | jq .
    echo '{"action": "create_entity", "name": "Child"}' | nc -U $SOCKET | jq .

    # Get entities again
    ENTITIES=$(echo '{"action": "get_entities"}' | nc -U $SOCKET | jq -r '.data[].id')
    ENTITY_ARRAY=($ENTITIES)
fi

PARENT_ID=${ENTITY_ARRAY[0]}
CHILD_ID=${ENTITY_ARRAY[1]}

echo "Parent ID: $PARENT_ID"
echo "Child ID: $CHILD_ID"

# Test 1: Set parent
echo ""
echo "2. Setting $CHILD_ID as child of $PARENT_ID..."
echo "{\"action\": \"set_entity_parent\", \"child_id\": $CHILD_ID, \"parent_id\": $PARENT_ID}" | nc -U $SOCKET | jq .

# Test 2: Verify hierarchy (check entities again to see structure)
echo ""
echo "3. Getting entities to verify hierarchy..."
echo '{"action": "get_entities"}' | nc -U $SOCKET | jq .

# Test 3: Clear parent
echo ""
echo "4. Clearing parent for $CHILD_ID..."
echo "{\"action\": \"clear_entity_parent\", \"child_id\": $CHILD_ID}" | nc -U $SOCKET | jq .

# Test 4: Verify cleared
echo ""
echo "5. Getting entities to verify parent cleared..."
echo '{"action": "get_entities"}' | nc -U $SOCKET | jq .

echo ""
echo "Tests complete!"
