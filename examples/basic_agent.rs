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
    let result = kairei::tokenizer::token::Tokenizer::new().tokenize(EXAMPLE);
    println!("Parse result: {:?}", result);
}
