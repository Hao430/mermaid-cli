pub struct Fixer;

impl Fixer {
    pub fn new() -> Self {
        Fixer
    }

    pub fn fix(&self, code: &str) -> (String, Vec<String>) {
        let mut fixed = code.to_string();
        let mut fixes = Vec::new();

        // === Universal keyword fix (all diagram types, 50+ patterns) ===
        let all_typos: Vec<(&str, &str)> = vec![
            // Flowchart keywords
            ("grpah", "graph"), ("grahp", "graph"), ("garph", "graph"),
            ("flochart", "flowchart"), ("flowchat", "flowchart"), ("flowhcart", "flowchart"),
            ("flowchrat", "flowchart"), ("flochrat", "flowchart"),
            ("subgrah", "subgraph"), ("subgrap", "subgraph"), ("subgrahp", "subgraph"),
            // Sequence keywords
            ("sequnceDiagram", "sequenceDiagram"),
            ("sequanceDiagram", "sequenceDiagram"),
            ("seqeunceDiagram", "sequenceDiagram"),
            ("seqenceDiagram", "sequenceDiagram"),
            ("sequeneDiagram", "sequenceDiagram"),
            ("sequecneDiagram", "sequenceDiagram"),
            ("partcipant", "participant"),
            ("paricipant", "participant"),
            ("particpant", "participant"),
            ("partipant", "participant"),
            ("partcipate", "participant"),
            // Class diagram keywords
            ("clasDiagram", "classDiagram"),
            ("clssDiagram", "classDiagram"),
            ("calssDiagram", "classDiagram"),
            ("classdiagram", "classDiagram"),
            // State diagram keywords
            ("stateDiagam", "stateDiagram"),
            ("stateDiagrm", "stateDiagram"),
            ("statediagram", "stateDiagram"),
            ("statDiagram", "stateDiagram"),
            // ER diagram keywords
            ("erdiagram", "erDiagram"),
            ("erDiagrm", "erDiagram"),
            ("er_diagram", "erDiagram"),
            // Gantt keywords
            ("gant", "gantt"), ("gantt", "gantt"),
            ("dateFormat", "dateFormat"), ("dataFormat", "dateFormat"),
            // Pie keywords
            ("pei", "pie"),
            // Mindmap
            ("mindmap", "mindmap"), ("mindMap", "mindmap"),
            // GitGraph
            ("gitgraph", "gitGraph"), ("Gitgraph", "gitGraph"),
            // Timeline
            ("timeling", "timeline"), ("timline", "timeline"),
            // Journey
            ("journy", "journey"), ("journe", "journey"),
            // Kanban
            ("kanban", "kanban"), ("kanbam", "kanban"),
            // Arrow typos
            ("-->>", "-->"), ("==>", "==>"), ("=>", "->"),
            ("-->", "-->"), ("- ->", "->"), ("- ->>", "-->>"),
            // Direction typos
            ("T-D", "TD"), ("L-R", "LR"), ("R-L", "RL"), ("B-T", "BT"),
            ("BT-D", "TD"), ("LR-D", "LR"),
            // Note typos
            ("Note rigth", "Note right"),
            ("Note left", "Note left"),
            ("Note over", "Note over"),
            ("note rigth", "Note right"),
            ("note left", "Note left"),
            // Activation typos
            ("acitvate", "activate"),
            ("deacitvate", "deactivate"),
            ("activte", "activate"),
            ("deactivte", "deactivate"),
            // Pie chart typos
            ("pie chart", "pie"),
            ("piechart", "pie"),
            // Block diagram typos
            ("block-diagram", "block"),
            // Architecture typos
            ("architecure", "architecture"),
            ("archtecture", "architecture"),
            // Requirement typos
            ("requriement", "requirement"),
            ("requirment", "requirement"),
        ];

        for (typo, correct) in &all_typos {
            if fixed.contains(typo) {
                fixed = fixed.replace(typo, correct);
                fixes.push(format!("Fixed typo: {} -> {}", typo, correct));
            }
        }

        // Apply type-specific fixes
        let trimmed = fixed.trim_start();
        let is_sequence = trimmed.starts_with("sequenceDiagram") || trimmed.starts_with("zenuml");
        let is_class = trimmed.starts_with("classDiagram");
        let is_flowchart = trimmed.starts_with("graph") || trimmed.starts_with("flowchart");

        if is_sequence {
            self.fix_sequence(&mut fixed, &mut fixes);
        } else if is_class {
            self.fix_class(&mut fixed, &mut fixes);
        } else if is_flowchart {
            self.fix_flowchart(&mut fixed, &mut fixes);
        } else {
            self.fix_flowchart(&mut fixed, &mut fixes);
        }

        (fixed, fixes)
    }

