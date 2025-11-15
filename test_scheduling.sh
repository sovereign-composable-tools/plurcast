#!/bin/bash
set -e

echo "=== Testing Plurcast Scheduling ==="
echo ""

# Schedule a post for 30 seconds from now
echo "1. Scheduling a test post for 30 seconds from now..."
POST_OUTPUT=$(./target/release/plur-post "Scheduled test at $(date)" --schedule "30s" --platform nostr)
echo "   Output: $POST_OUTPUT"
POST_ID=$(echo "$POST_OUTPUT" | cut -d':' -f2)
echo "   Post ID: $POST_ID"
echo ""

# List scheduled posts
echo "2. Listing scheduled posts..."
./target/release/plur-queue list
echo ""

# Show stats
echo "3. Queue statistics..."
./target/release/plur-queue stats
echo ""

echo "4. Now start plur-send in another terminal:"
echo "   ./target/release/plur-send --verbose"
echo ""
echo "   Wait 30 seconds and watch it post!"
