// Basic MicroAgent example
const EXAMPLE: &str = r#"
micro Counter {
    state {
        count: Int = 0
    }

    observe {
        on Tick {
            self.count = self.count + 1
        }
    }

    answer {
        on request GetCount() -> Result<Int, Error> {
            return Ok(count)
        }
    }
}
"#;

fn main() {
    let result = kairei::parser::parse_micro_agent(EXAMPLE);
    println!("Parse result: {:?}", result);
}