    fn fix_flowchart(&self, fixed: &mut String, fixes: &mut Vec<String>) {
        // Ensure ends with 'end' for flowchart if not present
        if !fixed.trim().ends_with("end") && !fixed.trim().is_empty()
            && !fixed.trim().ends_with("}\n") && !fixed.trim().ends_with('}')
            && !fixed.contains("end\n")
        {
            fixed.push_str("\nend");
            fixes.push("Added missing 'end'".to_string());
        }
    }

    fn fix_sequence(&self, fixed: &mut String, fixes: &mut Vec<String>) {
        // Fix missing end for block keywords
        let block_keywords = ["alt", "loop", "opt", "par", "else", "critical", "break"];
        let mut open_count = 0;
        for line in fixed.lines() {
            let trimmed = line.trim();
            for kw in &block_keywords {
                if trimmed.starts_with(kw) && (trimmed.len() == kw.len()
                    || trimmed.as_bytes().get(kw.len()) == Some(&b' ')
                    || trimmed.as_bytes().get(kw.len()) == Some(&b':'))
                {
                    open_count += 1;
                }
            }
            if trimmed == "end" {
                open_count -= 1;
            }
        }

        while open_count > 0 {
            fixed.push_str("\nend");
            fixes.push("Added missing 'end' for block".to_string());
            open_count -= 1;
        }
    }

