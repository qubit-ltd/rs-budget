// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Non-recursive lexical admission for one JSON input document.

use serde::de::Error as _;

use crate::JsonSerdeError;
use crate::JsonValueBudget;

/// Lexically validates and charges one JSON document without recursion.
pub(in crate::serde) struct JsonLexicalPreflight<'a, R> {
    /// JSON value resources charged while scanning the document.
    budget: &'a mut JsonValueBudget<R, usize>,

    /// Root-inclusive depth assigned to the inspected value.
    root_depth: usize,
}

impl<'a, R> JsonLexicalPreflight<'a, R>
where
    R: Clone,
{
    /// Creates a lexical preflight bound to one mutable value budget.
    pub(in crate::serde) const fn new(
        budget: &'a mut JsonValueBudget<R, usize>,
    ) -> Self {
        Self {
            budget,
            root_depth: 1,
        }
    }

    /// Creates a lexical preflight rooted at an enclosing serializer depth.
    pub(in crate::serde) const fn at_depth(
        budget: &'a mut JsonValueBudget<R, usize>,
        root_depth: usize,
    ) -> Self {
        Self { budget, root_depth }
    }

    /// Validates and charges one complete JSON document.
    ///
    /// # Errors
    ///
    /// Returns [`JsonSerdeError::Budget`] for the first resource violation, or
    /// [`JsonSerdeError::Json`] when `input` is not one complete JSON value.
    pub(in crate::serde) fn inspect(
        &mut self,
        input: &[u8],
    ) -> Result<(), JsonSerdeError<R>> {
        let mut cursor = JsonCursor::new(input, self.budget);
        let mut stack = Vec::new();
        cursor.skip_whitespace();
        cursor.value(self.root_depth, &mut stack)?;
        while let Some(frame) = stack.pop() {
            cursor.resume(frame, &mut stack)?;
        }
        cursor.skip_whitespace();
        if cursor.position == input.len() {
            Ok(())
        } else {
            Err(invalid_json())
        }
    }
}

/// Continuation for one JSON container being scanned iteratively.
enum ContainerFrame {
    /// An array ready for its first or next value.
    ArrayValue {
        /// Root-inclusive depth of the array.
        depth: usize,

        /// Items already admitted in this array.
        items: usize,
    },

    /// An array waiting for a comma or closing bracket.
    ArrayDelimiter {
        /// Root-inclusive depth of the array.
        depth: usize,

        /// Items already admitted in this array.
        items: usize,
    },

    /// An object ready for its first or next key.
    ObjectKey {
        /// Root-inclusive depth of the object.
        depth: usize,

        /// Entries already admitted in this object.
        entries: usize,
    },

    /// An object waiting for a comma or closing brace.
    ObjectDelimiter {
        /// Root-inclusive depth of the object.
        depth: usize,

        /// Entries already admitted in this object.
        entries: usize,
    },
}

/// Iterative cursor over the JSON bytes being admitted.
struct JsonCursor<'a, 'budget, R> {
    /// Complete JSON input.
    input: &'a [u8],

    /// Current input position.
    position: usize,

    /// Value budget charged by lexical admission.
    budget: &'budget mut JsonValueBudget<R, usize>,
}

