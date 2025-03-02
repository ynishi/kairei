# KAIREI

KAIREI is an AI Agent Orchestration Platform leveraging LLMs. It provides a flexible and scalable development and execution environment for AI agents using an intuitive DSL and event-driven architecture.

## Origin of KAIREI 🌊
KAIREI (海嶺, meaning 'Ridge' in Japanese) represents the fusion of **high-level context (intuitive DSL) and low-level high performance (optimized in Rust)**. It is designed to work efficiently in various environments, from cloud to edge, aiming to provide a seamless AI agent infrastructure.

## Vision of KAIREI 🤝
KAIREI aims to be a **co-creation AI platform**, where humans and AI collaborate in system development:
- **Humans and AI collaboratively generate ideas**
- **AI generates DSL based on ideas and dynamically constructs & executes agent systems**
- **Provides an environment where individuals and prototypes can handle tens to hundreds of agents**

With KAIREI, large-scale AI agent systems—once limited to enterprises and research institutions—can be built effortlessly by anyone! 🚀✨

## Features ✨
- **Intuitive DSL**: Simple syntax for constructing multi-agent AI systems
- **Event-driven architecture**: Optimized asynchronous processing for high performance and scalability
- **Type system for consistency**: Ensures safe and structured agent communication
- **High performance with Rust**: The generated binaries are lightweight and run efficiently across various environments

## Quick Start 🚀
First, install KAIREI and try running a simple AI agent.

### Installation
```sh
# Install dependencies
cd kairei
cargo build
```

### Example: Travel Planning Agent
With KAIREI, you can define an AI agent using the following DSL:

```kairei
type TravelPlanner {
    policy "Optimize travel plans considering budget and comfort"
    
    on request PlanTrip(destination: String, budget: Float) -> Result<String, Error> {
        flights = request FindFlights(destination, budget * 0.4)
        hotels = request FindHotels(destination, budget * 0.4)
        attractions = request FindAttractions(destination, budget * 0.2)
        
        return think("Generate the best travel plan using: ${flights}, ${hotels}, ${attractions}")
    }
}
```

Executing this code allows the agent to **generate an optimal travel itinerary within budget!** ✨

### 🔧 Running API Tests for Travel Planning
To enable API tests and run the Travel Planning example, follow these steps:

#### 1️. Setup API Credentials
```sh
cp tests/micro_agent_tests/test_secret.json.example tests/micro_agent_tests/test_secret.json
```
→ This creates a test_secret.json file.

Now, edit test_secret.json and replace the placeholders with your actual API keys:
- OpenAI API Key
- Serper API Key

#### 2. Run API Tests
Execute the following command to run the Travel Planner test:

```sh
RUN_API_TESTS=true cargo test -p kairei micro_agent_tests::travel_planning_test::test_travel_planner
```
→ This will run the KAIREI Travel Planner test and output debug logs. 🚀✨

## Architecture Overview 🏗
KAIREI consists of the following components:

- **DSL Parser**: Parses the DSL and constructs agents
- **Event Bus**: Manages inter-agent communication and optimizes asynchronous processing
- **Runtime**: Optimizes and processes agents to operate in an asynchronous environment
- **System**: Provides a secure and scalable execution environment for agents

## Contributing 🤝
KAIREI is an open-source project! Contributions, including bug reports, feature suggestions, and code improvements, are welcome!

1. Fork this repository
2. Work on the `develop` branch
3. Create a PR!

## License
MIT License

