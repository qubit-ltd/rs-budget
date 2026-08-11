// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Non-recursive lexical preflight for already validated JSON documents.

use crate::BudgetError;
use crate::JsonBudget;

/// Charges one validated JSON document directly from its lexical form.
///
/// Reading the JSON syntax, rather than Serde's data-model events, is
/// important because `serde_json` represents arbitrary-precision numbers with
/// a private map token. An ordinary object can legally contain the same text,
/// so its semantic resource type cannot be inferred from deserializer events.
pub(in crate::serde) struct JsonPreflight<'a, R> {
    /// Budget session mutated while walking the document.
    budget: &'a mut JsonBudget<R, usize>,
}

impl<'a, R> JsonPreflight<'a, R>
where
    R: Clone,
{
    /// Creates a lexical walker bound to one budget session.
    pub(in crate::serde) const fn new(budget: &'a mut JsonBudget<R, usize>) -> Self {
        Self { budget }
    }

    /// Charges one already validated JSON value at `depth`.
    ///
    /// # Errors
    ///
    /// Returns the first typed resource-budget violation. The caller must
    /// validate the complete JSON document before invoking this method.
    pub(in crate::serde) fn inspect(
        &mut self,
        input: &[u8],
        depth: usize,
    ) -> Result<(), BudgetError<R, usize>> {
        let mut cursor = JsonCursor::new(input, self.budget);
        cursor.skip_whitespace();
        let mut stack = Vec::new();
        cursor.value(depth, &mut stack)?;
        while let Some(frame) = stack.pop() {
            cursor.resume(frame, &mut stack)?;
        }
        cursor.skip_whitespace();
        debug_assert_eq!(cursor.position, input.len());
        Ok(())
    }
}

/// Continuation for one open JSON container in the iterative walker.
enum ContainerFrame {
    /// Array ready to consume its next child.
    ArrayNext {
        /// Root-inclusive depth of the array.
        depth: usize,

        /// Number of children already consumed.
        items: usize,
    },

    /// Array waiting to consume a comma or closing bracket.
    ArrayAfter {
        /// Root-inclusive depth of the array.
        depth: usize,

        /// Number of children already consumed.
        items: usize,
    },

    /// Object ready to consume its next key and value.
    ObjectNext {
        /// Root-inclusive depth of the object.
        depth: usize,

        /// Number of entries already consumed.
        entries: usize,
    },

    /// Object waiting to consume a comma or closing brace.
    ObjectAfter {
        /// Root-inclusive depth of the object.
        depth: usize,

        /// Number of entries already consumed.
        entries: usize,
    },
}

/// Cursor that measures a syntactically valid JSON byte slice.
struct JsonCursor<'a, 'budget, R> {
    /// Complete validated JSON bytes.
    input: &'a [u8],

    /// Current byte offset.
    position: usize,

    /// Budget charged by the lexical walk.
    budget: &'budget mut JsonBudget<R, usize>,
}

