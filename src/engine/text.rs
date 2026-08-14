// Text processing utilities. Will grow as we add more commands.

/// Split a string into (line_number, line_content) pairs.
/// Line numbers are 1-indexed.
pub fn enumerate_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines().enumerate().map(|(i, l)| (i + 1, l))
}
