#!/bin/bash

# Script to install Git hooks for the kairei project
# This script installs pre-commit hooks that run make fmt and make test

HOOK_DIR=$(git rev-parse --git-dir)/hooks
SCRIPT_DIR=$(dirname "$0")
PROJECT_ROOT=$(git rev-parse --show-toplevel)

echo "Installing Git hooks..."

# Create pre-commit hook
cat > "$HOOK_DIR/pre-commit" << 'EOF'
#!/bin/bash

echo "Running pre-commit hooks..."

# Run make fmt
echo "Running make fmt..."
make fmt
FMT_EXIT_CODE=$?

if [ $FMT_EXIT_CODE -ne 0 ]; then
  echo "Error: make fmt failed. Please fix the formatting issues before committing."
  exit 1
fi

# Run make test
echo "Running make test..."
make test
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
  echo "Error: make test failed. Please fix the test failures before committing."
  exit 1
fi

echo "All pre-commit hooks passed!"
exit 0
EOF

# Make the hook executable
chmod +x "$HOOK_DIR/pre-commit"

echo "Git hooks installed successfully!"
