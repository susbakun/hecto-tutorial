use std::collections::HashMap;
use super::{syntaxhighlighter::SyntaxHighlighter, Annotation, AnnotationType, Line};
use crate::prelude::*;


#[derive(Default)]
pub struct SelectHighlighter{
    selected_range: SelectRange,
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl SelectHighlighter {
    pub fn new(selected_range: SelectRange) -> Self {
        Self {
            selected_range,
            highlights: HashMap::new()
        }
    }

    fn highlight_selected_words(&mut self, idx: LineIdx, line: &Line, result: &mut Vec<Annotation>) {
        let start = self.selected_range.0;
        let end = self.selected_range.1;

        // Normalize selection range: ensure start comes before end
        if start.line_idx > end.line_idx {
            self.selected_range = (end, start);
            self.highlight_selected_words(idx, line, result);
            return;
        }

        // Determine highlight boundaries for this line
        // Start: beginning of line if we're past the selection start, otherwise use start position
        let highlight_start = if idx > start.line_idx {
            0 
        } else {
            start.grapheme_idx
        };

        // End: end of line if we're before the selection end, otherwise use end position
        let highlight_end = if idx < end.line_idx {
            line.grapheme_count()
        } else {
            end.grapheme_idx
        };

        // Add selection annotation if this line is within the selected range
        if idx >= start.line_idx && idx <= end.line_idx{
            result.push(Annotation {
                annotation_type: AnnotationType::Select,
                start: highlight_start,
                end: highlight_end,
            });
        }
    }
}

impl<'a> SyntaxHighlighter for SelectHighlighter {
    fn highlight(&mut self, idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        self.highlight_selected_words(idx, line, &mut result);
        
        self.highlights.insert(idx, result);
    }
    fn get_annotations(&self, idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&idx)
    }
}