impl<'a, 'budget, R> JsonCursor<'a, 'budget, R>
where
    R: Clone,
{
    /// Creates a cursor at the start of `input`.
    const fn new(input: &'a [u8], budget: &'budget mut JsonBudget<R, usize>) -> Self {
        Self {
            input,
            position: 0,
            budget,
        }
    }

    /// Advances past JSON whitespace.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    /// Returns the current byte without advancing.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    /// Charges and consumes one JSON value.
    fn value(
        &mut self,
        depth: usize,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), BudgetError<R, usize>> {
        match self.peek() {
            Some(b'{') => self.open_object(depth, stack),
            Some(b'[') => self.open_array(depth, stack),
            Some(b'"') => {
                self.budget.enter_node(depth)?;
                let bytes = self.string_bytes();
                self.budget.check_string_bytes(bytes)
            }
            Some(b'-' | b'0'..=b'9') => {
                self.budget.enter_node(depth)?;
                let bytes = self.number_bytes();
                self.budget.check_number_bytes(bytes)
            }
            Some(b't') => self.literal(depth, 4),
            Some(b'f') => self.literal(depth, 5),
            Some(b'n') => self.literal(depth, 4),
            _ => unreachable!("JSON was validated before lexical preflight"),
        }
    }

    /// Charges a scalar literal and advances by its validated byte length.
    fn literal(&mut self, depth: usize, bytes: usize) -> Result<(), BudgetError<R, usize>> {
        self.budget.enter_node(depth)?;
        self.position += bytes;
        Ok(())
    }

    /// Charges and opens one JSON array without recursively consuming children.
    fn open_array(
        &mut self,
        depth: usize,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), BudgetError<R, usize>> {
        self.budget.enter_node(depth)?;
        self.position += 1;
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(());
        }
        stack.push(ContainerFrame::ArrayNext { depth, items: 0 });
        Ok(())
    }

    /// Charges and opens one JSON object without recursively consuming values.
    fn open_object(
        &mut self,
        depth: usize,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), BudgetError<R, usize>> {
        self.budget.enter_node(depth)?;
        self.position += 1;
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(());
        }
        stack.push(ContainerFrame::ObjectNext { depth, entries: 0 });
        Ok(())
    }

    /// Resumes one open-container continuation.
    fn resume(
        &mut self,
        frame: ContainerFrame,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), BudgetError<R, usize>> {
        match frame {
            ContainerFrame::ArrayNext { depth, items } => {
                let items = items.saturating_add(1);
                self.budget.check_sequence_items(items)?;
                stack.push(ContainerFrame::ArrayAfter { depth, items });
                self.value(depth.saturating_add(1), stack)
            }
            ContainerFrame::ArrayAfter { depth, items } => {
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                        self.skip_whitespace();
                        stack.push(ContainerFrame::ArrayNext { depth, items });
                    }
                    Some(b']') => self.position += 1,
                    _ => unreachable!("JSON was validated before lexical preflight"),
                }
                Ok(())
            }
            ContainerFrame::ObjectNext { depth, entries } => {
                let entries = entries.saturating_add(1);
                self.budget.check_map_entries(entries)?;
                let key_bytes = self.string_bytes();
                self.budget.check_key_bytes(key_bytes)?;
                self.skip_whitespace();
                debug_assert_eq!(self.peek(), Some(b':'));
                self.position += 1;
                self.skip_whitespace();
                stack.push(ContainerFrame::ObjectAfter { depth, entries });
                self.value(depth.saturating_add(1), stack)
            }
            ContainerFrame::ObjectAfter { depth, entries } => {
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                        self.skip_whitespace();
                        stack.push(ContainerFrame::ObjectNext { depth, entries });
                    }
                    Some(b'}') => self.position += 1,
                    _ => unreachable!("JSON was validated before lexical preflight"),
                }
                Ok(())
            }
        }
    }

    /// Consumes one JSON string and returns its decoded UTF-8 byte length.
    fn string_bytes(&mut self) -> usize {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.position += 1;
        let mut decoded = 0_usize;
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.position += 1;
                    return decoded;
                }
                Some(b'\\') => {
                    self.position += 1;
                    match self.peek() {
                        Some(b'u') => decoded += self.unicode_escape_bytes(),
                        Some(_) => {
                            self.position += 1;
                            decoded += 1;
                        }
                        None => unreachable!("JSON was validated before lexical preflight"),
                    }
                }
                Some(byte) => {
                    let width = utf8_width(byte);
                    self.position += width;
                    decoded += width;
                }
                None => unreachable!("JSON was validated before lexical preflight"),
            }
        }
    }

    /// Consumes a `\uXXXX` escape and returns its decoded UTF-8 byte length.
    fn unicode_escape_bytes(&mut self) -> usize {
        debug_assert_eq!(self.peek(), Some(b'u'));
        self.position += 1;
        let first = self.hex_quad();
        if (0xD800..=0xDBFF).contains(&first) {
            debug_assert_eq!(self.peek(), Some(b'\\'));
            self.position += 2;
            let second = self.hex_quad();
            let scalar =
                0x1_0000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00);
            char::from_u32(scalar)
                .expect("validated surrogate pair must be a Unicode scalar")
                .len_utf8()
        } else {
            char::from_u32(u32::from(first))
                .expect("validated escape must be a Unicode scalar")
                .len_utf8()
        }
    }

    /// Consumes four validated hexadecimal digits.
    fn hex_quad(&mut self) -> u16 {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(byte @ b'0'..=b'9') => u16::from(byte - b'0'),
                Some(byte @ b'a'..=b'f') => u16::from(byte - b'a' + 10),
                Some(byte @ b'A'..=b'F') => u16::from(byte - b'A' + 10),
                _ => unreachable!("JSON was validated before lexical preflight"),
            };
            value = (value << 4) | digit;
            self.position += 1;
        }
        value
    }

    /// Consumes one validated JSON number and returns its textual byte length.
    fn number_bytes(&mut self) -> usize {
        let start = self.position;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.position += 1;
        }
        self.position - start
    }
}

/// Returns the byte width of a validated UTF-8 leading byte.
const fn utf8_width(byte: u8) -> usize {
    if byte < 0x80 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}
