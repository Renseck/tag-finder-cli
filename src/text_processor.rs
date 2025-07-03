use regex::Regex;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct DynamicPattern {
    pub prefix: String,
    pub suffix: String,
    pub pattern: String, // e.g., "type-{}"
    pub matching_classes: Vec<String>, // e.g., ["type-fire", "type-water"]
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
    pub fn detect_dynamic_patterns(&self, class_names: &[String]) -> Vec<DynamicPattern> {
        let mut pattern_groups: HashMap<String, Vec<String>> = HashMap::new();
        
        // Group classes by potential patterns
        for class_name in class_names {
            if let Some(pattern_key) = self.extract_pattern_key(class_name) {
                pattern_groups.entry(pattern_key).or_default().push(class_name.clone());
            }
        }
        
        // Filter groups that have multiple classes (indicating a pattern)
        let mut dynamic_patterns = Vec::new();
        for (_pattern_key, classes) in pattern_groups {
            if classes.len() >= 2 { // At least 2 classes to be considered a pattern
                if let Some(pattern) = self.create_dynamic_pattern(classes) {
                    dynamic_patterns.push(pattern);
                }
            }
        }
        
        dynamic_patterns
    }

    /* ========================================================================================== */
    pub fn find_pattern_usage(&self, content: &str, pattern: &DynamicPattern) -> bool {
        // Search for various forms of the pattern
        let search_patterns = vec![
        format!(r"{}\$\{{[^}}]*\}}{}", regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)), // template literal
        format!(r"{}\{{[^}}]*\}}{}", regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)), // string interpolation
        format!(r"{}['`][^'`]*['`]{}", regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)), // template strings
        format!(r#"["'`]{}\$\{{.*?\}}{}["'`]"#, regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)), // variable interpolation
    ];
        
        for search_pattern in search_patterns {
            if let Ok(regex) = Regex::new(&search_pattern) {
                if regex.is_match(content) {
                    return true;
                }
            }
        }
        
        // Also check for direct string concatenation patterns
        self.find_string_concatenation_usage(content, pattern)
    }

    /* ========================================================================================== */
    fn extract_pattern_key(&self, class_name: &str) -> Option<String> {
        // Look for common separators and extract potential pattern
        let separators = ['-', '_'];
        
        for separator in separators {
            if let Some(sep_pos) = class_name.find(separator) {
                let prefix = &class_name[..sep_pos + 1]; // Include separator
                
                // Count total separators to determine pattern type
                let separator_count = class_name.matches(separator).count();
                
                if separator_count > 1 {
                    // Multiple separators, could be prefix-variable-suffix
                    let last_sep = class_name.rfind(separator).unwrap();
                    if last_sep != sep_pos {
                        let suffix = &class_name[last_sep..];
                        return Some(format!("{}{}", prefix, suffix));
                    }
                }
                
                // Single separator or all separators at same position, just use prefix
                return Some(prefix.to_string());
            }
        }
        
        None
    }

    /* ========================================================================================== */
    fn create_dynamic_pattern(&self, classes: Vec<String>) -> Option<DynamicPattern> {
        // Find common prefix and suffix
        let first_class = &classes[0];
        let mut prefix = String::new();
        let mut suffix = String::new();
        
        // Find common prefix
        for (i, ch) in first_class.chars().enumerate() {
            if classes.iter().all(|class| class.chars().nth(i) == Some(ch)) {
                prefix.push(ch);
            } else {
                break;
            }
        }
        
        // Find common suffix
        let first_chars: Vec<char> = first_class.chars().collect();
        for i in 1..=first_chars.len() {
            let ch = first_chars[first_chars.len() - i];
            if classes.iter().all(|class| {
                let class_chars: Vec<char> = class.chars().collect();
                class_chars.len() >= i && class_chars[class_chars.len() - i] == ch
            }) {
                suffix.insert(0, ch);
            } else {
                break;
            }
        }
        
        // Only create pattern if we have a meaningful prefix
        if prefix.len() >= 2 {
            let pattern = if suffix.is_empty() {
                format!("{}*", prefix)
            } else {
                format!("{}*{}", prefix, suffix)
            };
            
            Some(DynamicPattern {
                prefix,
                suffix,
                pattern,
                matching_classes: classes,
            })
        } else {
            None
        }
    }

    /* ========================================================================================== */
    fn find_string_concatenation_usage(&self, content: &str, pattern: &DynamicPattern) -> bool {
        // Look for patterns like: "type-" + variable + suffix
        let concat_patterns = vec![
            format!(r#"["'`]{}\$\{{[^}}]*\}}{}["'`]"#, regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)),
            format!(r#"["'`]{}["'`]\s*\+\s*\w+\s*\+\s*["'`]{}["'`]"#, regex::escape(&pattern.prefix), regex::escape(&pattern.suffix)),
            format!(r#"["'`]{}["'`]\s*\+\s*\w+"#, regex::escape(&pattern.prefix)),
        ];
        
        for concat_pattern in concat_patterns {
            if let Ok(regex) = Regex::new(&concat_pattern) {
                if regex.is_match(content) {
                    return true;
                }
            }
        }
        
        false
    }

    /* ========================================================================================== */
    fn is_ignored_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty()
    }
}