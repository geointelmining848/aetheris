use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Language identifier mapped to `iced::highlighter` (`syntect`) file extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    PlainText,
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Json,
    Toml,
    Yaml,
    Xml,
    Html,
    Css,
    Markdown,
    Sql,
    Shell,
    PowerShell,
    Batch,
    C,
    Cpp,
    CSharp,
    Java,
    Go,
    Swift,
    Kotlin,
    Ruby,
    Php,
    Lua,
    R,
    Perl,
    Haskell,
    Zig,
    Dockerfile,
    Makefile,
    Ini,
    Csv,
    Diff,
    LaTeX,
    GraphQl,
    Protobuf,
    Nix,
    Regex,
}

impl Language {
    pub const ALL: &'static [Language] = &[
        Language::PlainText,
        Language::Rust,
        Language::Python,
        Language::JavaScript,
        Language::TypeScript,
        Language::Json,
        Language::Toml,
        Language::Yaml,
        Language::Xml,
        Language::Html,
        Language::Css,
        Language::Markdown,
        Language::Sql,
        Language::Shell,
        Language::PowerShell,
        Language::Batch,
        Language::C,
        Language::Cpp,
        Language::CSharp,
        Language::Java,
        Language::Go,
        Language::Swift,
        Language::Kotlin,
        Language::Ruby,
        Language::Php,
        Language::Lua,
        Language::R,
        Language::Perl,
        Language::Haskell,
        Language::Zig,
        Language::Dockerfile,
        Language::Makefile,
        Language::Ini,
        Language::Csv,
        Language::Diff,
        Language::LaTeX,
        Language::GraphQl,
        Language::Protobuf,
        Language::Nix,
        Language::Regex,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Language::PlainText => "Plain Text",
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Json => "JSON",
            Language::Toml => "TOML",
            Language::Yaml => "YAML",
            Language::Xml => "XML",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Markdown => "Markdown",
            Language::Sql => "SQL",
            Language::Shell => "Shell / Bash",
            Language::PowerShell => "PowerShell",
            Language::Batch => "Windows Batch",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Java => "Java",
            Language::Go => "Go",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Lua => "Lua",
            Language::R => "R",
            Language::Perl => "Perl",
            Language::Haskell => "Haskell",
            Language::Zig => "Zig",
            Language::Dockerfile => "Dockerfile",
            Language::Makefile => "Makefile",
            Language::Ini => "INI / Config",
            Language::Csv => "CSV / TSV",
            Language::Diff => "Diff / Patch",
            Language::LaTeX => "LaTeX",
            Language::GraphQl => "GraphQL",
            Language::Protobuf => "Protobuf",
            Language::Nix => "Nix",
            Language::Regex => "Regex",
        }
    }

    /// Extension token passed to `iced::highlighter::Highlighter`.
    pub fn syntax_extension(self) -> &'static str {
        match self {
            Language::PlainText => "txt",
            Language::Rust => "rs",
            Language::Python => "py",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Json => "json",
            Language::Toml => "toml",
            Language::Yaml => "yaml",
            Language::Xml => "xml",
            Language::Html => "html",
            Language::Css => "css",
            Language::Markdown => "md",
            Language::Sql => "sql",
            Language::Shell => "sh",
            Language::PowerShell => "ps1",
            Language::Batch => "bat",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::CSharp => "cs",
            Language::Java => "java",
            Language::Go => "go",
            Language::Swift => "swift",
            Language::Kotlin => "kt",
            Language::Ruby => "rb",
            Language::Php => "php",
            Language::Lua => "lua",
            Language::R => "r",
            Language::Perl => "pl",
            Language::Haskell => "hs",
            Language::Zig => "zig",
            Language::Dockerfile => "dockerfile",
            Language::Makefile => "makefile",
            Language::Ini => "ini",
            Language::Csv => "csv",
            Language::Diff => "diff",
            Language::LaTeX => "tex",
            Language::GraphQl => "graphql",
            Language::Protobuf => "proto",
            Language::Nix => "nix",
            Language::Regex => "regex",
        }
    }

    pub fn detect_from_path(path: &Path) -> Self {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "dockerfile" => return Language::Dockerfile,
                "makefile" | "gnumakefile" => return Language::Makefile,
                "cargo.lock" => return Language::Toml,
                ".bashrc" | ".zshrc" | ".profile" | ".bash_profile" => return Language::Shell,
                ".gitignore" | ".env" | ".editorconfig" => return Language::Ini,
                _ => {}
            }
        }

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return Language::PlainText;
        };

        match ext.to_ascii_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" | "pyw" | "pyi" => Language::Python,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => Language::TypeScript,
            "json" | "jsonc" | "jsonl" | "geojson" => Language::Json,
            "toml" => Language::Toml,
            "yml" | "yaml" => Language::Yaml,
            "xml" | "svg" | "xhtml" | "plist" | "csproj" => Language::Xml,
            "html" | "htm" => Language::Html,
            "css" | "scss" | "sass" | "less" => Language::Css,
            "md" | "markdown" | "mdx" => Language::Markdown,
            "sql" => Language::Sql,
            "sh" | "bash" | "zsh" | "fish" => Language::Shell,
            "ps1" | "psm1" | "psd1" => Language::PowerShell,
            "bat" | "cmd" => Language::Batch,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Language::Cpp,
            "cs" => Language::CSharp,
            "java" => Language::Java,
            "go" => Language::Go,
            "swift" => Language::Swift,
            "kt" | "kts" => Language::Kotlin,
            "rb" | "gemspec" => Language::Ruby,
            "php" => Language::Php,
            "lua" => Language::Lua,
            "r" | "rmd" => Language::R,
            "pl" | "pm" => Language::Perl,
            "hs" => Language::Haskell,
            "zig" => Language::Zig,
            "ini" | "cfg" | "conf" | "properties" => Language::Ini,
            "csv" | "tsv" => Language::Csv,
            "diff" | "patch" => Language::Diff,
            "tex" | "sty" | "cls" | "bib" => Language::LaTeX,
            "graphql" | "gql" => Language::GraphQl,
            "proto" => Language::Protobuf,
            "nix" => Language::Nix,
            _ => Language::PlainText,
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&l| l == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_extensions_and_special_filenames() {
        assert_eq!(
            Language::detect_from_path(Path::new("main.rs")),
            Language::Rust
        );
        assert_eq!(
            Language::detect_from_path(Path::new("Dockerfile")),
            Language::Dockerfile
        );
        assert_eq!(
            Language::detect_from_path(Path::new("Makefile")),
            Language::Makefile
        );
        assert_eq!(
            Language::detect_from_path(Path::new("data.tsv")),
            Language::Csv
        );
        assert_eq!(
            Language::detect_from_path(Path::new("README.md")),
            Language::Markdown
        );
        assert_eq!(
            Language::detect_from_path(Path::new("unknown.xyz")),
            Language::PlainText
        );
    }
}
