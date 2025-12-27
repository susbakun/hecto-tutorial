use crate::prelude::*;
use crate::editor::{AnnotationType, annotation::Annotation};
use std::cmp::min;

use super::{AnnotatedString, AnnotatedStringPart};

pub struct AnnotatedStringIterator<'a> {
    pub annotated_string: &'a AnnotatedString,
    pub current_idx: ByteIdx,
}

impl<'a> Iterator for AnnotatedStringIterator<'a> {
    type Item = AnnotatedStringPart<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.annotated_string.string.len() {
            return None;
        }

        // Find all annotations that cover the current position
        let active_annotations: Vec<&Annotation> = self
            .annotated_string
            .annotations
            .iter()
            .filter(|annotation| {
                annotation.start <= self.current_idx && annotation.end > self.current_idx
            })
            .collect();

        if !active_annotations.is_empty() {
            // Check if there's a Select annotation among the active ones
            let select_annotation = active_annotations
                .iter()
                .find(|annotation| annotation.annotation_type 
                    == AnnotationType::Select);


            if let Some(select_ann) = select_annotation {
                // Handle Select annotation with syntax highlighting preserved
                let mut end_idx = min(select_ann.end, self.annotated_string.string.len());
                
                // Find the earliest boundary among all active annotations
                // This ensures we break at syntax annotation boundaries
                for annotation in &active_annotations {
                    if annotation.annotation_type != AnnotationType::Select {
                        end_idx = min(end_idx, annotation.end);
                    }
                }
                
                // Also check if any other annotation starts within the Select range
                for annotation in &self.annotated_string.annotations {
                    if annotation.annotation_type != AnnotationType::Select
                        && annotation.start > self.current_idx
                        && annotation.start < end_idx
                    {
                        end_idx = annotation.start;
                    }
                }

                let start_idx = self.current_idx;
                self.current_idx = end_idx;

                let annotation_types: Vec<AnnotationType> = active_annotations
                    .iter()
                    .map(|a| a.annotation_type)
                    .collect();


                return Some(AnnotatedStringPart {
                    string: &self.annotated_string.string[start_idx..end_idx],
                    annotation_types,
                });
            } else {
                // No Select annotation, find the next boundary considering all annotations
                let mut end_idx = self.annotated_string.string.len();
                
                // Find the earliest end point among active annotations
                for annotation in &active_annotations {
                    end_idx = min(end_idx, annotation.end);
                }
                
                // Check if any Select annotation starts before this end point
                for annotation in &self.annotated_string.annotations {
                    if annotation.annotation_type == AnnotationType::Select
                        && annotation.start > self.current_idx
                        && annotation.start < end_idx
                    {
                        end_idx = annotation.start;
                        break;
                    }
                }

                let start_idx = self.current_idx;
                self.current_idx = end_idx;

                // Collect all active annotation types
                let annotation_types: Vec<AnnotationType> = active_annotations
                    .iter()
                    .map(|annotation| annotation.annotation_type)
                    .collect();

                return Some(AnnotatedStringPart {
                    string: &self.annotated_string.string[start_idx..end_idx],
                    annotation_types,
                });
            }
        }

        // No active annotations - find the boundary of the nearest annotation
        let mut end_idx = self.annotated_string.string.len();
        for annotation in &self.annotated_string.annotations {
            if annotation.start > self.current_idx && annotation.start < end_idx {
                end_idx = annotation.start;
            }
        }
        let start_idx = self.current_idx;
        self.current_idx = end_idx;

        Some(AnnotatedStringPart {
            string: &self.annotated_string.string[start_idx..end_idx],
            annotation_types: vec![],
        })
    }
}