    fn fix_class(&self, _fixed: &mut String, _fixes: &mut Vec<String>) {
        // Class-specific fixes handled in all_typos
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

    // ---- 序列图 Fixer 测试 ----

    #[test]
    fn test_fixer_sequence_typo_sequnce() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("sequnceDiagram\nAlice->Bob: Hello");
        assert!(fixed.contains("sequenceDiagram"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_sequence_typo_partcipant() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("sequenceDiagram\npartcipant Alice\nAlice->Bob: Hello");
        assert!(fixed.contains("participant Alice"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_sequence_missing_end() {
        let fixer = Fixer::new();
        let code = "sequenceDiagram\nAlice->Bob: Hello\nloop retry\nAlice->Bob: ping";
        let (fixed, fixes) = fixer.fix(code);
        assert!(fixed.trim().ends_with("end"), "Should add missing end for loop block");
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_sequence_valid_unchanged() {
        let fixer = Fixer::new();
        let code = "sequenceDiagram\nAlice->Bob: Hello";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.contains("sequenceDiagram"));
        assert!(fixed.contains("Alice->Bob: Hello"));
    }

    #[test]
    fn test_fixer_multiple_sequence_typos() {
        let fixer = Fixer::new();
        let code = "sequnceDiagram\npartcipant Alice\nAlice-->>Bob: Hello";
        let (fixed, fixes) = fixer.fix(code);
        assert!(fixed.contains("sequenceDiagram"), "Should fix sequnceDiagram");
        assert!(fixed.contains("participant"), "Should fix participant");
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_sequance_typo() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("sequanceDiagram\nA->B: test");
        assert!(fixed.contains("sequenceDiagram"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_seqeunce_typo() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("seqeunceDiagram\nA->B: test");
        assert!(fixed.contains("sequenceDiagram"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_loop_structure() {
        let fixer = Fixer::new();
        let code = "sequenceDiagram\nA->B: start\nloop retry\nA->B: try again";
        let (fixed, _fixes) = fixer.fix(code);
        // Should add end for loop
        let end_count = fixed.matches("\nend").count();
        assert!(end_count >= 1, "Should add at least one end");
    }

    #[test]
    fn test_fixer_opt_block_fix() {
        let fixer = Fixer::new();
        let code = "sequenceDiagram\nopt maybe\nA->B: if needed";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.trim().ends_with("end"), "Should add end for opt");
    }

    #[test]
    fn test_fixer_par_block_fix() {
        let fixer = Fixer::new();
        let code = "sequenceDiagram\npar parallel\nA->B: msg1\nA->C: msg2";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.trim().ends_with("end"), "Should add end for par");
    }

    #[test]
    fn test_fixer_flowchart_flowchrat() {
        let fixer = Fixer::new();
        let (fixed, fixes) = fixer.fix("flowchrat TD\nA-->B");
        assert!(fixed.contains("flowchart"), "Should fix flowchrat");
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_fixer_multiple_arrows() {
        let fixer = Fixer::new();
        let code = "graph TD\nA-->>B\nC-->>D\nE=>F";
        let (fixed, fixes) = fixer.fix(code);
        assert!(fixed.contains("-->"), "Should have arrows");
        assert!(fixes.len() >= 2, "Should have multiple arrow fixes");
    }

    #[test]
    fn test_fixer_default_constructor() {
        let fixer: Fixer = Default::default();
        let (fixed, _fixes) = fixer.fix("grpah TD\nA-->B");
        assert!(fixed.contains("graph"));
    }

    #[test]
    fn test_fixer_preserves_line_breaks() {
        let fixer = Fixer::new();
        let code = "graph TD\nA-->B\nC-->D\nE-->F";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.contains("\n"), "Should preserve line breaks");
        let lines: Vec<&str> = fixed.lines().collect();
        assert!(lines.len() >= 4, "Should preserve multiple lines");
    }

    #[test]
    fn test_fixer_flowchart_with_comments() {
        let fixer = Fixer::new();
        let code = "graph TD\n%% This is a comment\nA-->B\n%% Another comment\nC-->D";
        let (fixed, fixes) = fixer.fix(code);
        assert!(fixed.contains("%% This is a comment"), "Comments preserved");
        assert!(fixed.contains("%% Another comment"), "Comments preserved");
        assert!(fixed.contains("A-->B"), "Edge preserved");
        assert!(fixed.contains("C-->D"), "Edge preserved");
        assert!(!fixes.is_empty(), "Should add end");
    }

    #[test]
    fn test_fixer_preserves_whitespace_indentation() {
        let fixer = Fixer::new();
        let code = "graph TD\n    A-->B\n    C-->D\n    E-->F";
        let (fixed, _fixes) = fixer.fix(code);
        let lines: Vec<&str> = fixed.lines().collect();
        assert!(lines.len() >= 4, "Should have multiple lines");
        for line in &lines[1..] {
            if line.contains("-->") {
                assert!(line.starts_with("    "), "Indentation preserved in: {:?}", line);
            }
        }
    }

    #[test]
    fn test_fixer_class_diagram_content() {
        let fixer = Fixer::new();
        let code = "classDiagram\nclass Animal {\n+String name\n+int age\n}";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.contains("classDiagram"), "Class diagram keyword preserved");
        assert!(fixed.contains("Animal"), "Class name preserved");
        assert!(fixed.contains("String"), "Member type preserved");
    }

    #[test]
    fn test_fixer_er_diagram_content() {
        let fixer = Fixer::new();
        let code = "erDiagram\nCUSTOMER ||--o{ ORDER : places";
        let (fixed, _fixes) = fixer.fix(code);
        assert!(fixed.contains("erDiagram"), "ER diagram keyword preserved");
        assert!(fixed.contains("CUSTOMER"), "Entity preserved");
        assert!(fixed.contains("ORDER"), "Related entity preserved");
    }
}
