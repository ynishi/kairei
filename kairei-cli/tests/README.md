# Kairei CLI Tests

This directory contains integration tests for the Kairei CLI application.

## Test Structure

The test suite is organized as follows:

- **cli_login_test.rs**: Tests the `login` command functionality including credential storage and display.
- **api_integration_test.rs**: Tests the CLI commands that interact with the Kairei API.

## Unit Tests

Unit tests are located directly in the source files:

- **config.rs**: Contains tests for credential management, .env file integration, and configuration precedence.
- **api_client.rs**: Contains tests for API request handling, response parsing, and error handling.

## Running the Tests

To run all tests:

```bash
cargo test
```

To run a specific test file:

```bash
cargo test --test cli_login_test
```

To run a specific test:

```bash
cargo test test_login_save_credentials_to_env_file
```

## Test Dependencies

The tests use the following dependencies:

- **tempfile**: For creating temporary directories and files
- **mockito**: For mocking HTTP requests and responses
- **assert_cmd**: For testing CLI commands
- **predicates**: For assertions on command output

## Writing New Tests

When adding new features to the CLI, please add corresponding tests:

1. For new configuration options, add unit tests in `config.rs`
2. For new API endpoints, add unit tests in `api_client.rs`
3. For new CLI commands, add integration tests in this directory

### Testing Guidelines

- Use temporary files and directories whenever possible to avoid side effects
- Mock external API calls with mockito
- Pass configuration through CLI arguments to ensure test isolation
- Test both success and error cases
- For CLI output testing, prefer checking for key phrases rather than exact output