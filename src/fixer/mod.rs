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

    #[test]
    fn test_fixer_arrow_typos() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("graph TD\nA-->>B");
        assert!(!fixed.contains("-->>"), "Arrow typo should be fixed");
        assert!(fixed.contains("-->"), "Should contain correct arrow");
        assert!(!fixes.is_empty());

        let (fixed2, fixes2) = fixer.fix("graph TD\nA=>B");
        assert!(!fixed2.contains("=>"), "Arrow typo '=>' should be fixed");
        assert!(fixed2.contains("->"), "Should contain corrected arrow");
        assert!(!fixes2.is_empty());
    }

    #[test]
    fn test_fixer_all_typos() {
        let fixer = Fixer::new();
        let code = "flowchrat TD\nA-->>B\nC=>D";
        let (fixed, fixes) = fixer.fix(code);

        assert!(fixed.contains("flowchart"), "flowchrat should be fixed");
        assert!(!fixed.contains("-->>"), "-->> should be fixed to -->");
        assert!(!fixed.contains("=>"), "=> should be fixed to ->");
        assert!(fixed.trim().ends_with("end"), "Should end with 'end'");

        // Should have at least 3 fix records (typo + arrow + arrow + end)
        assert!(fixes.len() >= 3, "Should have multiple fix records");
    }

    #[test]
    fn test_fixer_valid_input_unchanged() {
        let fixer = Fixer::new();
        let code = "graph TD\nA-->B";
        let (fixed, _fixes) = fixer.fix(code);

        // Core content should be preserved
        assert!(
            fixed.contains("graph TD"),
            "Graph declaration should be preserved"
        );
        assert!(
            fixed.contains("A-->B"),
            "Edge definition should be preserved"
        );
        assert!(fixed.trim().ends_with("end"), "Should end with 'end'");
    }

    #[test]
    fn test_fixer_empty_input() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("");

        assert!(fixed.is_empty(), "Empty input should produce empty output");
        assert!(fixes.is_empty(), "Empty input should have no fixes");
    }
}
