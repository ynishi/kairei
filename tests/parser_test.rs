use kairei::{analyzer::Parser, preprocessor::Preprocessor};

extern crate kairei;

#[test]
fn it_parse_micro_agent() {
    let input = r#"
        micro TestAgent {
            lifecycle {
                onInit {
                    counter = 0
                }
                onDestroy {
                    emit Shutdown to manager
                }
            }
            state {
                counter: Int = 0,
                name: String = "test",
                active: Bool = true
            }
            observe {
                on Tick {
                    counter = counter + 1
                }
                on StateUpdated { agent: "other", state: "value" } {
                    name = "updated"
                }
            }
            answer {
                on request GetCount() -> Result<Int, Error> {
                    return Ok(counter)
                }

                on request SetName(newName: String) -> Result<Bool, Error>
                with constraints { strictness: 0.9, stability: 0.95 }
                {
                    name = newName
                    return Ok(true)
                }
            }
            react {
                on Message { content: "reset" } {
                    counter = 0
                    emit StateUpdated { agent: "self", state: "counter" } to manager
                }
            }
        }
    "#;
    let result = kairei::tokenizer::token::Tokenizer::new()
        .tokenize(input)
        .unwrap();
    let preprocessor = kairei::preprocessor::TokenPreprocessor::default();
    let tokens = preprocessor.process(result);
    let (_, agent_def) = kairei::analyzer::parsers::agent::parse_agent_def()
        .parse(tokens.as_slice(), 0)
        .unwrap();

    assert_eq!(agent_def.name, "TestAgent");
}
