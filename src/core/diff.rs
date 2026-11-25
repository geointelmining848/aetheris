use similar::{ChangeTag, TextDiff};

/// Tag for a line inside a diff row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Delete,
    Insert,
}

/// One aligned row in a side-by-side comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRow {
    pub left_line_no: Option<usize>,
    pub left_text: String,
    pub right_line_no: Option<usize>,
    pub right_text: String,
    pub tag: DiffTag,
}

/// Summary and aligned rows of a two-document comparison.
#[derive(Debug, Clone, Default)]
pub struct DiffReport {
    pub rows: Vec<DiffRow>,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub unchanged_lines: usize,
}

/// Compute a side-by-side diff between two texts.
pub fn compute_diff(left: &str, right: &str) -> DiffReport {
    let diff = TextDiff::from_lines(left, right);
    let mut rows = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut unchanged = 0usize;

    let mut left_no = 1usize;
    let mut right_no = 1usize;

    for change in diff.iter_all_changes() {
        let value = change
            .value()
            .trim_end_matches(&['\r', '\n'][..])
            .to_owned();
        match change.tag() {
            ChangeTag::Equal => {
                unchanged += 1;
                rows.push(DiffRow {
                    left_line_no: Some(left_no),
                    left_text: value.clone(),
                    right_line_no: Some(right_no),
                    right_text: value,
                    tag: DiffTag::Equal,
                });
                left_no += 1;
                right_no += 1;
            }
            ChangeTag::Delete => {
                removed += 1;
                rows.push(DiffRow {
                    left_line_no: Some(left_no),
                    left_text: value,
                    right_line_no: None,
                    right_text: String::new(),
                    tag: DiffTag::Delete,
                });
                left_no += 1;
            }
            ChangeTag::Insert => {
                added += 1;
                rows.push(DiffRow {
                    left_line_no: None,
                    left_text: String::new(),
                    right_line_no: Some(right_no),
                    right_text: value,
                    tag: DiffTag::Insert,
                });
                right_no += 1;
            }
        }
    }

    DiffReport {
        rows,
        added_lines: added,
        removed_lines: removed,
        unchanged_lines: unchanged,
    }
}

/// Generate a standard unified diff string (suitable for saving as `.patch`).
pub fn unified_diff(left_name: &str, left: &str, right_name: &str, right: &str) -> String {
    let diff = TextDiff::from_lines(left, right);
    diff.unified_diff()
        .context_radius(3)
        .header(left_name, right_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_side_by_side_and_unified_diffs() {
        let a = "alpha\nbeta\ngamma\n";
        let b = "alpha\nBETA\ngamma\ndelta\n";
        let report = compute_diff(a, b);
        assert_eq!(report.unchanged_lines, 2);
        assert_eq!(report.removed_lines, 1);
        assert_eq!(report.added_lines, 2);

        let patch = unified_diff("a.txt", a, "b.txt", b);
        assert!(patch.contains("-beta"));
        assert!(patch.contains("+BETA"));
        assert!(patch.contains("+delta"));
    }
}
