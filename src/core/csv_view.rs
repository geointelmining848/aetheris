/// Parsed tabular representation of a CSV / TSV / DSV document.
#[derive(Debug, Clone, Default)]
pub struct CsvTable {
    pub delimiter: u8,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub col_widths: Vec<usize>,
}

impl CsvTable {
    pub fn delimiter_label(&self) -> &'static str {
        match self.delimiter {
            b',' => "Comma (,)",
            b'\t' => "Tab (\\t)",
            b';' => "Semicolon (;)",
            b'|' => "Pipe (|)",
            _ => "Custom",
        }
    }

    /// Return rows matching a case-insensitive filter query, optionally sorted by a column.
    pub fn filtered_rows(
        &self,
        filter: &str,
        sort_col: Option<usize>,
        ascending: bool,
        max_rows: usize,
    ) -> Vec<&Vec<String>> {
        let needle = filter.trim().to_ascii_lowercase();
        let mut matched: Vec<&Vec<String>> = self
            .rows
            .iter()
            .filter(|row| {
                needle.is_empty()
                    || row
                        .iter()
                        .any(|cell| cell.to_ascii_lowercase().contains(&needle))
            })
            .collect();

        if let Some(col) = sort_col {
            matched.sort_by(|a, b| {
                let va = a.get(col).map(String::as_str).unwrap_or("");
                let vb = b.get(col).map(String::as_str).unwrap_or("");
                let ord = match (va.parse::<f64>(), vb.parse::<f64>()) {
                    (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
                    _ => va.cmp(vb),
                };
                if ascending { ord } else { ord.reverse() }
            });
        }

        matched.truncate(max_rows);
        matched
    }
}

/// Auto-detect the most likely delimiter among comma, tab, semicolon, and pipe.
pub fn detect_delimiter(text: &str) -> u8 {
    let candidates = *b",\t;|";
    let sample_lines: Vec<&str> = text
        .lines()
        .take(20)
        .filter(|l| !l.trim().is_empty())
        .collect();
    if sample_lines.is_empty() {
        return b',';
    }

    let mut best = b',';
    let mut best_score = 0usize;

    for &delim in &candidates {
        let ch = delim as char;
        let counts: Vec<usize> = sample_lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == ch).count())
            .collect();
        let total: usize = counts.iter().sum();
        if total == 0 {
            continue;
        }
        let first = counts[0];
        let consistent = counts.iter().filter(|&&c| c == first && c > 0).count();
        let score = consistent * 100 + total;
        if score > best_score {
            best_score = score;
            best = delim;
        }
    }

    best
}

/// Parse delimited text into a structured `CsvTable` (capped at `max_rows` for fast UI preview).
pub fn parse_csv(text: &str, max_rows: usize) -> Result<CsvTable, String> {
    let delimiter = detect_delimiter(text);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut all_rows: Vec<Vec<String>> = Vec::new();
    let mut max_cols = 0usize;

    for record in rdr.records().take(max_rows + 1) {
        let rec = record.map_err(|e| format!("CSV parse error: {e}"))?;
        let cells: Vec<String> = rec.iter().map(str::to_owned).collect();
        max_cols = max_cols.max(cells.len());
        all_rows.push(cells);
    }

    if all_rows.is_empty() {
        return Ok(CsvTable {
            delimiter,
            ..Default::default()
        });
    }

    let mut headers = all_rows.remove(0);
    while headers.len() < max_cols {
        headers.push(format!("Col {}", headers.len() + 1));
    }

    let mut col_widths: Vec<usize> = headers
        .iter()
        .map(|h| h.chars().count().clamp(6, 36))
        .collect();

    for row in &mut all_rows {
        while row.len() < max_cols {
            row.push(String::new());
        }
        for (i, cell) in row.iter().enumerate() {
            let w = cell.chars().count().clamp(6, 36);
            if w > col_widths[i] {
                col_widths[i] = w;
            }
        }
    }

    Ok(CsvTable {
        delimiter,
        headers,
        rows: all_rows,
        col_widths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_delimiters_parses_and_sorts_csv() {
        let tsv = "name\tscore\tcity\nAlice\t92\tNYC\nBob\t105\tLA\nCarol\t88\tBoston\n";
        assert_eq!(detect_delimiter(tsv), b'\t');
        let table = parse_csv(tsv, 100).unwrap();
        assert_eq!(table.headers, vec!["name", "score", "city"]);
        assert_eq!(table.rows.len(), 3);

        let sorted = table.filtered_rows("", Some(1), false, 10);
        assert_eq!(sorted[0][0], "Bob");
        assert_eq!(sorted[1][0], "Alice");

        let filtered = table.filtered_rows("boston", None, true, 10);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Carol");
    }
}
