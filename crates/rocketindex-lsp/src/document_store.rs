//! In-memory document store for tracking open files and their content.
//!
//! This module provides a thread-safe store for document content, enabling
//! the language server to work with unsaved changes instead of only saved files.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::{Position, TextDocumentContentChangeEvent, Url};

/// A document tracked in memory.
#[derive(Debug, Clone)]
pub struct Document {
    /// The current content of the document
    pub content: String,
    /// The document version (incremented on each change)
    pub version: i32,
}

impl Document {
    /// Create a new document with the given content.
    pub fn new(content: String, version: i32) -> Self {
        Self { content, version }
    }

    /// Apply incremental changes to the document content.
    ///
    /// Handles both full document sync and incremental changes.
    pub fn apply_changes(
        &mut self,
        changes: Vec<TextDocumentContentChangeEvent>,
        new_version: i32,
    ) {
        for change in changes {
            match change.range {
                Some(range) => {
                    // Incremental change - replace the specified range
                    let start_offset = self.position_to_offset(&range.start);
                    let end_offset = self.position_to_offset(&range.end);

                    if let (Some(start), Some(end)) = (start_offset, end_offset) {
                        // Guard against a malformed (inverted/out-of-range) range so
                        // the capacity arithmetic and slices below can't panic.
                        if start <= end && end <= self.content.len() {
                            let mut new_content = String::with_capacity(
                                self.content.len() - (end - start) + change.text.len(),
                            );
                            new_content.push_str(&self.content[..start]);
                            new_content.push_str(&change.text);
                            new_content.push_str(&self.content[end..]);
                            self.content = new_content;
                        }
                    }
                }
                None => {
                    // Full document sync - replace entire content
                    self.content = change.text;
                }
            }
        }
        self.version = new_version;
    }

    /// Convert a Position (line, UTF-16 character) to a byte offset in the content.
    ///
    /// LSP positions count lines by `\n` and the `character` field in UTF-16 code
    /// units (the default `positionEncoding`). We therefore iterate lines including
    /// their terminators (so `\r\n` and the missing-final-newline cases stay byte
    /// accurate) and convert the UTF-16 column to a byte offset within the line.
    ///
    /// The returned offset always lands on a UTF-8 char boundary, so callers can
    /// slice `content` with it without panicking.
    fn position_to_offset(&self, position: &Position) -> Option<usize> {
        let target_line = position.line as usize;
        let target_char = position.character as usize;

        let mut line_start = 0usize;
        let mut current_line = 0usize;

        // `split_inclusive` keeps the trailing `\n` on each line, preserving byte
        // offsets exactly (unlike `lines()`, which strips `\r` and the newline).
        for line in self.content.split_inclusive('\n') {
            if current_line == target_line {
                return Some(line_start + utf16_to_byte_offset(line, target_char));
            }
            line_start += line.len();
            current_line += 1;
        }

        // Position on the empty line after a trailing newline (or an empty doc).
        if current_line == target_line {
            let rest = &self.content[line_start..];
            return Some(line_start + utf16_to_byte_offset(rest, target_char));
        }

        None
    }
}

/// Convert a UTF-16 code-unit column within `line` to a byte offset.
///
/// Clamps to the end of the line content (excluding any trailing `\r\n`/`\n`), so
/// the result is always a valid UTF-8 char boundary within `line`.
fn utf16_to_byte_offset(line: &str, utf16_col: usize) -> usize {
    // Don't let a column past the end of the line spill onto the newline.
    let content_end = line
        .strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(line)
        .len();

    let mut utf16_count = 0usize;
    for (byte_idx, ch) in line.char_indices() {
        if utf16_count >= utf16_col {
            return byte_idx.min(content_end);
        }
        utf16_count += ch.len_utf16();
    }
    content_end
}

/// Thread-safe store for open documents.
#[derive(Debug, Default)]
pub struct DocumentStore {
    /// Map from file path to document
    documents: Arc<RwLock<HashMap<PathBuf, Document>>>,
}

impl DocumentStore {
    /// Create a new empty document store.
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Open a document and store its content.
    pub async fn open(&self, uri: &Url, content: String, version: i32) {
        if let Ok(path) = uri.to_file_path() {
            let document = Document::new(content, version);
            self.documents.write().await.insert(path, document);
        }
    }

    /// Apply changes to an open document.
    pub async fn change(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if let Ok(path) = uri.to_file_path() {
            let mut docs = self.documents.write().await;
            if let Some(doc) = docs.get_mut(&path) {
                doc.apply_changes(changes, version);
            }
        }
    }

    /// Close a document and remove it from the store.
    pub async fn close(&self, uri: &Url) {
        if let Ok(path) = uri.to_file_path() {
            self.documents.write().await.remove(&path);
        }
    }

    /// Get the content of a document.
    ///
    /// Returns the in-memory content if the document is open,
    /// otherwise attempts to read from disk.
    pub async fn get_content(&self, path: &PathBuf) -> Option<String> {
        // First, check if the document is open in memory
        {
            let docs = self.documents.read().await;
            if let Some(doc) = docs.get(path) {
                return Some(doc.content.clone());
            }
        }

        // Fall back to reading from disk (async I/O)
        tokio::fs::read_to_string(path).await.ok()
    }
}

impl Clone for DocumentStore {
    fn clone(&self) -> Self {
        Self {
            documents: Arc::clone(&self.documents),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_offset_simple() {
        let doc = Document::new("hello\nworld\n".to_string(), 1);

        // First character of first line
        assert_eq!(doc.position_to_offset(&Position::new(0, 0)), Some(0));
        // Third character of first line
        assert_eq!(doc.position_to_offset(&Position::new(0, 2)), Some(2));
        // First character of second line
        assert_eq!(doc.position_to_offset(&Position::new(1, 0)), Some(6));
        // Third character of second line
        assert_eq!(doc.position_to_offset(&Position::new(1, 2)), Some(8));
    }

    #[test]
    fn test_apply_full_change() {
        let mut doc = Document::new("original content".to_string(), 1);

        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "new content".to_string(),
            }],
            2,
        );

