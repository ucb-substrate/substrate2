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

#[test]
fn default_severity_is_warning() {
    assert_eq!(Severity::default(), Severity::Warning);
}

#[test]
fn severity_as_tracing_level() {
    assert_eq!(Severity::Info.as_tracing_level(), tracing::Level::INFO);
    assert_eq!(Severity::Warning.as_tracing_level(), tracing::Level::WARN);
    assert_eq!(Severity::Error.as_tracing_level(), tracing::Level::ERROR);
}

#[test]
fn severity_is_error() {
    assert!(!Severity::Info.is_error());
    assert!(!Severity::Warning.is_error());
    assert!(Severity::Error.is_error());
}

#[test]
fn default_help_is_none() {
    assert!(TestIssue {
        severity: Severity::Warning,
    }
    .help()
    .is_none());
}
