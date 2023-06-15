pub struct SourceLoc {
    file: String,
    line: u32,
    col: u32,
}

pub struct GeneratedLoc {}

pub struct LinkedFiles {}

pub struct Diagnostic {
    source_loc: SourceLoc,
    generated_loc: Option<GeneratedLoc>,
    level: Level,
}

pub enum Level {
    Info,
    Warning,
    Error,
}