impl<'a, 'budget, R> JsonCursor<'a, 'budget, R>
where
    R: Clone,
{
    /// Creates a cursor positioned at the beginning of `input`.
    const fn new(
        input: &'a [u8],
        budget: &'budget mut JsonValueBudget<R, usize>,
    ) -> Self {
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

    /// Admits one JSON value and schedules any container continuation.
    fn value(
        &mut self,
        depth: usize,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), JsonSerdeError<R>> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => {
                self.budget
                    .enter_node(depth)
                    .map_err(JsonSerdeError::Budget)?;
                self.position += 1;
                stack.push(ContainerFrame::ObjectKey { depth, entries: 0 });
                Ok(())
            }
            Some(b'[') => {
                self.budget
                    .enter_node(depth)
                    .map_err(JsonSerdeError::Budget)?;
                self.position += 1;
                stack.push(ContainerFrame::ArrayValue { depth, items: 0 });
                Ok(())
            }
            Some(b'"') => {
                self.budget
                    .enter_node(depth)
                    .map_err(JsonSerdeError::Budget)?;
                let bytes = self.string_bytes()?;
                self.budget
                    .consume_string_bytes(bytes)
                    .map_err(JsonSerdeError::Budget)
            }
            Some(b'-' | b'0'..=b'9') => {
                self.budget
                    .enter_node(depth)
                    .map_err(JsonSerdeError::Budget)?;
                let bytes = self.number_bytes()?;
                self.budget
                    .consume_number_bytes(bytes)
                    .map_err(JsonSerdeError::Budget)
            }
            Some(b't') => self.literal(depth, b"true"),
            Some(b'f') => self.literal(depth, b"false"),
            Some(b'n') => self.literal(depth, b"null"),
            _ => Err(invalid_json()),
        }
    }

    /// Charges and consumes one scalar literal.
    fn literal(
        &mut self,
        depth: usize,
        literal: &[u8],
    ) -> Result<(), JsonSerdeError<R>> {
        if !self.input[self.position..].starts_with(literal) {
            return Err(invalid_json());
        }
        let end = self.position.saturating_add(literal.len());
        if !is_value_delimiter(self.input.get(end).copied()) {
            return Err(invalid_json());
        }
        self.budget
            .enter_node(depth)
            .map_err(JsonSerdeError::Budget)?;
        self.position = end;
        Ok(())
    }

    /// Resumes a container after its child value has completed.
    fn resume(
        &mut self,
        frame: ContainerFrame,
        stack: &mut Vec<ContainerFrame>,
    ) -> Result<(), JsonSerdeError<R>> {
        match frame {
            ContainerFrame::ArrayValue { depth, items } => {
                self.skip_whitespace();
                if self.peek() == Some(b']') {
                    if items == 0 {
                        self.position += 1;
                        return Ok(());
                    }
                    return Err(invalid_json());
                }
                let items = items.checked_add(1).ok_or_else(invalid_json)?;
                self.budget
                    .check_sequence_items(items)
                    .map_err(JsonSerdeError::Budget)?;
                stack.push(ContainerFrame::ArrayDelimiter { depth, items });
                self.value(depth.saturating_add(1), stack)
            }
            ContainerFrame::ArrayDelimiter { depth, items } => {
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                        stack.push(ContainerFrame::ArrayValue { depth, items });
                        Ok(())
                    }
                    Some(b']') => {
                        self.position += 1;
                        Ok(())
                    }
                    _ => Err(invalid_json()),
                }
            }
            ContainerFrame::ObjectKey { depth, entries } => {
                self.skip_whitespace();
                if self.peek() == Some(b'}') {
                    if entries == 0 {
                        self.position += 1;
                        return Ok(());
                    }
                    return Err(invalid_json());
                }
                if self.peek() != Some(b'"') {
                    return Err(invalid_json());
                }
                let entries =
                    entries.checked_add(1).ok_or_else(invalid_json)?;
                self.budget
                    .check_map_entries(entries)
                    .map_err(JsonSerdeError::Budget)?;
                let bytes = self.string_bytes()?;
                self.budget
                    .consume_key_bytes(bytes)
                    .map_err(JsonSerdeError::Budget)?;
                self.skip_whitespace();
                if self.peek() != Some(b':') {
                    return Err(invalid_json());
                }
                self.position += 1;
                stack.push(ContainerFrame::ObjectDelimiter { depth, entries });
                self.value(depth.saturating_add(1), stack)
            }
            ContainerFrame::ObjectDelimiter { depth, entries } => {
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                        stack
                            .push(ContainerFrame::ObjectKey { depth, entries });
                        Ok(())
                    }
                    Some(b'}') => {
                        self.position += 1;
                        Ok(())
                    }
                    _ => Err(invalid_json()),
                }
            }
        }
    }

    /// Consumes one JSON string and returns its decoded UTF-8 byte length.
    fn string_bytes(&mut self) -> Result<usize, JsonSerdeError<R>> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.position += 1;
        let mut decoded = 0_usize;
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.position += 1;
                    return Ok(decoded);
                }
                Some(b'\\') => {
                    self.position += 1;
                    let bytes = match self.peek() {
                        Some(
                            b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r'
                            | b't',
                        ) => {
                            self.position += 1;
                            1
                        }
                        Some(b'u') => self.unicode_escape_bytes()?,
                        _ => return Err(invalid_json()),
                    };
                    decoded =
                        decoded.checked_add(bytes).ok_or_else(invalid_json)?;
                }
                Some(0x20..=0x7F) => {
                    self.position += 1;
                    decoded =
                        decoded.checked_add(1).ok_or_else(invalid_json)?;
                }
                Some(byte) if byte >= 0x80 => {
                    let width = utf8_width(byte).ok_or_else(invalid_json)?;
                    let end = self
                        .position
                        .checked_add(width)
                        .ok_or_else(invalid_json)?;
                    let text = self
                        .input
                        .get(self.position..end)
                        .ok_or_else(invalid_json)?;
                    let character = std::str::from_utf8(text)
                        .ok()
                        .and_then(|text| text.chars().next())
                        .filter(|character| character.len_utf8() == width)
                        .ok_or_else(invalid_json)?;
                    self.position = end;
                    decoded = decoded
                        .checked_add(character.len_utf8())
                        .ok_or_else(invalid_json)?;
                }
                Some(_) | None => return Err(invalid_json()),
            }
        }
    }

    /// Consumes a Unicode escape and returns its decoded UTF-8 byte length.
    fn unicode_escape_bytes(&mut self) -> Result<usize, JsonSerdeError<R>> {
        debug_assert_eq!(self.peek(), Some(b'u'));
        self.position += 1;
        let first = self.hex_quad()?;
        let scalar = if (0xD800..=0xDBFF).contains(&first) {
            if self
                .input
                .get(self.position..self.position.saturating_add(2))
                != Some(b"\\u")
            {
                return Err(invalid_json());
            }
            self.position += 2;
            let second = self.hex_quad()?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(invalid_json());
            }
            0x1_0000
                + ((u32::from(first) - 0xD800) << 10)
                + (u32::from(second) - 0xDC00)
        } else {
            if (0xDC00..=0xDFFF).contains(&first) {
                return Err(invalid_json());
            }
            u32::from(first)
        };
        char::from_u32(scalar)
            .map(char::len_utf8)
            .ok_or_else(invalid_json)
    }

    /// Consumes four hexadecimal digits from a Unicode escape.
    fn hex_quad(&mut self) -> Result<u16, JsonSerdeError<R>> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(byte @ b'0'..=b'9') => u16::from(byte - b'0'),
                Some(byte @ b'a'..=b'f') => u16::from(byte - b'a' + 10),
                Some(byte @ b'A'..=b'F') => u16::from(byte - b'A' + 10),
                _ => return Err(invalid_json()),
            };
            value = (value << 4) | digit;
            self.position += 1;
        }
        Ok(value)
    }

    /// Consumes one JSON number and returns its original lexical byte length.
    fn number_bytes(&mut self) -> Result<usize, JsonSerdeError<R>> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        match self.peek() {
            Some(b'0') => self.position += 1,
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(invalid_json()),
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            self.consume_digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            self.consume_digits()?;
        }
        if !is_value_delimiter(self.peek()) {
            return Err(invalid_json());
        }
        Ok(self.position - start)
    }

    /// Consumes the required digits following a decimal point or exponent.
    fn consume_digits(&mut self) -> Result<(), JsonSerdeError<R>> {
        if !matches!(self.peek(), Some(b'0'..=b'9')) {
            return Err(invalid_json());
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        Ok(())
    }
}

/// Tests whether the next byte can follow a complete JSON scalar value.
const fn is_value_delimiter(byte: Option<u8>) -> bool {
    matches!(
        byte,
        None | Some(b' ' | b'\n' | b'\r' | b'\t' | b',' | b']' | b'}')
    )
}

/// Returns the UTF-8 width encoded by one leading byte, when valid in width.
const fn utf8_width(byte: u8) -> Option<usize> {
    match byte {
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

/// Constructs the JSON error used for lexical syntax rejections.
fn invalid_json<R>() -> JsonSerdeError<R> {
    JsonSerdeError::Json(serde_json::Error::custom("invalid JSON input"))
}
