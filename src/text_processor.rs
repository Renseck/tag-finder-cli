use regex::Regex;

pub struct TextProcessor {
    patterns: Vec<(String, Regex)>,
}

#[derive(Debug, Clone)]
pub struct TextMatch {
    pub pattern_name: String,
    pub matched_text: String,
    pub line: usize,
    pub column: usize,
}

// TODO Smarter filtering: using `type-{}` (formatted later) should flag `type-fire` as used
impl TextProcessor {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /* ========================================================================================== */
    pub fn add_pattern(mut self, name: &str, pattern: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let regex = Regex::new(pattern)?;
        self.patterns.push((name.to_string(), regex));
        Ok(self)
    }

    /* ========================================================================================== */
    pub fn process_content(&self, content: &str) -> Vec<TextMatch> {
        let mut matches = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            if self.is_ignored_line(line) {
                continue;
            }
            
            for (pattern_name, regex) in &self.patterns {
                for cap in regex.captures_iter(line) {
                    if let Some(matched) = cap.get(1) {
                        matches.push(TextMatch {
                            pattern_name: pattern_name.clone(),
                            matched_text: matched.as_str().to_string(),
                            line: line_num + 1,
                            column: matched.start(),
                        });
                    }
                }
            }
        }
        
        matches
    }

    /* ========================================================================================== */
    pub fn find_exact_words(&self, content: &str, target_word: &str) -> bool {
        content
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .any(|word| word == target_word)
    }

    /* ========================================================================================== */
    fn is_ignored_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty()
    }
}