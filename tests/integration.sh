#!/bin/bash
set -euo pipefail

AHOY="${1:?Usage: integration.sh <path-to-ahoy-binary>}"
PASS=0
FAIL=0

pass() { PASS=$((PASS + 1)); echo "  PASS: $1"; }
fail() { FAIL=$((FAIL + 1)); echo "  FAIL: $1 — $2"; }

echo "Running ahoy integration tests..."
echo ""

# 1. Direct message
if "$AHOY" "Hello" -t "Test" 2>/dev/null; then
    pass "direct message"
else
    fail "direct message" "non-zero exit"
fi

# 2. JSON input
if "$AHOY" --json '{"title":"T","body":"B"}' 2>/dev/null; then
    pass "json input"
else
    fail "json input" "non-zero exit"
fi

# 3. --from-claude with tool permission
TOOL_JSON='{"cwd":"/Users/test/myproject","tool_name":"Bash","tool_input":{"command":"npm install"}}'
if echo "$TOOL_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude tool permission"
else
    fail "from-claude tool permission" "non-zero exit"
fi

# 4. --from-claude with transcript
TMPDIR_TESTS=$(mktemp -d)
trap 'rm -rf "$TMPDIR_TESTS"' EXIT
TRANSCRIPT="$TMPDIR_TESTS/transcript.jsonl"
echo '{"type":"user","message":{"content":"Deploy to production"}}' > "$TRANSCRIPT"
TRANSCRIPT_JSON="{\"cwd\":\"/Users/test/myproject\",\"transcript_path\":\"$TRANSCRIPT\"}"
if echo "$TRANSCRIPT_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude with transcript"
else
    fail "from-claude with transcript" "non-zero exit"
fi

# 5. --from-claude empty stdin
if echo -n "" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude empty stdin"
else
    fail "from-claude empty stdin" "non-zero exit"
fi

# 6. --from-claude invalid JSON
if echo "not json" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    fail "from-claude invalid json" "should have exited non-zero"
else
    pass "from-claude invalid json rejects"
fi

# 7. No message and no flags
if "$AHOY" 2>/dev/null; then
    fail "no args" "should have exited non-zero"
else
    pass "no args rejects"
fi

# 8. --help
HELP_OUTPUT=$("$AHOY" --help 2>&1 || true)
if echo "$HELP_OUTPUT" | grep -q "Usage"; then
    pass "--help shows usage"
else
    fail "--help" "output does not contain 'Usage'"
fi

# 9. Trailing slash in cwd
TRAILING_JSON='{"cwd":"/foo/bar/"}'
if echo "$TRAILING_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "trailing slash cwd doesn't crash"
else
    fail "trailing slash cwd" "non-zero exit"
fi

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
