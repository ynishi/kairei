use kairei::parse_micro_agent;

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
    let result = parse_micro_agent(input);
    assert!(result.is_ok());
    let agent = result.unwrap().1;
    assert_eq!(agent.name, "TestAgent");
}
