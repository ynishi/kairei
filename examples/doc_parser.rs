use kairei_core::analyzer::prelude::*;
use kairei_core::analyzer::{DocParserExt, ParserCategory, ParserDocumentation};
use kairei_core::ast;
use kairei_core::tokenizer::token::Token;

/// An example of how to document the `parse_expression` parser
pub fn documented_parse_expression() -> impl DocParserExt<Token, ast::Expression> {
    // Create the original parser
    let expression_parser = with_context(lazy(parse_binary_expression), "expression");
    
    // Create documentation for the parser
    let documentation = DocBuilder::new("parse_expression", ParserCategory::Expression)
        .description("Parses a complete expression in the KAIREI DSL")
        .example("x + 5")
        .example("foo(bar)")
        .example("think(\"What is the weather like?\")")
        .related_parser("parse_binary_expression")
        .related_parser("parse_primary")
        .build();
    
    // Wrap the parser with documentation
    document(expression_parser, documentation)
}

/// An example of how to document the `parse_think` parser
pub fn documented_parse_think() -> impl DocParserExt<Token, ast::Expression> {
    // Create the original parser
    let think_parser = with_context(
        choice(vec![
            Box::new(parse_think_multiple()),
            Box::new(parse_think_single()),
        ]),
        "think",
    );
    
    // Create documentation for the parser
    let documentation = DocBuilder::new("parse_think", ParserCategory::Expression)
        .description("Parses a think expression for LLM invocation")
        .example("think(\"What is the capital of France?\")")
        .example("think(\"Summarize this text\", text)")
        .example("think(prompt: \"Generate ideas\", context: data) with { model: \"gpt-4\" }")
        .related_parser("parse_think_single")
        .related_parser("parse_think_multiple")
        .related_parser("parse_think_attributes")
        .build();
    
    // Wrap the parser with documentation
    document(think_parser, documentation)
}

/// Function that shows how to extract and use documentation
pub fn example_documentation_usage() {
    // Create a documented parser
    let parser = documented_parse_think();
    
    // Access its documentation
    let doc = parser.documentation();
    
    println!("Parser: {}", doc.name);
    println!("Category: {}", doc.category);
    println!("Description: {}", doc.description);
    
    println!("\nExamples:");
    for (i, example) in doc.examples.iter().enumerate() {
        println!("  {}. {}", i + 1, example);
    }
    
    println!("\nRelated parsers:");
    for related in &doc.related_parsers {
        println!("  - {}", related);
    }
}

/// A simplified example of how to collect documentation from parsers
pub fn collect_parser_documentation() -> Vec<ParserDocumentation> {
    let mut docs = Vec::new();
    
    // Register documented parsers
    let parsers: Vec<Box<dyn DocParserExt<Token, ast::Expression>>> = vec![
        Box::new(documented_parse_expression()),
        Box::new(documented_parse_think()),
    ];
    
    // Collect documentation
    for parser in parsers {
        docs.push(parser.documentation().clone());
    }
    
    docs
}

fn main() {
    example_documentation_usage();
    
    let all_docs = collect_parser_documentation();
    println!("\nCollected {} parser documentation entries", all_docs.len());
}