        assert_eq!(doc.content, "new content");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_apply_incremental_change() {
        let mut doc = Document::new("hello world".to_string(), 1);

        // Replace "world" with "rust"
        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: Position::new(0, 6),
                    end: Position::new(0, 11),
                }),
                range_length: None,
                text: "rust".to_string(),
            }],
            2,
        );

        assert_eq!(doc.content, "hello rust");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_apply_insert() {
        let mut doc = Document::new("hello world".to_string(), 1);

        // Insert "beautiful " before "world"
        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: Position::new(0, 6),
                    end: Position::new(0, 6),
                }),
                range_length: None,
                text: "beautiful ".to_string(),
            }],
            2,
        );

        assert_eq!(doc.content, "hello beautiful world");
    }

    #[test]
    fn test_apply_delete() {
        let mut doc = Document::new("hello beautiful world".to_string(), 1);

        // Delete "beautiful "
        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: Position::new(0, 6),
                    end: Position::new(0, 16),
                }),
                range_length: None,
                text: "".to_string(),
            }],
            2,
        );

        assert_eq!(doc.content, "hello world");
    }

    #[test]
    fn test_multiline_position() {
        let doc = Document::new("line one\nline two\nline three".to_string(), 1);

        // Start of line 2
        assert_eq!(doc.position_to_offset(&Position::new(2, 0)), Some(18));
        // "three" starts at position 5 on line 2
        assert_eq!(doc.position_to_offset(&Position::new(2, 5)), Some(23));
    }

    #[tokio::test]
    async fn test_document_store_open_close() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///test.fs").unwrap();
        let path = uri.to_file_path().unwrap();

        // Initially empty
        assert!(store.get_content(&path).await.is_none());

        // Open document
        store.open(&uri, "content".to_string(), 1).await;
        assert_eq!(store.get_content(&path).await, Some("content".to_string()));

        // Close document
        store.close(&uri).await;
        assert!(store.get_content(&path).await.is_none());
    }

    #[tokio::test]
    async fn test_document_store_change() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///test.fs").unwrap();
        let path = uri.to_file_path().unwrap();

        // Open document
        store.open(&uri, "hello".to_string(), 1).await;

        // Apply change
        store
            .change(
                &uri,
                vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "goodbye".to_string(),
                }],
                2,
            )
            .await;

        assert_eq!(store.get_content(&path).await, Some("goodbye".to_string()));
    }

    #[test]
    fn test_position_to_offset_crlf() {
        // \r\n line endings: line.len() must account for the \r.
        let doc = Document::new("hello\r\nworld\r\n".to_string(), 1);

        assert_eq!(doc.position_to_offset(&Position::new(0, 0)), Some(0));
        // Start of line 1 is after "hello\r\n" = 7 bytes.
        assert_eq!(doc.position_to_offset(&Position::new(1, 0)), Some(7));
        // Third char of line 1.
        assert_eq!(doc.position_to_offset(&Position::new(1, 2)), Some(9));
        // Empty line after trailing CRLF.
        assert_eq!(doc.position_to_offset(&Position::new(2, 0)), Some(14));
    }

    #[test]
    fn test_position_to_offset_multibyte() {
        // "héllo" — é is 2 bytes (U+00E9), 1 UTF-16 unit.
        let doc = Document::new("héllo\nworld".to_string(), 1);

        // Column 1 (after 'h') = byte 1.
        assert_eq!(doc.position_to_offset(&Position::new(0, 1)), Some(1));
        // Column 2 (after 'é') = byte 3 (h=1 + é=2).
        assert_eq!(doc.position_to_offset(&Position::new(0, 2)), Some(3));
        // Line 1 starts after "héllo\n" = 7 bytes (1+2+1+1+1 + newline).
        assert_eq!(doc.position_to_offset(&Position::new(1, 0)), Some(7));
    }

    #[test]
    fn test_position_to_offset_astral_utf16() {
        // "😀x" — emoji is 4 bytes, 2 UTF-16 code units.
        let doc = Document::new("😀x".to_string(), 1);

        // UTF-16 column 2 lands after the emoji (byte 4), not after 2 chars.
        assert_eq!(doc.position_to_offset(&Position::new(0, 2)), Some(4));
        // Column 3 = after 'x' = byte 5.
        assert_eq!(doc.position_to_offset(&Position::new(0, 3)), Some(5));
    }

    #[test]
    fn test_apply_incremental_change_crlf_no_panic() {
        // Regression: editing a CRLF + multibyte document must not panic or corrupt.
        let mut doc = Document::new("café\r\nbar".to_string(), 1);

        // Replace "bar" (line 1, cols 0..3) with "baz".
        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: Position::new(1, 0),
                    end: Position::new(1, 3),
                }),
                range_length: None,
                text: "baz".to_string(),
            }],
            2,
        );

        assert_eq!(doc.content, "café\r\nbaz");
    }

    #[test]
    fn test_apply_change_inverted_range_is_ignored() {
        // A malformed range (start > end) must not panic.
        let mut doc = Document::new("hello".to_string(), 1);
        doc.apply_changes(
            vec![TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: Position::new(0, 4),
                    end: Position::new(0, 1),
                }),
                range_length: None,
                text: "X".to_string(),
            }],
            2,
        );
        // Content unchanged (change skipped).
        assert_eq!(doc.content, "hello");
    }
}
