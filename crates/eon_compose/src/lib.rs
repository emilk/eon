//! Experimental composition layer for Eon documents.
//!
//! This crate resolves a small opt-in layer on top of ordinary Eon:
//! root-level `use` imports and immutable `$alias.path` references.

use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use eon::{Map, Value};

/// Result type used by the composition resolver.
pub type Result<T> = std::result::Result<T, Error>;

/// Resolve a source string relative to `root_dir`.
pub fn resolve_str(source: &str, root_dir: impl Into<PathBuf>) -> Result<Value> {
    Resolver::new(root_dir).resolve_str(source)
}

/// Resolve a file and all local file imports it references.
pub fn resolve_file(path: impl AsRef<Path>) -> Result<Value> {
    Resolver::new(".").resolve_file(path)
}

/// A resolver for Eon composition imports and references.
#[derive(Clone, Debug)]
pub struct Resolver {
    root_dir: PathBuf,
}

impl Resolver {
    /// Create a resolver with a root directory for relative root imports.
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    /// Resolve a source string relative to this resolver's root directory.
    pub fn resolve_str(&self, source: &str) -> Result<Value> {
        let source_id = ErrorSource::Inline {
            root_dir: self.root_dir.clone(),
        };
        self.resolve_source_inner(source, &self.root_dir, &mut Vec::new(), source_id)
    }

    /// Resolve a file relative to this resolver's root directory.
    pub fn resolve_file(&self, path: impl AsRef<Path>) -> Result<Value> {
        let path = join_path(&self.root_dir, path.as_ref());
        self.resolve_file_inner(&path, &mut Vec::new())
    }

    fn resolve_file_inner(&self, path: &Path, stack: &mut Vec<PathBuf>) -> Result<Value> {
        let canonical = fs::canonicalize(path).map_err(|err| Error::io(path, &err))?;
        if let Some(position) = stack.iter().position(|entry| entry == &canonical) {
            let mut cycle = stack[position..].to_vec();
            cycle.push(canonical);
            return Err(Error::ImportCycle {
                cycle,
                trace: Vec::new(),
            });
        }

        let source = fs::read_to_string(&canonical).map_err(|err| Error::io(&canonical, &err))?;
        let parent = canonical
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let source_id = ErrorSource::File(canonical.clone());

        stack.push(canonical);
        let resolved = self.resolve_source_inner(&source, &parent, stack, source_id);
        stack.pop();

        resolved
    }

    fn resolve_source_inner(
        &self,
        source: &str,
        current_dir: &Path,
        stack: &mut Vec<PathBuf>,
        source_id: ErrorSource,
    ) -> Result<Value> {
        let placeholder_source = substitute_references(source, |_| Ok("null".to_owned()))
            .map_err(|err| err.with_frame_if_empty(source_id.clone(), "scanning references"))?;
        let placeholder_value = parse_value(&placeholder_source)
            .map_err(|err| err.with_frame(source_id.clone(), "parsing document"))?;
        let imports = collect_imports(&placeholder_value)
            .map_err(|err| err.with_frame(source_id.clone(), "reading root `use` imports"))?;

        let mut imported_values = BTreeMap::<String, Value>::new();
        for (alias, spec) in imports {
            let import_path = join_path(current_dir, Path::new(&spec.file));
            let imported = self
                .resolve_file_inner(&import_path, stack)
                .map_err(|err| err.while_importing(&alias, &spec.file, source_id.clone()))?;
            let selected = select_segments(&imported, &spec.select, &spec.file)
                .map_err(|err| err.while_selecting_import(&alias, &spec, source_id.clone()))?;
            imported_values.insert(alias, selected);
        }

        let resolved_source = substitute_references(source, |reference| {
            let value = resolve_reference(reference, &imported_values)
                .map_err(|err| err.while_resolving_reference(reference, source_id.clone()))?;
            Ok(value.to_string_with_core())
        })
        .map_err(|err| err.with_frame_if_empty(source_id.clone(), "scanning references"))?;
        let value = parse_value(&resolved_source)
            .map_err(|err| err.with_frame(source_id, "parsing resolved document"))?;

        Ok(strip_root_use(value))
    }
}

