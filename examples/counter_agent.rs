use kairei::core::types::generate_event_enum;
use quote::quote;
use std::{fs::File, io::Write, process::Command};
use syn::{parse_quote, ItemFn};

// Basic MicroAgent example
const EXAMPLE: &str = r#"
micro Counter {
    state {
        count: i64 = 0
    }

    observe {
        on Tick {
            self.count = self.count + 1
        }
    }

    answer {
        on request GetCount() -> Result<i64, Error> {
            return Ok(self.count)
        }
    }
}
"#;

const CARGO_TOML: &str = r#"
[package]
name = "inner_project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
"#;

fn generate_main(code: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let event_enum = generate_event_enum();

    let main_fn: ItemFn = parse_quote! {
        fn main() {
            let mut agent = Counter {
                count: 0,
            };

            loop {
                agent.handle_event(&Event::Tick);

                let count = agent.GetCount().unwrap();
                println!("Count: {}", count);

                std::thread::sleep(std::time::Duration::from_secs(1));

                println!();

                if count >= 10 {
                    break;
                }
            }
        }
    };

    quote! {
        use anyhow::Error;

        #event_enum

        #main_fn

        #code
    }
}

#[tokio::main]
async fn main() {
    use kairei::ast::CodeGen;

    let (_, agent_def) = kairei::parser::parse_micro_agent(EXAMPLE).unwrap();

    let rust_code = agent_def.generate_rust();

    // インナープロジェクトのディレクトリを作成
    std::fs::create_dir_all("inner_project/src").unwrap();

    // 生成されたコードを inner_project/src/main.rs に書き出す
    let mut file = File::create("inner_project/src/main.rs").unwrap();
    file.write_all(generate_main(rust_code).to_string().as_bytes())
        .unwrap();

    // inner_project の Cargo.toml を作成 (必要に応じて)
    let mut cargo_toml = File::create("inner_project/Cargo.toml").unwrap();
    cargo_toml.write_all(CARGO_TOML.as_bytes()).unwrap();

    // inner_project ディレクトリで cargo build を実行
    let output = Command::new("cargo")
        .args(&["build"])
        .current_dir("inner_project")
        .output()
        .expect("failed to execute cargo build");

    if output.status.success() {
        println!("Successfully built generated code!");

        // cargo run を実行する場合
        // let output = Command::new("cargo")
        //     .args(&["run"])
        //     .current_dir("inner_project")
        //     .output()
        //     .expect("failed to execute cargo run");
        // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        eprintln!("Failed to build generated code:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
