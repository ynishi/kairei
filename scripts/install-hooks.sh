#\!/bin/bash

# Script to install Git hooks for the kairei project
# This script installs:
# - pre-commit hook that runs make fmt (warning only)
# - pre-push hook that runs make test (blocking)

HOOK_DIR=$(git rev-parse --git-dir)/hooks
SCRIPT_DIR=$(dirname "$0")
PROJECT_ROOT=$(git rev-parse --show-toplevel)

echo "Installing Git hooks..."

# Create pre-commit hook
cat > "$HOOK_DIR/pre-commit" << 'HOOK_EOF'
#\!/bin/bash

echo "Running pre-commit hooks..."

# Run make fmt
echo "Running make fmt..."
make fmt
FMT_EXIT_CODE=$?

if [ $FMT_EXIT_CODE -ne 0 ]; then
  echo "Warning: make fmt found formatting issues. The commit will proceed, but please fix before pushing."
  echo "You can fix formatting with: make fmt"
  # Note: Not exiting with error to allow commit to proceed
fi

echo "Pre-commit hooks completed."
exit 0

# To bypass this hook: git commit --no-verify
HOOK_EOF

# Create pre-push hook
cat > "$HOOK_DIR/pre-push" << 'HOOK_EOF'
#\!/bin/bash

echo "Running pre-push hooks..."

# Run make test
echo "Running make test..."
make test
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
  echo "Error: make test failed. Please fix the test failures before pushing."
  exit 1
fi

echo "All pre-push hooks passed\!"
exit 0

# To bypass this hook: git push --no-verify
HOOK_EOF

# Make the hooks executable
chmod +x "$HOOK_DIR/pre-commit"
chmod +x "$HOOK_DIR/pre-push"

echo "Git hooks installed successfully\!"
echo "- pre-commit: Runs make fmt (warning only)"
echo "- pre-push: Runs make test (blocks push if tests fail)"
echo ""
echo "To bypass hooks when needed, use the --no-verify flag."
echo "Example: git commit --no-verify or git push --no-verify"
