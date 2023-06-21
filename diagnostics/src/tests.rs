use crate::*;

#[derive(Debug, Clone)]
pub struct TestIssue {
    severity: Severity,
}

impl Display for TestIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.severity)
    }
}

impl Diagnostic for TestIssue {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl From<Severity> for TestIssue {
    fn from(severity: Severity) -> Self {
        Self { severity }
    }
}

#[test]
fn issue_set_counters() {
    let mut issues: IssueSet<TestIssue> = IssueSet::new();
    issues.add(Severity::Info.into());
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert!(!issues.has_error());
    assert!(!issues.has_warning());
    issues.add(Severity::Warning.into());
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 1);
    assert!(!issues.has_error());
    assert!(issues.has_warning());
    issues.add(Severity::Error.into());
    assert_eq!(issues.num_errors(), 1);
    assert_eq!(issues.num_warnings(), 1);
    assert!(issues.has_error());
    assert!(issues.has_warning());
    issues.add(Severity::Warning.into());
    assert_eq!(issues.num_errors(), 1);
    assert_eq!(issues.num_warnings(), 2);
    assert!(issues.has_error());
    assert!(issues.has_warning());
}
