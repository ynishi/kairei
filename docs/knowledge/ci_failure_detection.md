# CI Failure Detection and Loop Prevention

## Overview
A feature to detect consecutive CI failures and prevent loops caused by similar error patterns.

## Operation
1. When CI fails 3 consecutive times:
   - Automatically detects potential loop
   - Analyzes error patterns
   - Adds labels and comments to PR
   - Creates tracking issue

## Error Pattern Analysis
- Compares last 3 failure logs
- Detects identical error patterns
- Summarizes errors and suggests solutions

## Recommended Actions
1. Review error messages
2. Run tests locally
3. Consider task splitting if needed

## Implementation Details
- Implemented in `.github/workflows/ci-failure-detection.yml`
- Uses GitHub Actions workflow_run trigger
- Error pattern similarity analysis