/// A source attached to a composition error trace frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorSource {
    /// A file source.
    File(PathBuf),
    /// An inline source resolved relative to a root directory.
    Inline {
        /// Root directory used for relative imports.
        root_dir: PathBuf,
    },
}

impl fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => write!(f, "{}", path.display()),
            Self::Inline { root_dir } => {
                write!(f, "<inline source rooted at {}>", root_dir.display())
            }
        }
    }
}

/// One frame in a composition error source chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorTraceFrame {
    /// Source file or inline source where the action happened.
    pub source: ErrorSource,
    /// Human-readable action, such as `importing alias`.
    pub action: String,
}

/// Errors returned by the composition resolver.
#[derive(Debug)]
pub enum Error {
    /// File-system error while reading an imported file.
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying error message.
        message: String,
        /// Source chain accumulated while resolving imports and references.
        trace: Vec<ErrorTraceFrame>,
    },
    /// Eon parse error.
    Parse {
        /// Parse error message.
        message: String,
        /// Source chain accumulated while resolving imports and references.
        trace: Vec<ErrorTraceFrame>,
    },
    /// Invalid root-level `use` declaration.
    InvalidUse {
        /// Validation error message.
        message: String,
        /// Source chain accumulated while resolving imports and references.
        trace: Vec<ErrorTraceFrame>,
    },
    /// Invalid or unresolved `$alias.path` reference.
    InvalidReference {
        /// Reference text.
        reference: String,
        /// Validation error message.
        message: String,
        /// Source chain accumulated while resolving imports and references.
        trace: Vec<ErrorTraceFrame>,
    },
    /// Import cycle.
    ImportCycle {
        /// Canonicalized cycle paths.
        cycle: Vec<PathBuf>,
        /// Source chain accumulated while resolving imports and references.
        trace: Vec<ErrorTraceFrame>,
    },
}

impl Error {
    /// Source chain accumulated while resolving imports and references.
    pub fn trace(&self) -> &[ErrorTraceFrame] {
        match self {
            Self::Io { trace, .. }
            | Self::Parse { trace, .. }
            | Self::InvalidUse { trace, .. }
            | Self::InvalidReference { trace, .. }
            | Self::ImportCycle { trace, .. } => trace,
        }
    }

    fn io(path: &Path, err: &std::io::Error) -> Self {
        Self::Io {
            path: path.to_owned(),
            message: err.to_string(),
            trace: Vec::new(),
        }
    }

    fn parse(err: &eon::Error) -> Self {
        Self::Parse {
            message: err.to_string(),
            trace: Vec::new(),
        }
    }

    fn invalid_use(message: impl Into<String>) -> Self {
        Self::InvalidUse {
            message: message.into(),
            trace: Vec::new(),
        }
    }

    fn invalid_reference(reference: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidReference {
            reference: reference.into(),
            message: message.into(),
            trace: Vec::new(),
        }
    }

    fn trace_mut(&mut self) -> &mut Vec<ErrorTraceFrame> {
        match self {
            Self::Io { trace, .. }
            | Self::Parse { trace, .. }
            | Self::InvalidUse { trace, .. }
            | Self::InvalidReference { trace, .. }
            | Self::ImportCycle { trace, .. } => trace,
        }
    }

    fn with_frame(mut self, source: ErrorSource, action: impl Into<String>) -> Self {
        self.trace_mut().push(ErrorTraceFrame {
            source,
            action: action.into(),
        });
        self
    }

    fn with_frame_if_empty(self, source: ErrorSource, action: impl Into<String>) -> Self {
        if self.trace().is_empty() {
            self.with_frame(source, action)
        } else {
            self
        }
    }

