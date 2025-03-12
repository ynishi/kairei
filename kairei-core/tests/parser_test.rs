use kairei_core::{
    MicroAgentDef, analyzer::Parser, preprocessor::Preprocessor, tokenizer::token::Token,
};
use tracing::debug;

extern crate kairei_core;

fn parse_agent(input: &str) -> MicroAgentDef {
    let token_spans = kairei_core::tokenizer::token::Tokenizer::new()
        .tokenize(input)
        .unwrap();
    let preprocessor = kairei_core::preprocessor::TokenPreprocessor::default();
    let tokens: Vec<Token> = preprocessor
        .process(token_spans)
        .iter()
        .map(|e| e.token.clone())
        .collect();
    debug!("{:?}", tokens);
    let (_, agent_def) = kairei_core::analyzer::parsers::agent::parse_agent_def()
        .parse(tokens.as_slice(), 0)
        .unwrap();
    agent_def
}

#[test]
fn it_parse_micro_agent_state() {
    let input = r#"
        micro TestAgent {
            state {
                counter: Int = 0;
                name: String = "test";
                active: Bool = true;
            }
        }
    "#;
    let agent_def = parse_agent(input);
    debug!("{:?}", agent_def);
    assert_eq!(agent_def.name, "TestAgent");
    assert!(agent_def.state.is_some());
}

#[test]
fn it_parse_micro_agent_lifecycle() {
    let input = r#"
        micro TestAgent {
            lifecycle {
                onInit {
                    counter = 0
                }
                onDestroy {
                    emit Shutdown() to manager
                }
            }
        }
    "#;
    let agent_def = parse_agent(input);
    debug!("{:?}", agent_def);
    assert_eq!(agent_def.name, "TestAgent");
    assert!(agent_def.lifecycle.is_some());
}

#[test]
fn it_parse_micro_agent_observe() {
    let input = r#"
        micro TestAgent {
            observe {
                on Tick {
                    counter = counter + 1
                }
                on StateUpdated {
                    name = "updated"
                }
            }
        }
    "#;
    let agent_def = parse_agent(input);
    debug!("{:?}", agent_def);
    assert_eq!(agent_def.name, "TestAgent");
    assert!(agent_def.observe.is_some());
}

#[test]
fn it_parse_micro_agent_answer() {
    let input = r#"
        micro TestAgent {
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(counter)
                }

                on request SetName(newName: String) -> Result<Bool, Error>
                with { strictness: 0.9, stability: 0.95 }
                {
                    name = newName
                    return Ok(true)
                }
            }
        }
    "#;
    let agent_def = parse_agent(input);
    debug!("{:?}", agent_def);
    assert_eq!(agent_def.name, "TestAgent");
    assert!(agent_def.answer.is_some());
}

#[test]
fn it_parse_micro_agent_react() {
    let input = r#"
        micro TestAgent {
            react {
                on Message(status: String) {
                    counter = 0
                    emit StateUpdated(agent: self, counter: counter) to manager
                }
                on Message(CounterUpdated) {
                    counter = 0
                    emit StateUpdated(agent: self, counter: counter) to manager
                }
                on StatusMessage(status: String) {
                    counter = 0
                    emit StateUpdated(agent: self, counter: counter) to manager
                }
            }
        }
    "#;
    let agent_def = parse_agent(input);
    debug!("{:?}", agent_def);
    assert_eq!(agent_def.name, "TestAgent");
    assert!(agent_def.react.is_some());
}

#[test]
fn it_parse_micro_agent() {
    let input = r#"
        micro TestAgent {
            lifecycle {
                onInit {
                    counter = 0
                }
                onDestroy {
                    emit Shutdown() to manager
                }
            }
            state {
                counter: Int = 0;
                name: String = "test";
                active: Bool = true;
            }
            observe {
                on Tick {
                    counter = counter + 1
                }
                on StateUpdated {
                    name = "updated"
                }
            }
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(counter)
                }

                on request SetName(newName: String) -> Result<Bool, Error>
                with { strictness: 0.9, stability: 0.95 }
                {
                    name = newName
                    return Ok(true)
                }
            }
            react {
                on Message(CounterUpdated) {
                    counter = 0
                    emit StateUpdated(agent: self, counter: counter) to manager
                }
            }
        }
    "#;
    let result = kairei_core::tokenizer::token::Tokenizer::new()
        .tokenize(input)
        .unwrap();
    let preprocessor = kairei_core::preprocessor::TokenPreprocessor::default();
    let tokens: Vec<Token> = preprocessor
        .process(result)
        .iter()
        .map(|e| e.token.clone())
        .collect();
    debug!("{:?}", tokens);
    let (_, agent_def) = kairei_core::analyzer::parsers::agent::parse_agent_def()
        .parse(tokens.as_slice(), 0)
        .unwrap();

    assert_eq!(agent_def.name, "TestAgent");
}
