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
        let code_chars: Vec<char> = code.chars().collect();
        let code_len = code_chars.len();

        while pos < code_len {
            // Search for the keyword
            if let Some(start_idx) = self.find_keyword(code, keyword, pos) {
                // Find the start of the block
                let block_start = start_idx;
                let mut block_end = code_len;

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

                        // Convert character indices to byte indices for slicing
                        let mut char_to_byte_map = Vec::new();

                        for (i, _) in code.char_indices() {
                            char_to_byte_map.push(i);
                        }
                        char_to_byte_map.push(code.len()); // Add end position

                        for (i, c) in code_chars.iter().enumerate().take(code_len).skip(paren_pos) {
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
                            let byte_end_pos = char_to_byte_map[end_pos];
                            let after_paren = &code[byte_end_pos + 1..];
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
                                    block_end = byte_end_pos + 1;
                                }
                            } else {
                                // Just think() with no block
                                block_end = byte_end_pos + 1;
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
    /// * `start_pos` - The position to start searching from (in character indices)
    ///
    /// # Returns
    /// The position of the keyword (in byte indices), or None if not found
    fn find_keyword(&self, code: &str, keyword: &str, start_pos: usize) -> Option<usize> {
        // Create a mapping from character positions to byte positions
        let mut char_to_byte_map = Vec::new();
        for (byte_idx, _) in code.char_indices() {
            char_to_byte_map.push(byte_idx);
        }
        char_to_byte_map.push(code.len()); // Add end position

        // Convert start_pos from character index to byte index
        let byte_start_pos = if start_pos < char_to_byte_map.len() {
            char_to_byte_map[start_pos]
        } else {
            return None; // Start position is beyond the end of the string
        };

        // Get the search area as a substring
        let search_area = &code[byte_start_pos..];

        // Different pattern based on keyword
        let pattern = if keyword == "think" {
            // For think, it's often used in assignments like "result = think(...)"
            format!(r"(?:\s|^|=\s*){}(?:\s*\()", regex::escape(keyword))
        } else {
            // For other keywords, they're followed by whitespace or block start
            format!(r"(?:\s|^){}(?:\s|\{{)", regex::escape(keyword))
        };

        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(mat) = re.find(search_area) {
                // Find the keyword within the match
                if let Some(keyword_offset) = search_area[mat.start()..].find(keyword) {
                    let keyword_byte_pos = byte_start_pos + mat.start() + keyword_offset;
                    return Some(keyword_byte_pos);
                }
            }
        }

        None
    }

    /// Find the matching closing brace
    ///
    /// # Arguments
    /// * `code` - The code to search in
    /// * `open_brace_pos` - The position of the opening brace (in bytes)
    ///
    /// # Returns
    /// The position of the matching closing brace (in bytes), or None if not found
    fn find_matching_brace(&self, code: &str, open_brace_pos: usize) -> Option<usize> {
        // Create a mapping from byte positions to character positions
        let mut byte_to_char_map = Vec::new();
        for (i, _) in code.char_indices() {
            byte_to_char_map.push(i);
        }
        byte_to_char_map.push(code.len()); // Add end position

        // Find the character position corresponding to the byte position
        let mut char_pos = 0;
        for (i, pos) in byte_to_char_map.iter().enumerate() {
            if *pos >= open_brace_pos {
                char_pos = i;
                break;
            }
        }

        let chars: Vec<char> = code.chars().collect();
        let mut depth = 0;

        for (i, &c) in chars.iter().enumerate().skip(char_pos) {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        // Convert back to byte position for the result
                        return Some(byte_to_char_map[i]);
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
    /// * `start_pos` - The position to start searching from (in bytes)
    ///
    /// # Returns
    /// The position of the end of the statement (in bytes)
    fn find_statement_end(&self, code: &str, start_pos: usize) -> Option<usize> {
        // Create a mapping from byte positions to character positions
        let mut byte_to_char_map = Vec::new();
        let mut char_to_byte_map = Vec::new();

        for (i, (byte_idx, _)) in code.char_indices().enumerate() {
            byte_to_char_map.push(i);
            char_to_byte_map.push(byte_idx);
        }
        byte_to_char_map.push(code.len()); // Add end position
        char_to_byte_map.push(code.len()); // Add end position

        // Find the character position corresponding to the byte position
        let mut char_pos = 0;
        for (i, pos) in byte_to_char_map.iter().enumerate() {
            if *pos >= start_pos {
                char_pos = i;
                break;
            }
        }

        let chars: Vec<char> = code.chars().collect();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, &c) in chars.iter().enumerate().skip(char_pos) {
            match c {
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
                                return Some(char_to_byte_map[i + 2]);
                            }
                            return Some(char_to_byte_map[i + 1]);
                        }
                    }
                }
                ';' => {
                    if !in_string && depth == 0 {
                        return Some(char_to_byte_map[i + 1]);
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
        Some(code.len())
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

        // Create a mapping between character indices and byte indices for the parent content
        let mut char_indices = Vec::new();
        for (i, _) in parent_content.char_indices() {
            char_indices.push(i);
        }
        char_indices.push(parent_content.len()); // Add end position

        // Extract the content inside the parent block (between the braces)
        if let Some(content_start) = parent_content.find('{') {
            // Convert to character index
            let content_start_char_idx = parent_content[..content_start].chars().count();
            let parent_content_char_len = parent_content.chars().count();

            // Get the content between the braces using character indices
            let content_chars: Vec<char> = parent_content.chars().collect();
            let content_str: String = content_chars
                [content_start_char_idx + 1..parent_content_char_len - 1]
                .iter()
                .collect();

            // List of keywords to extract at the top level
            let keywords = vec![
                "policy", "state", "answer", "observe", "onInit", "onEnd", "on", "think", "await",
                "world",
            ];

            // Extract top-level elements for each keyword
            for keyword in &keywords {
                let extracted = self.extract_top_level_elements(&content_str, keyword);
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

        // Convert content to characters for easier processing
        let content_chars: Vec<char> = content.chars().collect();

        // Find all occurrences of the keyword
        let mut keyword_positions = Vec::new();
        let keyword_chars: Vec<char> = keyword.chars().collect();

        // Simple search for the keyword
        for (i, _) in content_chars.iter().enumerate() {
            if i + keyword_chars.len() <= content_chars.len() {
                let mut match_found = true;
                for (j, kc) in keyword_chars.iter().enumerate() {
                    if content_chars[i + j] != *kc {
                        match_found = false;
                        break;
                    }
                }

                // Check if it's a word boundary
                if match_found {
                    let is_start_boundary = i == 0 || !content_chars[i - 1].is_alphanumeric();
                    let is_end_boundary = i + keyword_chars.len() >= content_chars.len()
                        || !content_chars[i + keyword_chars.len()].is_alphanumeric();

                    if is_start_boundary && is_end_boundary {
                        keyword_positions.push(i);
                    }
                }
            }
        }

        // Process each keyword occurrence
        for &start_pos in &keyword_positions {
            let mut end_pos = content_chars.len();

            // For policy and similar keywords without blocks, find the end of the statement
            if keyword == "policy" {
                // Find the first quote after the keyword
                let mut quote_pos = None;
                for (i, content_char) in content_chars
                    .iter()
                    .enumerate()
                    .skip(start_pos + keyword.chars().count())
                {
                    if *content_char == '"' {
                        quote_pos = Some(i);
                        break;
                    }
                }

                if let Some(q_pos) = quote_pos {
                    // Check for triple quotes
                    if q_pos + 2 < content_chars.len()
                        && content_chars[q_pos] == '"'
                        && content_chars[q_pos + 1] == '"'
                        && content_chars[q_pos + 2] == '"'
                    {
                        // Find closing triple quotes
                        for (i, &c) in content_chars
                            .iter()
                            .enumerate()
                            .skip(q_pos + 3)
                            .take(content_chars.len() - 2 - (q_pos + 3))
                        {
                            if c == '"'
                                && i + 1 < content_chars.len()
                                && content_chars[i + 1] == '"'
                                && i + 2 < content_chars.len()
                                && content_chars[i + 2] == '"'
                            {
                                end_pos = i + 3;
                                break;
                            }
                        }
                    } else {
                        // Find closing single quote
                        for (i, &c) in content_chars.iter().enumerate().skip(q_pos + 1) {
                            if c == '"' {
                                end_pos = i + 1;
                                break;
                            }
                        }
                    }
                }
            }
            // For keywords with blocks, find the end of the block
            else {
                // Find opening brace
                let mut brace_pos = None;
                for (i, &c) in content_chars
                    .iter()
                    .enumerate()
                    .skip(start_pos + keyword.chars().count())
                {
                    if c == '{' {
                        brace_pos = Some(i);
                        break;
                    }
                }

                if let Some(b_pos) = brace_pos {
                    // Find matching closing brace
                    let mut depth = 0;
                    for (i, &c) in content_chars.iter().enumerate().skip(b_pos) {
                        match c {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    end_pos = i + 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // No block, find end of statement (semicolon)
                    for (i, &c) in content_chars.iter().enumerate().skip(start_pos) {
                        if c == ';' {
                            end_pos = i + 1;
                            break;
                        }
                    }
                }
            }

            // Extract the element
            let element: String = content_chars[start_pos..end_pos].iter().collect();
            elements.push(element.trim().to_string());
        }
        elements
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

    #[test]
    fn test_utf8_multibyte_support() {
        // Create a test DSL string with Japanese characters
        let dsl = r#"
micro 旅行エージェント {
    policy "旅行プランの提案と予約管理"
    policy "ユーザー体験の最適化"
    
    state {
        予約数: Int = 0;
        ユーザー設定: String = "デフォルト";
        最終更新: Duration;
    }
    
    answer {
        on request 旅行プラン作成(目的地: String, 日数: Int) -> Result<String, Error> {
        プラン = think("""
            以下の条件で旅行プランを作成します：
            
            目的地: ${目的地}
            日数: ${日数}日間
            
            1日目: 観光スポット巡り
            2日目: 現地体験アクティビティ
            3日目以降: 自由行動
            
            オプション：
            - 現地ガイド手配
            - レストラン予約
            - 交通手段手配
        """)
    
        return プラン
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
        assert!(blocks.contains_key("state"), "Should contain 'state' block");
        assert!(
            blocks.contains_key("answer"),
            "Should contain 'answer' block"
        );
        assert!(blocks.contains_key("on"), "Should contain 'on' block");
        assert!(blocks.contains_key("think"), "Should contain 'think' block");

        // Verify the content of the blocks contains the Japanese characters
        if let Some(micro_blocks) = blocks.get("micro") {
            assert_eq!(micro_blocks.len(), 1, "Should have 1 micro block");
            assert!(
                micro_blocks[0].contains("旅行エージェント"),
                "Micro block should contain '旅行エージェント'"
            );
        }

        if let Some(state_blocks) = blocks.get("state") {
            assert_eq!(state_blocks.len(), 1, "Should have 1 state block");
            assert!(
                state_blocks[0].contains("予約数"),
                "State block should contain '予約数'"
            );
            assert!(
                state_blocks[0].contains("ユーザー設定"),
                "State block should contain 'ユーザー設定'"
            );
            assert!(
                state_blocks[0].contains("最終更新"),
                "State block should contain '最終更新'"
            );
        }

        if let Some(on_blocks) = blocks.get("on") {
            assert_eq!(on_blocks.len(), 1, "Should have 1 on block");
            assert!(
                on_blocks[0].contains("旅行プラン作成"),
                "On block should contain '旅行プラン作成'"
            );
            assert!(
                on_blocks[0].contains("目的地"),
                "On block should contain '目的地'"
            );
            assert!(
                on_blocks[0].contains("日数"),
                "On block should contain '日数'"
            );
        }

        if let Some(think_blocks) = blocks.get("think") {
            assert_eq!(think_blocks.len(), 1, "Should have 1 think block");

            // Check if the think block contains the expected Japanese text
            assert!(
                think_blocks[0].contains("以下の条件で旅行プランを作成します"),
                "Think block should contain '以下の条件で旅行プランを作成します'"
            );

            // Check for other text that is actually present in the think block
            assert!(
                think_blocks[0].contains("目的地"),
                "Think block should contain '目的地'"
            );
            assert!(
                think_blocks[0].contains("日数"),
                "Think block should contain '日数'"
            );
            assert!(
                think_blocks[0].contains("1日目: 観光スポット巡り"),
                "Think block should contain '1日目: 観光スポット巡り'"
            );
        }

        // Test the one-tier extraction
        let one_tier_blocks = splitter.split_dsl_blocks_one_tier(dsl, "micro");

        // Verify that the one-tier blocks were correctly extracted
        assert!(
            one_tier_blocks.contains_key("policy_one"),
            "Should contain 'policy_one' blocks"
        );
        assert!(
            one_tier_blocks.contains_key("state_one"),
            "Should contain 'state_one' blocks"
        );
        assert!(
            one_tier_blocks.contains_key("answer_one"),
            "Should contain 'answer_one' blocks"
        );

        // Verify the content of the one-tier blocks
        if let Some(policy_blocks) = one_tier_blocks.get("policy_one") {
            assert_eq!(policy_blocks.len(), 2, "Should have 2 policy blocks");
            assert!(
                policy_blocks[0].contains("旅行プランの提案"),
                "First policy should contain '旅行プランの提案'"
            );
            assert!(
                policy_blocks[1].contains("ユーザー体験"),
                "Second policy should contain 'ユーザー体験'"
            );
        }

        if let Some(state_blocks) = one_tier_blocks.get("state_one") {
            assert_eq!(state_blocks.len(), 1, "Should have 1 state block");
            assert!(
                state_blocks[0].contains("予約数"),
                "State block should contain '予約数'"
            );
        }

        if let Some(answer_blocks) = one_tier_blocks.get("answer_one") {
            assert_eq!(answer_blocks.len(), 1, "Should have 1 answer block");
            assert!(
                answer_blocks[0].contains("旅行プラン作成"),
                "Answer block should contain '旅行プラン作成'"
            );
        }
    }
}
