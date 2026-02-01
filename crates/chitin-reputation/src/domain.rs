// crates/chitin-reputation/src/domain.rs
//
// Domain/topic classification for context-scoped trust in the Chitin Protocol.

use serde::{Deserialize, Serialize};

/// A domain context identifying a Reef Zone topic area.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainContext {
    /// Unique identifier for this domain (e.g., "medical", "code/python").
    pub domain_id: String,
    /// Human-readable name of the domain.
    pub name: String,
}

/// A rule mapping keywords to a domain.
#[derive(Debug, Clone)]
struct DomainRule {
    domain: DomainContext,
    keywords: Vec<String>,
}

/// Classifies text content into domain contexts using keyword matching.
///
/// Used to determine which Reef Zone a Polyp belongs to,
/// and to scope trust computations to relevant domains.
#[derive(Debug)]
pub struct DomainClassifier {
    /// Domain rules with keyword lists.
    domains: Vec<DomainRule>,
}

impl DomainClassifier {
    /// Create a new DomainClassifier with default domain rules.
    pub fn new() -> Self {
        let domains = vec![
            DomainRule {
                domain: DomainContext {
                    domain_id: "medical".to_string(),
                    name: "Medical & Health".to_string(),
                },
                keywords: vec![
                    "patient".to_string(),
                    "diagnosis".to_string(),
                    "treatment".to_string(),
                    "clinical".to_string(),
                    "medical".to_string(),
                    "disease".to_string(),
                    "symptom".to_string(),
                    "therapy".to_string(),
                    "pharmaceutical".to_string(),
                    "healthcare".to_string(),
                ],
            },
            DomainRule {
                domain: DomainContext {
                    domain_id: "code/python".to_string(),
                    name: "Python Programming".to_string(),
                },
                keywords: vec![
                    "python".to_string(),
                    "def ".to_string(),
                    "import ".to_string(),
                    "pip ".to_string(),
                    "django".to_string(),
                    "flask".to_string(),
                    "numpy".to_string(),
                    "pandas".to_string(),
                    "__init__".to_string(),
                    "pytest".to_string(),
                ],
            },
            DomainRule {
                domain: DomainContext {
                    domain_id: "code/rust".to_string(),
                    name: "Rust Programming".to_string(),
                },
                keywords: vec![
                    "rust".to_string(),
                    "fn ".to_string(),
                    "impl ".to_string(),
                    "cargo".to_string(),
                    "crate".to_string(),
                    "struct ".to_string(),
                    "trait ".to_string(),
                    "tokio".to_string(),
                    "async ".to_string(),
                    "lifetime".to_string(),
                ],
            },
            DomainRule {
                domain: DomainContext {
                    domain_id: "science".to_string(),
                    name: "Science & Research".to_string(),
                },
                keywords: vec![
                    "hypothesis".to_string(),
                    "experiment".to_string(),
                    "research".to_string(),
                    "scientific".to_string(),
                    "molecule".to_string(),
                    "physics".to_string(),
                    "chemistry".to_string(),
                    "biology".to_string(),
                    "laboratory".to_string(),
                    "quantum".to_string(),
                ],
            },
            DomainRule {
                domain: DomainContext {
                    domain_id: "finance".to_string(),
                    name: "Finance & Economics".to_string(),
                },
                keywords: vec![
                    "investment".to_string(),
                    "portfolio".to_string(),
                    "dividend".to_string(),
                    "equity".to_string(),
                    "financial".to_string(),
                    "market".to_string(),
                    "banking".to_string(),
                    "interest rate".to_string(),
                    "stock".to_string(),
                    "bond".to_string(),
                ],
            },
            DomainRule {
                domain: DomainContext {
                    domain_id: "legal".to_string(),
                    name: "Legal & Compliance".to_string(),
                },
                keywords: vec![
                    "contract".to_string(),
                    "statute".to_string(),
                    "jurisdiction".to_string(),
                    "plaintiff".to_string(),
                    "defendant".to_string(),
                    "litigation".to_string(),
                    "legal".to_string(),
                    "court".to_string(),
                    "attorney".to_string(),
                    "regulation".to_string(),
                ],
            },
        ];

        Self { domains }
    }

    /// Classify a text string into a domain context.
    ///
    /// Lowercases the text and counts keyword matches per domain.
    /// Returns the highest-scoring domain, or None if no keywords match.
    pub fn classify(&self, text: &str) -> Option<DomainContext> {
        if text.is_empty() {
            return None;
        }

        let lower = text.to_lowercase();
        let mut best_domain: Option<&DomainContext> = None;
        let mut best_score = 0usize;

        for rule in &self.domains {
            let score: usize = rule
                .keywords
                .iter()
                .filter(|kw| lower.contains(kw.as_str()))
                .count();

            if score > best_score {
                best_score = score;
                best_domain = Some(&rule.domain);
            }
        }

        if best_score == 0 {
            return None;
        }

        best_domain.cloned()
    }
}

impl Default for DomainClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medical_text_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify(
            "The patient showed symptoms of the disease and required clinical treatment",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "medical");
    }

    #[test]
    fn python_code_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify("import numpy as np\ndef process(data):\n    return pandas.DataFrame(data)");
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "code/python");
    }

    #[test]
    fn rust_code_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify(
            "fn main() { let x = impl struct trait cargo tokio async }",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "code/rust");
    }

    #[test]
    fn unknown_text_returns_none() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify("xyzzy plugh nothing to see here");
        assert!(result.is_none());
    }

    #[test]
    fn empty_string_returns_none() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify("");
        assert!(result.is_none());
    }

    #[test]
    fn finance_text_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify(
            "The investment portfolio showed strong dividend growth in the financial market",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "finance");
    }

    #[test]
    fn legal_text_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify(
            "The plaintiff filed litigation in court against the defendant",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "legal");
    }

    #[test]
    fn science_text_classified() {
        let classifier = DomainClassifier::new();
        let result = classifier.classify(
            "The experiment tested the hypothesis in the laboratory using quantum physics",
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().domain_id, "science");
    }
}
