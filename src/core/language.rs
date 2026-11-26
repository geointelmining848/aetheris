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

    /// Infer programming/markup language directly from live buffer content (no file extension required).
    /// Returns `None` for plain prose or ambiguous short input so normal writing is never falsely highlighted.
    pub fn detect_from_content(text: &str) -> Option<Self> {
        let trimmed = text.trim();
        if trimmed.len() < 4 {
            return None;
        }

        // 1. Shebang detection (instant, exact)
        if let Some(first_line) = trimmed.lines().next()
            && let Some(shebang) = first_line.strip_prefix("#!")
        {
            let lower = shebang.to_ascii_lowercase();
            if lower.contains("python") {
                return Some(Language::Python);
            }
            if lower.contains("node") || lower.contains("deno") || lower.contains("bun") {
                return Some(Language::JavaScript);
            }
            if lower.contains("bash")
                || lower.contains("/sh")
                || lower.contains("zsh")
                || lower.contains("fish")
                || lower.ends_with(" sh")
            {
                return Some(Language::Shell);
            }
            if lower.contains("ruby") {
                return Some(Language::Ruby);
            }
            if lower.contains("perl") {
                return Some(Language::Perl);
            }
            if lower.contains("php") {
                return Some(Language::Php);
            }
            if lower.contains("pwsh") || lower.contains("powershell") {
                return Some(Language::PowerShell);
            }
            if lower.contains("lua") {
                return Some(Language::Lua);
            }
        }

        // 2. Unambiguous structural headers
        let lower_head: String = trimmed
            .chars()
            .take(2048)
            .collect::<String>()
            .to_ascii_lowercase();
        if trimmed.starts_with("<?php") {
            return Some(Language::Php);
        }
        if lower_head.starts_with("<!doctype html") || lower_head.starts_with("<html") {
            return Some(Language::Html);
        }
        if trimmed.starts_with("<?xml") {
            return Some(Language::Xml);
        }
        if trimmed.starts_with("diff --git ")
            || (trimmed.contains("\n--- ")
                && trimmed.contains("\n+++ ")
                && trimmed.contains("\n@@ "))
        {
            return Some(Language::Diff);
        }
        if trimmed.contains("\\documentclass") || trimmed.contains("\\begin{document}") {
            return Some(Language::LaTeX);
        }

        // Valid or clearly structured JSON object/array
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && trimmed.contains('"')
            && (serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
                || (trimmed.starts_with('{') && trimmed.contains("\":")))
        {
            return Some(Language::Json);
        }

        // Inspect up to first 120 non-empty lines for fast live typing performance
        let sample_lines: Vec<&str> = trimmed
            .lines()
            .filter(|l| !l.trim().is_empty())
            .take(120)
            .collect();
        let sample = sample_lines.join("\n");
        let sample_upper = sample.to_ascii_uppercase();

        // Dockerfile check
        if sample_lines
            .first()
            .is_some_and(|l| l.trim_start().starts_with("FROM "))
            && sample_lines.iter().any(|l| {
                let t = l.trim_start();
                t.starts_with("RUN ")
                    || t.starts_with("COPY ")
                    || t.starts_with("WORKDIR ")
                    || t.starts_with("ENTRYPOINT ")
                    || t.starts_with("CMD ")
            })
        {
            return Some(Language::Dockerfile);
        }

        // 3. Weighted language scoring
        let mut scores: [(Language, i32); 16] = [
            (Language::Rust, 0),
            (Language::Python, 0),
            (Language::TypeScript, 0),
            (Language::JavaScript, 0),
            (Language::Sql, 0),
            (Language::Cpp, 0),
            (Language::C, 0),
            (Language::Go, 0),
            (Language::Java, 0),
            (Language::CSharp, 0),
            (Language::Swift, 0),
            (Language::Shell, 0),
            (Language::Html, 0),
            (Language::Css, 0),
            (Language::Toml, 0),
            (Language::Markdown, 0),
        ];

        let mut add = |lang: Language, pts: i32| {
            if let Some((_, s)) = scores.iter_mut().find(|(l, _)| *l == lang) {
                *s += pts;
            }
        };

        // Rust
        for (pat, pts) in [
            ("fn main(", 5),
            ("pub fn ", 4),
            ("fn ", 2),
            ("let mut ", 4),
            ("impl ", 3),
            ("use std::", 5),
            ("#[derive(", 5),
            ("println!(", 4),
            ("-> Result<", 4),
            ("-> Option<", 4),
            ("pub struct ", 4),
            ("pub enum ", 4),
            ("match ", 2),
        ] {
            if sample.contains(pat) {
                add(Language::Rust, pts);
            }
        }

        // Python
        for (pat, pts) in [
            ("def ", 3),
            ("if __name__ ==", 6),
            ("elif ", 4),
            ("import ", 2),
            ("from ", 1),
            ("print(", 2),
            ("self.", 3),
            ("None", 1),
            ("lambda ", 3),
            ("async def ", 5),
        ] {
            if sample.contains(pat) {
                add(Language::Python, pts);
            }
        }
        if sample_lines.iter().any(|l| {
            let t = l.trim();
            (t.starts_with("def ") || t.starts_with("class ")) && t.ends_with(':')
        }) {
            add(Language::Python, 4);
        }

        // TypeScript & JavaScript
        for (pat, pts) in [
            ("interface ", 4),
            (": string", 3),
            (": number", 3),
            (": boolean", 3),
            ("export interface ", 5),
            ("export type ", 5),
            ("readonly ", 3),
            ("as const", 4),
        ] {
            if sample.contains(pat) {
                add(Language::TypeScript, pts);
            }
        }
        for (pat, pts) in [
            ("console.log(", 4),
            ("const ", 2),
            ("function ", 3),
            ("=> {", 3),
            ("export default ", 4),
            ("export function ", 4),
            ("export const ", 3),
            ("import {", 3),
            ("document.", 3),
            ("async function ", 4),
            ("===", 2),
        ] {
            if sample.contains(pat) {
                add(Language::JavaScript, pts);
                add(Language::TypeScript, pts - 1);
            }
        }

        // SQL
        for (pat, pts) in [
            ("SELECT ", 3),
            (" FROM ", 3),
            ("CREATE TABLE", 5),
            ("INSERT INTO", 5),
            ("UPDATE ", 2),
            ("DELETE FROM", 5),
            (" WHERE ", 3),
            ("GROUP BY", 4),
            ("ORDER BY", 4),
            ("INNER JOIN", 5),
            ("LEFT JOIN", 5),
        ] {
            if sample_upper.contains(pat) {
                add(Language::Sql, pts);
            }
        }

        // C / C++
        if sample.contains("#include <") || sample.contains("#include \"") {
            add(Language::C, 4);
            add(Language::Cpp, 4);
        }
        for (pat, pts) in [
            ("std::", 5),
            ("<iostream>", 5),
            ("<vector>", 4),
            ("template<", 4),
            ("nullptr", 4),
            ("namespace ", 3),
        ] {
            if sample.contains(pat) {
                add(Language::Cpp, pts);
            }
        }
        for (pat, pts) in [
            ("<stdio.h>", 5),
            ("<stdlib.h>", 5),
            ("printf(", 3),
            ("int main(", 3),
            ("malloc(", 3),
        ] {
            if sample.contains(pat) {
                add(Language::C, pts);
            }
        }

        // Go
        for (pat, pts) in [
            ("package main", 5),
            ("func main()", 5),
            ("fmt.Print", 5),
            ("func (", 4),
            (":= ", 3),
            ("go func(", 4),
        ] {
            if sample.contains(pat) {
                add(Language::Go, pts);
            }
        }

        // Java & C#
        for (pat, pts) in [
            ("public static void main", 6),
            ("System.out.println", 5),
            ("public class ", 4),
            ("import java.", 5),
        ] {
            if sample.contains(pat) {
                add(Language::Java, pts);
            }
        }
        for (pat, pts) in [
            ("using System;", 6),
            ("Console.WriteLine", 5),
            ("namespace ", 2),
            ("public async Task", 5),
        ] {
            if sample.contains(pat) {
                add(Language::CSharp, pts);
            }
        }

        // Swift
        for (pat, pts) in [
            ("import Foundation", 5),
            ("import SwiftUI", 6),
            ("guard let ", 5),
            ("if let ", 3),
            ("func ", 2),
            (": View {", 5),
        ] {
            if sample.contains(pat) {
                add(Language::Swift, pts);
            }
        }

        // Shell
        for (pat, pts) in [
            ("set -euo pipefail", 6),
            ("export ", 2),
            ("if [[ ", 4),
            (" ]]; then", 4),
            ("\nfi", 3),
            ("echo \"$", 3),
        ] {
            if sample.contains(pat) {
                add(Language::Shell, pts);
            }
        }

        // HTML / XML tags
        for (pat, pts) in [
            ("<div", 3),
            ("</div>", 3),
            ("<span", 2),
            ("<body", 4),
            ("<head", 4),
            ("<script", 4),
            ("<style", 3),
            ("<section", 3),
        ] {
            if lower_head.contains(pat) {
                add(Language::Html, pts);
            }
        }

        // CSS
        if sample.contains('{') && sample.contains('}') {
            for (pat, pts) in [
                ("display:", 3),
                ("background-color:", 4),
                ("font-size:", 3),
                ("margin:", 2),
                ("padding:", 2),
                ("border-radius:", 4),
                ("@media ", 4),
                ("flex-direction:", 4),
            ] {
                if sample.contains(pat) {
                    add(Language::Css, pts);
                }
            }
        }

        // TOML
        let toml_sections = sample_lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with('[') && t.ends_with(']') && !t.contains(' ')
            })
            .count();
        let toml_kv = sample_lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                !t.starts_with('#') && t.contains(" = ")
            })
            .count();
        if toml_sections >= 1 && toml_kv >= 2 {
            add(Language::Toml, 5);
        }

        // Markdown
        let md_headings = sample_lines
            .iter()
            .filter(|l| {
                let t = l.trim_start();
                t.starts_with("# ") || t.starts_with("## ") || t.starts_with("### ")
            })
            .count();
        if md_headings >= 1 {
            add(Language::Markdown, 3);
        }
        if sample.contains("```") {
            add(Language::Markdown, 3);
        }
        if sample.contains("](") && sample.contains('[') {
            add(Language::Markdown, 2);
        }
        if sample.contains("**") {
            add(Language::Markdown, 2);
        }

        // Pick highest-scoring language above threshold (5 pts avoids false positives on normal English)
        let best = scores
            .into_iter()
            .max_by_key(|&(_, score)| score)
            .filter(|&(_, score)| score >= 5)
            .map(|(lang, _)| lang);

        if best.is_some() {
            return best;
        }

        // Fallback YAML check (multiple clean `key: value` lines without braces/semicolons)
        let yaml_kv = sample_lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                if t.starts_with('#') || t.contains('{') || t.contains(';') {
                    return false;
                }
                if let Some((k, v)) = t.split_once(':') {
                    !k.is_empty()
                        && k.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                        && (v.is_empty() || v.starts_with(' '))
                } else {
                    false
                }
            })
            .count();
        if (trimmed.starts_with("---\n") && yaml_kv >= 2) || yaml_kv >= 4 {
            return Some(Language::Yaml);
        }

        // Fallback generic XML check
        if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.contains("</") {
            return Some(Language::Xml);
        }

        None
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

    #[test]
    fn detects_languages_live_from_unsaved_content() {
        assert_eq!(
            Language::detect_from_content(
                "fn main() {\n    let mut x = 1;\n    println!(\"{x}\");\n}"
            ),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::detect_from_content(
                "def greet(name):\n    print(name)\n\nif __name__ == '__main__':\n    greet('hi')"
            ),
            Some(Language::Python)
        );
        assert_eq!(
            Language::detect_from_content(r#"{"name": "Aetheris", "fast": true}"#),
            Some(Language::Json)
        );
        assert_eq!(
            Language::detect_from_content(
                "SELECT id, name FROM users WHERE active = 1 ORDER BY id;"
            ),
            Some(Language::Sql)
        );
        assert_eq!(
            Language::detect_from_content(
                "<!DOCTYPE html><html><body><div>Hello</div></body></html>"
            ),
            Some(Language::Html)
        );
        assert_eq!(
            Language::detect_from_content(
                "export interface User {\n  id: number;\n  name: string;\n}"
            ),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::detect_from_content("#!/usr/bin/env bash\necho \"hello\"\n"),
            Some(Language::Shell)
        );
        assert_eq!(
            Language::detect_from_content(
                "package main\nimport \"fmt\"\nfunc main() { fmt.Println(\"hi\") }"
            ),
            Some(Language::Go)
        );
        assert_eq!(
            Language::detect_from_content(
                "Just writing an ordinary English note for tomorrow's meeting."
            ),
            None
        );
    }
}
