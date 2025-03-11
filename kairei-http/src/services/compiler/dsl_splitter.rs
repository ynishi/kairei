use std::collections::HashMap;

/// Splitter for Kairei DSL code
#[derive(Debug, Clone, Default)]
pub struct DslSplitter;

impl DslSplitter {
    /// Create a new DslSplitter
    pub fn new() -> Self {
        Self
    }

    /// Split DSL code into blocks by keyword
    ///
    /// # Arguments
    /// * `code` - The DSL code to split
    ///
    /// # Returns
    /// A HashMap with keywords as keys and lists of related blocks as values
    pub fn split_dsl_blocks(&self, code: &str) -> HashMap<String, Vec<String>> {
        let mut blocks = HashMap::new();

        // List of keywords to extract
        let keywords = vec![
            "micro", "answer", "state", "observe", "onInit", "onEnd", "on", "think", "await",
            "world",
        ];

        // Extract blocks for each keyword
        for keyword in &keywords {
            let extracted_blocks = self.extract_blocks(code, keyword);
            if !extracted_blocks.is_empty() {
                blocks.insert(keyword.to_string(), extracted_blocks);
            }
        }

        blocks
    }

    /// Extract blocks related to a specific keyword
    ///
    /// # Arguments
    /// * `code` - The DSL code to analyze
    /// * `keyword` - The keyword to extract blocks for
    ///
    /// # Returns
    /// A list of extracted blocks
    fn extract_blocks(&self, code: &str, keyword: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        let mut pos = 0;

        while pos < code.len() {
            // Search for the keyword
            if let Some(start_idx) = self.find_keyword(code, keyword, pos) {
                // Find the start of the block
                let block_start = start_idx;
                let mut block_end = code.len();

                // Handle different keyword types
                if keyword == "think" {
                    // Handle think with different patterns
                    if let Some(paren_start) = code[start_idx..].find('(') {
                        let paren_pos = start_idx + paren_start;

                        // Find the matching closing parenthesis
                        let mut paren_depth = 0;
                        let mut in_string = false;
                        let mut escape_next = false;
                        let mut paren_end = None;

                        for i in paren_pos..code.len() {
                            let c = code.chars().nth(i).unwrap_or(' ');

                            match c {
                                '\\' if in_string => escape_next = !escape_next,
                                '"' if !escape_next => in_string = !in_string,
                                '(' => {
                                    if !in_string {
                                        paren_depth += 1
                                    }
                                }
                                ')' => {
                                    if !in_string {
                                        paren_depth -= 1;
                                        if paren_depth == 0 {
                                            paren_end = Some(i);
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    if in_string {
                                        escape_next = false
                                    }
                                }
                            }
                        }

                        if let Some(end_pos) = paren_end {
                            // Check if there's a block after the parenthesis
                            let after_paren = &code[end_pos + 1..];
                            let trimmed = after_paren.trim_start();

                            if trimmed.starts_with('{') {
                                // Handle think() with block
                                let brace_pos = code.len() - trimmed.len();
                                if let Some(brace_end) = self.find_matching_brace(code, brace_pos) {
                                    block_end = brace_end + 1;
                                }
                            } else if trimmed.starts_with("with") {
                                // Handle think() with {...} {...}
                                let with_pos = code.len() - trimmed.len();
                                let after_with = &code[with_pos + 4..];
                                let trimmed_after_with = after_with.trim_start();

                                if trimmed_after_with.starts_with('{') {
                                    let first_brace_pos = code.len() - trimmed_after_with.len();
                                    if let Some(first_brace_end) =
                                        self.find_matching_brace(code, first_brace_pos)
                                    {
                                        let after_first_brace = &code[first_brace_end + 1..];
                                        let trimmed_after_first = after_first_brace.trim_start();

                                        if trimmed_after_first.starts_with('{') {
                                            let second_brace_pos =
                                                code.len() - trimmed_after_first.len();
                                            if let Some(second_brace_end) =
                                                self.find_matching_brace(code, second_brace_pos)
                                            {
                                                block_end = second_brace_end + 1;
                                            }
                                        } else {
                                            block_end = first_brace_end + 1;
                                        }
                                    }
                                } else {
                                    // Just think() with no block
                                    block_end = end_pos + 1;
                                }
                            } else {
                                // Just think() with no block
                                block_end = end_pos + 1;
                            }
                        }
                    }
                } else if keyword == "await" {
                    // Handle await similar to think
                    if let Some(end_pos) = self.find_statement_end(code, start_idx) {
                        block_end = end_pos;
                    }
                } else {
                    // For other keywords, check if there's a brace
                    if let Some(brace_start) = code[start_idx..].find('{') {
                        let brace_pos = start_idx + brace_start;

                        // Find the matching closing brace
                        if let Some(end_pos) = self.find_matching_brace(code, brace_pos) {
                            block_end = end_pos + 1;
                        }
                    }
                }

                // Extract the block
                let block = &code[block_start..block_end];
                blocks.push(block.to_string());

                // Update the position for the next search
                pos = block_end;
            } else {
                // If the keyword wasn't found, move to the next position
                pos += 1;
            }
        }

        blocks
    }

    /// Find a keyword in the code
    ///
    /// # Arguments
    /// * `code` - The code to search in
    /// * `keyword` - The keyword to search for
    /// * `start_pos` - The position to start searching from
    ///
    /// # Returns
    /// The position of the keyword, or None if not found
    fn find_keyword(&self, code: &str, keyword: &str, start_pos: usize) -> Option<usize> {
        // Simple keyword search
        let search_area = &code[start_pos..];

        // Different pattern based on keyword
        let pattern = if keyword == "think" {
            // For think, it's often used in assignments like "result = think(...)"
            format!(r"(?:\s|^|=\s*){}(?:\s*\()", keyword)
        } else {
            // For other keywords, they're followed by whitespace or block start
            format!(r"(?:\s|^){}(?:\s|\{{)", keyword)
        };

        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(mat) = re.find(search_area) {
                // Calculate the actual start position of the keyword
                let keyword_start = start_pos + mat.start();
                // Skip whitespace before the keyword
                let actual_start = code[keyword_start..].find(keyword).unwrap_or(0) + keyword_start;
                return Some(actual_start);
            }
        }

        None
    }

    /// Find the matching closing brace
    ///
    /// # Arguments
    /// * `code` - The code to search in
    /// * `open_brace_pos` - The position of the opening brace
    ///
    /// # Returns
    /// The position of the matching closing brace, or None if not found
    fn find_matching_brace(&self, code: &str, open_brace_pos: usize) -> Option<usize> {
        let chars: Vec<char> = code.chars().collect();
        let mut depth = 0;

        for (i, c) in chars.iter().enumerate().skip(open_brace_pos) {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Find the end of a statement (for think() or await())
    ///
    /// # Arguments
    /// * `code` - The code to search in
    /// * `start_pos` - The position to start searching from
    ///
    /// # Returns
    /// The position of the end of the statement
    fn find_statement_end(&self, code: &str, start_pos: usize) -> Option<usize> {
        let chars: Vec<char> = code.chars().collect();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for i in start_pos..chars.len() {
            match chars[i] {
                '\\' if in_string => escape_next = !escape_next,
                '"' if !escape_next => in_string = !in_string,
                '(' => {
                    if !in_string {
                        depth += 1
                    }
                }
                ')' => {
                    if !in_string {
                        depth -= 1;
                        if depth == 0 {
                            // Include semicolon after parenthesis if present
                            if i + 1 < chars.len() && chars[i + 1] == ';' {
                                return Some(i + 2);
                            }
                            return Some(i + 1);
                        }
                    }
                }
                ';' => {
                    if !in_string && depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {
                    if in_string {
                        escape_next = false
                    }
                }
            }
        }

        // If we reach the end of the code
        Some(chars.len())
    }

    /// Split DSL code into one-tier blocks (only top-level elements within a specified block)
    ///
    /// # Arguments
    /// * `code` - The DSL code to split
    /// * `parent_block` - The parent block to extract from (e.g., "micro")
    ///
    /// # Returns
    /// A HashMap with keywords as keys and lists of related blocks as values
    pub fn split_dsl_blocks_one_tier(
        &self,
        code: &str,
        parent_block: &str,
    ) -> HashMap<String, Vec<String>> {
        let mut blocks = HashMap::new();

        // First, extract the parent block
        let parent_blocks = self.extract_blocks(code, parent_block);
        if parent_blocks.is_empty() {
            return blocks;
        }

        // Use the first parent block found
        let parent_content = &parent_blocks[0];

        // Extract the content inside the parent block (between the braces)
        if let Some(content_start) = parent_content.find('{') {
            let content = &parent_content[content_start + 1..parent_content.len() - 1];

            // List of keywords to extract at the top level
            let keywords = vec![
                "policy", "state", "answer", "observe", "onInit", "onEnd", "on", "think", "await",
                "world",
            ];

            // Extract top-level elements for each keyword
            for keyword in &keywords {
                let extracted = self.extract_top_level_elements(content, keyword);
                if !extracted.is_empty() {
                    blocks.insert(format!("{}_one", keyword), extracted);
                }
            }
        }

        blocks
    }

    /// Extract top-level elements for a specific keyword
    ///
    /// # Arguments
    /// * `content` - The content to analyze
    /// * `keyword` - The keyword to extract elements for
    ///
    /// # Returns
    /// A list of extracted elements
    fn extract_top_level_elements(&self, content: &str, keyword: &str) -> Vec<String> {
        let mut elements = Vec::new();
        let mut pos = 0;

        while pos < content.len() {
            // Search for the keyword
            if let Some(start_idx) = self.find_keyword_at_position(content, keyword, pos) {
                let element_start = start_idx;
                let mut element_end = content.len();

                // For policy and similar keywords without blocks, find the end of the statement
                if keyword == "policy" {
                    // Find the first quote after the keyword
                    if let Some(quote_start) = content[start_idx..].find('\"') {
                        let quote_pos = start_idx + quote_start;

                        // Handle triple quotes
                        if quote_pos + 2 < content.len()
                            && content[quote_pos..quote_pos + 3] == *"\"\"\""
                        {
                            // Find the closing triple quotes
                            if let Some(triple_quote_end) = content[quote_pos + 3..].find("\"\"\"")
                            {
                                element_end = quote_pos + 3 + triple_quote_end + 3;
                            }
                        } else {
                            // Handle single quotes - find the closing quote
                            if let Some(quote_end) = content[quote_pos + 1..].find('\"') {
                                element_end = quote_pos + 1 + quote_end + 1;
                            }
                        }
                    }
                }
                // For keywords with blocks, find the end of the statement or block
                else {
                    // Check if there's a brace after the keyword
                    if let Some(brace_start) = content[start_idx..].find('{') {
                        let brace_pos = start_idx + brace_start;

                        // For top-level extraction, we don't need to process the block content
                        // Just find the matching closing brace
                        if let Some(end_pos) = self.find_matching_brace(content, brace_pos) {
                            element_end = end_pos + 1;
                        }
                    } else {
                        // For keywords without braces, find the end of the statement
                        if let Some(end_pos) = self.find_statement_end(content, start_idx) {
                            element_end = end_pos;
                        }
                    }
                }

                // Extract the element
                let element = &content[element_start..element_end];
                elements.push(element.trim().to_string());

                // Update the position for the next search
                pos = element_end;
            } else {
                // If the keyword wasn't found, move to the next position
                pos += 1;
            }
        }

        elements
    }

    /// Find a keyword at a specific position in the code
    ///
    /// # Arguments
    /// * `code` - The code to search in
    /// * `keyword` - The keyword to search for
    /// * `start_pos` - The position to start searching from
    ///
    /// # Returns
    /// The position of the keyword, or None if not found
    fn find_keyword_at_position(
        &self,
        code: &str,
        keyword: &str,
        start_pos: usize,
    ) -> Option<usize> {
        if start_pos >= code.len() {
            return None;
        }

        // Find the keyword
        let search_area = &code[start_pos..];
        let keyword_pattern = format!(r"\b{}\b", regex::escape(keyword));

        if let Ok(re) = regex::Regex::new(&keyword_pattern) {
            if let Some(mat) = re.find(search_area) {
                return Some(start_pos + mat.start());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        }
    }

    #[test]
    fn test_split_dsl_blocks_one_tier() {
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
    }
}
