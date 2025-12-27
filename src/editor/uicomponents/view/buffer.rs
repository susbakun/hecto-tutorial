use super::super::super::AnnotatedString;
use super::FileInfo;
use super::Highlighter;
use super::Line;
use crate::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Error;
use std::io::Write;
use std::ops::Range;

#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,
    file_info: FileInfo,
    dirty: bool,
}

impl Buffer {
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub const fn get_file_info(&self) -> &FileInfo {
        &self.file_info
    }

    pub fn grapheme_count(&self, idx: LineIdx) -> GraphemeIdx {
        self.lines.get(idx).map_or(0, Line::grapheme_count)
    }
    pub fn width_until(&self, idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(idx)
            .map_or(0, |line| line.width_until(until))
    }

    pub fn get_highlighted_substring(
        &self,
        line_idx: LineIdx,
        range: Range<GraphemeIdx>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_idx).map(|line| {
            line.get_annotated_visible_substr(range, Some(&highlighter.get_annotations(line_idx)))
        })
    }

    pub fn highlight(&self, idx: LineIdx, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(idx) {
            highlighter.highlight(idx, line);
        }
    }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self {
            lines,
            file_info: FileInfo::from(file_name),
            dirty: false,
        })
    }

    pub fn search_forward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }
        let mut is_first = true;
        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .cycle()
            .skip(from.line_idx)
            .take(self.lines.len().saturating_add(1))
        //taking one more, to search the current line twice (once from the middle, once from the start)
        {
            let from_grapheme_idx = if is_first {
                is_first = false;
                from.grapheme_idx
            } else {
                0
            };
            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
    }
    pub fn search_backward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }
        let mut is_first = true;
        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(from.line_idx)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_idx = if is_first {
                is_first = false;
                from.grapheme_idx
            } else {
                line.grapheme_count()
            };
            if let Some(grapheme_idx) = line.search_backward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
    }

    fn save_to_file(&self, file_info: &FileInfo) -> Result<(), Error> {
        if let Some(file_path) = &file_info.get_path() {
            let mut file = File::create(file_path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to save with no file path present");
            }
        }
        Ok(())
    }
    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }
    pub fn height(&self) -> LineIdx {
        self.lines.len()
    }
    pub fn insert_char(&mut self, character: char, at: Location) {
        debug_assert!(at.line_idx <= self.height());
        if at.line_idx == self.height() {
            self.lines.push(Line::from(&character.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(character, at.grapheme_idx);
            self.dirty = true;
        }
    }
    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && self.height() > at.line_idx.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statment
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_idx < line.grapheme_count() {
                // clippy::indexing_slicing: We checked for existence of this line in the surrounding if statment
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].delete(at.grapheme_idx);
                self.dirty = true;
            }
        }
    }

    pub fn delete_range(&mut self, range: SelectRange) {
        let (mut start, mut end) = range;
        let was_backward = start.line_idx > end.line_idx 
            || (start.line_idx == end.line_idx && start.grapheme_idx > end.grapheme_idx);

        // Normalize: ensure start comes before end
        if was_backward {
            std::mem::swap(&mut start, &mut end);
        }

        // Calculate total number of graphemes to delete
        let deletion_count = if start.line_idx == end.line_idx {
            // Single line: delete from start.grapheme_idx to end.grapheme_idx
            end.grapheme_idx.saturating_sub(start.grapheme_idx)
        } else {
            // Multi-line: count graphemes from start to end
            // First line: from start.grapheme_idx to end of line
            // clippy::indexing_slicing: start has valid line index
            #[allow(clippy::indexing_slicing)]
            let first_line_len = self.lines[start.line_idx].grapheme_count();
            let mut count = first_line_len.saturating_sub(start.grapheme_idx);

            // Middle lines: entire lines
            for line_idx in (start.line_idx + 1)..end.line_idx {
                // clippy::indexing_slicing: line_idx is within valid range
                #[allow(clippy::indexing_slicing)]
                {
                    count = count.saturating_add(self.lines[line_idx].grapheme_count());
                }
            }

            // Last line: from start to end.grapheme_idx
            count = count.saturating_add(end.grapheme_idx);
            
            // Handling new line chars
            count = count.saturating_add(end.line_idx.saturating_sub(start.line_idx));
            
            count
        };

        // Delete the calculated number of graphemes
        for _ in 0..deletion_count {
            self.delete(start);
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new);
            self.dirty = true;
        }
    }

    pub fn get_a_grapheme(&self, at: Location) -> Option<String> {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx < line.grapheme_count() {
                // Extract the full grapheme at the given index (may be multi-character)
                return line.get_a_grapheme(at.grapheme_idx);
            } else if at.grapheme_idx == line.grapheme_count() {
                // At end of line, deleting merges with next line (deletes newline)
                return Some("\n".to_string());
            }
        }
        None
    }

    pub fn get_range_grapheme(&self, range: SelectRange) -> String {
        let (mut start, mut end) = range;
        
        // Normalize: ensure start comes before end
        let was_backward = start.line_idx > end.line_idx 
            || (start.line_idx == end.line_idx && start.grapheme_idx > end.grapheme_idx);
        
        if was_backward {
            std::mem::swap(&mut start, &mut end);
        }
        
        if start.line_idx == end.line_idx {
            // Single line case: extract graphemes from start to end
            if let Some(line) = self.lines.get(start.line_idx) {
                return line.get_grapheme_range(start.grapheme_idx, end.grapheme_idx);
            }
            return String::new();
        } else {
            // Multi-line case
            let mut graphemes = String::new();
            
            // First line: from start.grapheme_idx to end of line
            if let Some(first_line) = self.lines.get(start.line_idx) {
                let first_line_end = first_line.grapheme_count();
                graphemes.push_str(&first_line.get_grapheme_range(start.grapheme_idx, first_line_end));
            }
            graphemes.push('\n');
            
            // Middle lines: entire lines (if any)
            for line_idx in (start.line_idx + 1)..end.line_idx {
                if let Some(line) = self.lines.get(line_idx) {
                    let line_end = line.grapheme_count();
                    graphemes.push_str(&line.get_grapheme_range(0, line_end));
                    graphemes.push('\n');
                }
            }
            
            // Last line: from start to end.grapheme_idx
            if let Some(last_line) = self.lines.get(end.line_idx) {
                if end.grapheme_idx > 0 {
                    graphemes.push_str(&last_line.get_grapheme_range(0, end.grapheme_idx));
                }
            }
            
            graphemes
        }
    }
}
