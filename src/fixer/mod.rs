pub struct Fixer;

impl Fixer {
    pub fn new() -> Self {
        Fixer
    }

    pub fn fix(&self, code: &str) -> (String, Vec<String>) {
        let mut fixed = code.to_string();
        let mut fixes = Vec::new();

        // 修复常见拼写错误
        let typos = vec![
            ("grpah", "graph"),
            ("flowchrat", "flowchart"),
            ("-->>", "-->"),
            ("=>", "->"),
        ];

        for (typo, correct) in typos {
            if fixed.contains(typo) {
                fixed = fixed.replace(typo, correct);
                fixes.push(format!("Fixed typo: {} -> {}", typo, correct));
            }
        }

        // 确保以 end 结尾（如果需要）
        if !fixed.trim().ends_with("end") && !fixed.trim().is_empty() {
            fixed.push_str("\nend");
            fixes.push("Added missing 'end'".to_string());
        }

        (fixed, fixes)
    }
}

impl Default for Fixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixer_typo() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("grpah TD\nA-->B");
        assert!(fixed.contains("graph"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_missing_end() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("graph TD\nA-->B");
        assert!(fixed.trim().ends_with("end"));
        assert!(!fixes.is_empty());
    }
}
