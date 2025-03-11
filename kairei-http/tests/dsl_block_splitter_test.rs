use kairei_http::services::compiler::DslSplitter;

#[test]
fn test_split_dsl_blocks() {
    // Create a test DSL string
    let dsl = r#"
micro StateValidator {
    policy "KAIREI validator for state, parts of DSL"
    policy "provide detailed error information"
    
    answer {
        on request ValidateState(stateInput: String, parseError: String) -> Result<String, Error> {
        stateResult = think("""
            Spec:
            state is group of state variable assignment.
            [pattern 1]
            {var_name}: {type_name} = {value}; // ; is needed
            [pattern 2]
            {var_name}: {type_name};
            [pattern 3]
            {var_name} = {value}; // it is allowed type infer
        
            Example:
            ```kairei
            micro TravelAgent {
                state {
                    active_trips: Int = 0;
                    user_preferences: String = "";
                    last_update: Duration;
                }
            }
            ```
        
            Validate target:
            ${stateInput}
        
            Error:
            ${parseError}
        """)
    
        return stateResult
        }
    }
}
    "#;

    // Create a DslSplitter instance
    let splitter = DslSplitter::new();

    // Split the DSL blocks
    let blocks = splitter.split_dsl_blocks(dsl);

    // Verify that the blocks were correctly split
    assert!(blocks.contains_key("micro"), "Should contain 'micro' block");
    assert!(
        blocks.contains_key("answer"),
        "Should contain 'answer' block"
    );
    assert!(blocks.contains_key("on"), "Should contain 'on' block");
    assert!(blocks.contains_key("think"), "Should contain 'think' block");

    // Verify the number of blocks for each keyword
    if let Some(micro_blocks) = blocks.get("micro") {
        assert_eq!(micro_blocks.len(), 1, "Should have 1 micro block");
    }

    if let Some(answer_blocks) = blocks.get("answer") {
        assert_eq!(answer_blocks.len(), 1, "Should have 1 answer block");
    }

    if let Some(on_blocks) = blocks.get("on") {
        assert_eq!(on_blocks.len(), 1, "Should have 1 on block");
    }

    if let Some(think_blocks) = blocks.get("think") {
        assert_eq!(think_blocks.len(), 1, "Should have 1 think block");
        println!("Think block: {}", think_blocks[0]);
        // We're not checking the content of the think block, just that it exists
    }

    // Print the blocks for debugging
    println!("Blocks: {:#?}", blocks);

    // Test the one-tier extraction
    let one_tier_blocks = splitter.split_dsl_blocks_one_tier(dsl, "micro");

    // Verify that the one-tier blocks were correctly extracted
    assert!(
        one_tier_blocks.contains_key("policy_one"),
        "Should contain 'policy_one' blocks"
    );
    assert!(
        one_tier_blocks.contains_key("answer_one"),
        "Should contain 'answer_one' blocks"
    );

    // Verify the number of blocks for each keyword
    if let Some(policy_blocks) = one_tier_blocks.get("policy_one") {
        assert_eq!(policy_blocks.len(), 2, "Should have 2 policy blocks");
        assert!(
            policy_blocks[0].contains("KAIREI validator"),
            "First policy should contain 'KAIREI validator'"
        );
        assert!(
            policy_blocks[1].contains("provide detailed error"),
            "Second policy should contain 'provide detailed error'"
        );
    }

    if let Some(answer_blocks) = one_tier_blocks.get("answer_one") {
        assert_eq!(answer_blocks.len(), 1, "Should have 1 answer block");
    }

    // Print the one-tier blocks for debugging
    println!("One-tier blocks: {:#?}", one_tier_blocks);
}