    fn while_importing(self, alias: &str, file: &str, source: ErrorSource) -> Self {
        self.with_frame(source, format!("importing `{alias}` from `{file}`"))
    }

    fn while_selecting_import(self, alias: &str, spec: &ImportSpec, source: ErrorSource) -> Self {
        let action = if spec.select.is_empty() {
            format!("selecting import `{alias}` from `{}`", spec.file)
        } else {
            format!(
                "selecting `{}` from import `{alias}` in `{}`",
                format_path_segments(&spec.select),
                spec.file
            )
        };
        self.with_frame(source, action)
    }

    fn while_resolving_reference(self, reference: &Reference, source: ErrorSource) -> Self {
        self.with_frame(source, format!("resolving reference `{reference}`"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message, .. } => write!(f, "{}: {message}", path.display())?,
            Self::Parse { message, .. } => message.fmt(f)?,
            Self::InvalidUse { message, .. } => {
                write!(f, "invalid use declaration: {message}")?;
            }
            Self::InvalidReference {
                reference, message, ..
            } => {
                write!(f, "invalid reference `{reference}`: {message}")?;
            }
            Self::ImportCycle { cycle, .. } => {
                write!(f, "import cycle")?;
                for path in cycle {
                    write!(f, " -> {}", path.display())?;
                }
            }
        }

        for frame in self.trace() {
            write!(f, "\n  while {} in {}", frame.action, frame.source)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug)]
struct ImportSpec {
    file: String,
    select: Vec<PathSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Reference {
    alias: String,
    segments: Vec<PathSegment>,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.alias)?;
        for segment in &self.segments {
            match segment {
                PathSegment::Field(field) => write!(f, ".{field}")?,
                PathSegment::Index(index) => write!(f, "[{index}]")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PathSegment {
    Field(String),
    Index(usize),
}

fn format_path_segments(segments: &[PathSegment]) -> String {
    let mut out = String::new();
    for segment in segments {
        match segment {
            PathSegment::Field(field) => {
                out.push('.');
                out.push_str(field);
            }
            PathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    if out.is_empty() { ".".to_owned() } else { out }
}

fn join_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

fn parse_value(source: &str) -> Result<Value> {
    Value::from_str_with_core(source).map_err(|err| Error::parse(&err))
}

fn collect_imports(root: &Value) -> Result<BTreeMap<String, ImportSpec>> {
    let Some(root_map) = root.as_map() else {
        return Ok(BTreeMap::new());
    };
    let Some(use_value) = root_map.get_str("use") else {
        return Ok(BTreeMap::new());
    };
    let Some(use_map) = use_value.as_map() else {
        return Err(Error::invalid_use("root `use` value must be a map"));
    };

    let mut imports = BTreeMap::new();
    for (alias, value) in use_map {
        let Some(alias) = alias.as_string() else {
            return Err(Error::invalid_use(
                "import aliases must be string-like keys",
            ));
        };
        if imports
            .insert(alias.to_owned(), import_spec_from_value(value)?)
            .is_some()
        {
            return Err(Error::invalid_use(format!(
                "duplicate import alias `{alias}`"
            )));
        }
    }

    Ok(imports)
}

fn import_spec_from_value(value: &Value) -> Result<ImportSpec> {
    if let Some(file) = value.as_string() {
        return Ok(ImportSpec {
            file: file.to_owned(),
            select: Vec::new(),
        });
    }

    let Some(map) = value.as_map() else {
        return Err(Error::invalid_use(
            "import values must be a string path or a { from, select } map",
        ));
    };

    let file = map
        .get_str("from")
        .and_then(Value::as_string)
        .ok_or_else(|| Error::invalid_use("selected imports require a string `from` field"))?
        .to_owned();
    let select = map
        .get_str("select")
        .map(|value| {
            value
                .as_string()
                .ok_or_else(|| Error::invalid_use("`select` must be a string path"))
                .and_then(parse_select_path)
        })
        .transpose()?
        .unwrap_or_default();

    for (key, _) in map {
        let Some(key) = key.as_string() else {
            return Err(Error::invalid_use(
                "selected import spec keys must be string-like",
            ));
        };
        if key != "from" && key != "select" {
            return Err(Error::invalid_use(format!(
                "unknown selected import field `{key}`"
            )));
        }
    }

    Ok(ImportSpec { file, select })
}

fn parse_select_path(path: &str) -> Result<Vec<PathSegment>> {
    let mut pos = 0;
    let mut segments = Vec::new();
    parse_path_tail(path.trim(), &mut pos, &mut segments, true)?;
    if pos != path.trim().len() {
        return Err(Error::invalid_use(format!("invalid select path `{path}`")));
    }
    Ok(segments)
}

fn substitute_references<F>(source: &str, mut resolve: F) -> Result<String>
where
    F: FnMut(&Reference) -> Result<String>,
{
    let mut out = String::with_capacity(source.len());
    let mut pos = 0;

    while pos < source.len() {
        if source[pos..].starts_with("//") {
            let start = pos;
            pos += 2;
            while pos < source.len() && source.as_bytes()[pos] != b'\n' {
                pos += 1;
            }
            if pos < source.len() {
                pos += 1;
            }
            out.push_str(&source[start..pos]);
        } else if source[pos..].starts_with("\"\"\"") {
            let end = find_string_end(source, pos, "\"\"\"");
            out.push_str(&source[pos..end]);
            pos = end;
        } else if source[pos..].starts_with("'''") {
            let end = find_string_end(source, pos, "'''");
            out.push_str(&source[pos..end]);
            pos = end;
        } else if source.as_bytes()[pos] == b'"' {
            let end = find_basic_string_end(source, pos);
            out.push_str(&source[pos..end]);
            pos = end;
        } else if source.as_bytes()[pos] == b'\'' {
            let end = find_literal_string_end(source, pos);
            out.push_str(&source[pos..end]);
            pos = end;
        } else if source.as_bytes()[pos] == b'$' {
            let (end, reference) = parse_reference_at(source, pos)?;
            out.push_str(&resolve(&reference)?);
            pos = end;
        } else {
            let chr = source[pos..]
                .chars()
                .next()
                .expect("pos is within the source string");
            out.push(chr);
            pos += chr.len_utf8();
        }
    }

    Ok(out)
}

fn find_string_end(source: &str, start: usize, delimiter: &str) -> usize {
    let mut pos = start + delimiter.len();
    while pos < source.len() {
        if source[pos..].starts_with(delimiter) {
            return pos + delimiter.len();
        }
        pos += 1;
    }
    source.len()
}

fn find_basic_string_end(source: &str, start: usize) -> usize {
    let mut pos = start + 1;
    while pos < source.len() {
        match source.as_bytes()[pos] {
            b'\\' => {
                pos += 1;
                if pos < source.len() {
                    pos += 1;
                }
            }
            b'"' => return pos + 1,
            _ => pos += 1,
        }
    }
    source.len()
}

fn find_literal_string_end(source: &str, start: usize) -> usize {
    let mut pos = start + 1;
    while pos < source.len() {
        if source.as_bytes()[pos] == b'\'' {
            return pos + 1;
        }
        pos += 1;
    }
    source.len()
}

fn parse_reference_at(source: &str, start: usize) -> Result<(usize, Reference)> {
    let mut pos = start + 1;
    let alias = parse_identifier(source, &mut pos).ok_or_else(|| {
        Error::invalid_reference("$", format!("expected identifier after byte {start}"))
    })?;
    let mut segments = Vec::new();
    parse_path_tail(source, &mut pos, &mut segments, false)?;

    Ok((pos, Reference { alias, segments }))
}

fn parse_path_tail(
    source: &str,
    pos: &mut usize,
    segments: &mut Vec<PathSegment>,
    allow_initial_field: bool,
) -> Result<()> {
    if allow_initial_field {
        if let Some(field) = parse_identifier(source, pos) {
            segments.push(PathSegment::Field(field));
        }
    }

    while *pos < source.len() {
        match source.as_bytes()[*pos] {
            b'.' => {
                *pos += 1;
                let field = parse_identifier(source, pos)
                    .ok_or_else(|| Error::invalid_reference(source, "expected field after `.`"))?;
                segments.push(PathSegment::Field(field));
            }
            b'[' => {
                *pos += 1;
                let start = *pos;
                while *pos < source.len() && source.as_bytes()[*pos].is_ascii_digit() {
                    *pos += 1;
                }
                if start == *pos || *pos >= source.len() || source.as_bytes()[*pos] != b']' {
                    return Err(Error::invalid_reference(source, "invalid list index"));
                }
                let index = source[start..*pos]
                    .parse::<usize>()
                    .map_err(|err| Error::invalid_reference(source, err.to_string()))?;
                *pos += 1;
                segments.push(PathSegment::Index(index));
            }
            _ => break,
        }
    }

    Ok(())
}

fn parse_identifier(source: &str, pos: &mut usize) -> Option<String> {
    if *pos >= source.len() || !is_identifier_start(source.as_bytes()[*pos]) {
        return None;
    }
    let start = *pos;
    *pos += 1;
    while *pos < source.len() && is_identifier_continue(source.as_bytes()[*pos]) {
        *pos += 1;
    }
    Some(source[start..*pos].to_owned())
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn resolve_reference(reference: &Reference, imports: &BTreeMap<String, Value>) -> Result<Value> {
    let Some(imported) = imports.get(&reference.alias) else {
        return Err(Error::invalid_reference(
            reference.to_string(),
            format!("unknown import alias `{}`", reference.alias),
        ));
    };

    select_segments(imported, &reference.segments, &reference.to_string())
}

fn select_segments(value: &Value, segments: &[PathSegment], label: &str) -> Result<Value> {
    let mut current = value;
    for segment in segments {
        match segment {
            PathSegment::Field(field) => {
                let Some(map) = current.as_map() else {
                    return Err(Error::invalid_reference(
                        label,
                        format!("cannot access field `{field}` on non-map value"),
                    ));
                };
                current = map.get_str(field).ok_or_else(|| {
                    Error::invalid_reference(label, format!("missing field `{field}`"))
                })?;
            }
            PathSegment::Index(index) => {
                let Some(list) = current.as_list() else {
                    return Err(Error::invalid_reference(
                        label,
                        format!("cannot index `{index}` on non-list value"),
                    ));
                };
                current = list.get(*index).ok_or_else(|| {
                    Error::invalid_reference(label, format!("missing list index `{index}`"))
                })?;
            }
        }
    }

    Ok(current.clone())
}

fn strip_root_use(value: Value) -> Value {
    let Value::Map(map) = value else {
        return value;
    };

    let mut out = Map::with_capacity(map.len());
    for (key, value) in map {
        if !matches!(&key, Value::String(key) if key == "use") {
            out.insert(key, value);
        }
    }

    Value::Map(out)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use eon::Value;

    use super::{Error, Resolver};

    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/eon_compose_tests")
            .join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolves_imported_reference_paths() {
        let dir = test_dir("resolves_imported_reference_paths");
        fs::write(
            dir.join("common.eon"),
            r#"
database: {
    host: "localhost"
    port: 5432
}
"#,
        )
        .unwrap();

        let source = r#"
use: {
    common: "common.eon"
}

host: $common.database.host
port: $common.database.port
"#;

        let resolved = Resolver::new(&dir).resolve_str(source).unwrap();
        let map = resolved.as_map().unwrap();

        assert!(map.get_str("use").is_none());
        assert_eq!(
            map.get_str("host").and_then(Value::as_string),
            Some("localhost")
        );
        assert_eq!(map.get_str("port"), Some(&Value::from(5432)));
    }

    #[test]
    fn resolves_selected_imports() {
        let dir = test_dir("resolves_selected_imports");
        fs::write(
            dir.join("app.eon"),
            r#"
app_config: {
    key: "secret"
}
"#,
        )
        .unwrap();

        let source = r#"
use: {
    app_key: {
        from: "app.eon"
        select: ".app_config.key"
    }
}

active_key: $app_key
"#;

        let resolved = Resolver::new(&dir).resolve_str(source).unwrap();
        let map = resolved.as_map().unwrap();

        assert_eq!(
            map.get_str("active_key").and_then(Value::as_string),
            Some("secret")
        );
    }

    #[test]
    fn leaves_references_in_strings_and_comments_alone() {
        let dir = test_dir("leaves_references_in_strings_and_comments_alone");
        fs::write(dir.join("common.eon"), "value: \"resolved\"\n").unwrap();

        let source = r#"
use: {
    common: "common.eon"
}

// $common.value should stay in a comment
literal: '$common.value'
actual: $common.value
"#;

        let resolved = Resolver::new(&dir).resolve_str(source).unwrap();
        let map = resolved.as_map().unwrap();

        assert_eq!(
            map.get_str("literal").and_then(Value::as_string),
            Some("$common.value")
        );
        assert_eq!(
            map.get_str("actual").and_then(Value::as_string),
            Some("resolved")
        );
    }

    #[test]
    fn reports_import_cycles() {
        let dir = test_dir("reports_import_cycles");
        fs::write(
            dir.join("a.eon"),
            r#"
use: { b: "b.eon" }
value: $b.value
"#,
        )
        .unwrap();
        fs::write(
            dir.join("b.eon"),
            r#"
use: { a: "a.eon" }
value: $a.value
"#,
        )
        .unwrap();

        let err = Resolver::new(&dir).resolve_file("a.eon").unwrap_err();

        assert!(matches!(err, Error::ImportCycle { .. }));
    }

    #[test]
    fn traces_nested_reference_errors() {
        let dir = test_dir("traces_nested_reference_errors");
        fs::write(dir.join("leaf.eon"), "actual: 42\n").unwrap();
        fs::write(
            dir.join("middle.eon"),
            r#"
use: { leaf: "leaf.eon" }
value: $leaf.missing
"#,
        )
        .unwrap();
        fs::write(
            dir.join("root.eon"),
            r#"
use: { middle: "middle.eon" }
value: $middle.value
"#,
        )
        .unwrap();

        let err = Resolver::new(&dir).resolve_file("root.eon").unwrap_err();
        let rendered = err.to_string();

        assert!(rendered.contains("missing field `missing`"), "{rendered}");
        assert!(
            rendered.contains("resolving reference `$leaf.missing`"),
            "{rendered}"
        );
        assert!(
            rendered.contains("importing `middle` from `middle.eon`"),
            "{rendered}"
        );
        assert_eq!(err.trace().len(), 2);
    }

    #[test]
    fn traces_selected_import_errors() {
        let dir = test_dir("traces_selected_import_errors");
        fs::write(dir.join("app.eon"), "app_config: {}\n").unwrap();

        let source = r#"
use: {
    app_key: {
        from: "app.eon"
        select: ".app_config.key"
    }
}

value: $app_key
"#;

        let err = Resolver::new(&dir).resolve_str(source).unwrap_err();
        let rendered = err.to_string();

        assert!(rendered.contains("missing field `key`"), "{rendered}");
        assert!(
            rendered.contains("selecting `.app_config.key` from import `app_key`"),
            "{rendered}"
        );
        assert_eq!(err.trace().len(), 1);
    }
}
