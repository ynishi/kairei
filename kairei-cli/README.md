# Kairei CLI

Command-line interface for interacting with the Kairei AI Agent Orchestration Platform.

## Installation

```bash
cargo install --path .
```

## Usage

The Kairei CLI provides both local functionality and remote API access:

```
kairei [OPTIONS] <COMMAND>
```

### Global Options

- `--config <CONFIG>`: Path to config file [default: config.json]
- `--secret <SECRET>`: Path to secret file [default: secret.json]
- `--verbose`: Enable debug mode
- `--api-url <API_URL>`: API server URL [default: http://localhost:3000]
- `--api-key <API_KEY>`: API key for authentication
- `--output <OUTPUT>`: Output format (json, yaml, table) [default: json]

### Commands

#### Running Locally

```bash
# Format a DSL file
kairei fmt [OPTIONS] [FILE]

# Run a local Kairei system
kairei run [OPTIONS] [DSL]
```

#### System Management (Remote API)

```bash
# List all systems
kairei system list

# Create a new system
kairei system create --name <NAME> [--description <DESCRIPTION>] [--config-file <CONFIG_FILE>]

# Get system details
kairei system get <ID>

# Start a system
kairei system start <ID> [--dsl <DSL>]

# Stop a system
kairei system stop <ID>

# Delete a system
kairei system delete <ID>
```

#### Agent Management (Remote API)

```bash
# List all agents in a system
kairei agent list <SYSTEM_ID>

# Get agent details
kairei agent get <SYSTEM_ID> <AGENT_ID>

# Create a new agent
kairei agent create <SYSTEM_ID> --file <FILE>

# Update an agent
kairei agent update <SYSTEM_ID> <AGENT_ID> --file <FILE>

# Start an agent
kairei agent start <SYSTEM_ID> <AGENT_ID>

# Stop an agent
kairei agent stop <SYSTEM_ID> <AGENT_ID>

# Delete an agent
kairei agent delete <SYSTEM_ID> <AGENT_ID>
```

#### Event Management (Remote API)

```bash
# List events in a system
kairei event list <SYSTEM_ID>

# Emit an event to a system
kairei event emit <SYSTEM_ID> --file <FILE>
```

## Examples

### Local Usage

Format a DSL file:
```bash
kairei fmt --stdout path/to/file.kairei
```

Run a local system:
```bash
kairei run --dsl path/to/agent.kairei
```

### Remote API Usage

List all systems:
```bash
kairei system list --api-url https://api.kairei.example.com --api-key your_api_key
```

Create a new agent:
```bash
kairei agent create test-system-id --file agent_definition.json --api-key your_api_key
```

Emit an event:
```bash
kairei event emit test-system-id --file event.json --api-key your_api_key
```

## Credential Management

The Kairei CLI now provides multiple ways to manage your API credentials:

```bash
# Save API credentials
kairei login --api-key your_api_key --api-url https://api.kairei.example.com

# Save credentials and also create a .env file
kairei login --api-key your_api_key --env

# View current credentials (key will be partially masked)
kairei login

# Test your current credentials
kairei login --test
```

Credentials are stored securely in your system's user config directory and are used automatically when making API requests.

### Credential Precedence

API credentials are loaded from multiple sources in the following order of precedence:

1. Command-line arguments: `--api-key` and `--api-url`
2. Environment variables: `KAIREI_API_KEY` and `KAIREI_API_URL`
3. `.env` file in the current directory
4. Saved credentials in the user's config directory

## Environment Variables

- `KAIREI_API_KEY`: API key for authentication
- `KAIREI_API_URL`: API server URL
- `RUST_LOG`: Controls log level (error, warn, info, debug, trace)