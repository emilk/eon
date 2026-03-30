use core::fmt;

use crate::{
    Event, EventSink, Scalar, SpannedEvent, VariantName, write_scalar, write_variant_name,
};

/// Errors produced by [`EventWriter`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializeError {
    /// Writing to the underlying formatter failed.
    Fmt,
    /// The event stream exceeded the configured stack depth.
    DepthLimitExceeded,
    /// The event stream is structurally invalid for the compact writer.
    UnexpectedEvent(&'static str),
    /// The writer was finished before a complete root value was emitted.
    IncompleteDocument,
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fmt => f.write_str("formatter write failed"),
            Self::DepthLimitExceeded => f.write_str("serializer nesting depth exceeded"),
            Self::UnexpectedEvent(msg) => f.write_str(msg),
            Self::IncompleteDocument => f.write_str("incomplete document"),
        }
    }
}

impl From<fmt::Error> for SerializeError {
    #[inline]
    fn from(_: fmt::Error) -> Self {
        Self::Fmt
    }
}

/// Compact event-driven Eon writer with a fixed-size stack.
pub struct EventWriter<W, const N: usize> {
    writer: W,
    root: Frame,
    frames: [Frame; N],
    depth: usize,
}

impl<W, const N: usize> EventWriter<W, N>
where
    W: fmt::Write,
{
    /// Create a new event writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            root: Frame::Root { has_value: false },
            frames: [Frame::Root { has_value: false }; N],
            depth: 0,
        }
    }

    /// Consume a single event.
    pub fn write(&mut self, event: Event<'_>) -> Result<(), SerializeError> {
        match event {
            Event::BeginMap { implicit } => {
                if implicit && self.depth != 0 {
                    return Err(SerializeError::UnexpectedEvent(
                        "implicit maps are only valid at the root",
                    ));
                }
                self.before_value_start()?;
                self.push(Frame::Map {
                    implicit,
                    entries: 0,
                    phase: MapPhase::ExpectKeyMarker,
                })?;
                if !implicit {
                    self.writer.write_char('{')?;
                }
            }
            Event::EndMap => {
                let frame = self.pop()?;
                let Frame::Map {
                    implicit, phase, ..
                } = frame
                else {
                    return Err(SerializeError::UnexpectedEvent(
                        "end-map without a matching begin-map",
                    ));
                };
                if phase != MapPhase::ExpectKeyMarker {
                    return Err(SerializeError::UnexpectedEvent(
                        "map ended while waiting for a key or value",
                    ));
                }
                if !implicit {
                    self.writer.write_char('}')?;
                }
                self.value_completed()?;
            }
            Event::MapKey => {
                let (entries, phase) = match self.top() {
                    Frame::Map { entries, phase, .. } => (entries, phase),
                    _ => {
                        return Err(SerializeError::UnexpectedEvent("map-key outside of a map"));
                    }
                };
                if phase != MapPhase::ExpectKeyMarker {
                    return Err(SerializeError::UnexpectedEvent(
                        "map-key arrived while another map item was incomplete",
                    ));
                }
                if entries > 0 {
                    self.writer.write_str(", ")?;
                }

                let Frame::Map { phase, .. } = self.top_mut() else {
                    return Err(SerializeError::UnexpectedEvent("map-key outside of a map"));
                };
                *phase = MapPhase::WritingKey;
            }
            Event::MapValue => {
                let phase = match self.top() {
                    Frame::Map { phase, .. } => phase,
                    _ => {
                        return Err(SerializeError::UnexpectedEvent(
                            "map-value outside of a map",
                        ));
                    }
                };
                if phase != MapPhase::ExpectValueMarker {
                    return Err(SerializeError::UnexpectedEvent(
                        "map-value arrived before the key completed",
                    ));
                }

                self.writer.write_str(": ")?;

                let Frame::Map { phase, .. } = self.top_mut() else {
                    return Err(SerializeError::UnexpectedEvent(
                        "map-value outside of a map",
                    ));
                };
                *phase = MapPhase::WritingValue;
            }
            Event::BeginList => {
                self.before_value_start()?;
                self.push(Frame::List { first: true })?;
                self.writer.write_char('[')?;
            }
            Event::EndList => {
                let Frame::List { .. } = self.pop()? else {
                    return Err(SerializeError::UnexpectedEvent(
                        "end-list without a matching begin-list",
                    ));
                };
                self.writer.write_char(']')?;
                self.value_completed()?;
            }
            Event::BeginVariant { name } => {
                self.before_value_start()?;
                self.push(Frame::Variant { first: true })?;
                self.write_variant_name(name)?;
                self.writer.write_char('(')?;
            }
            Event::EndVariant => {
                let Frame::Variant { .. } = self.pop()? else {
                    return Err(SerializeError::UnexpectedEvent(
                        "end-variant without a matching begin-variant",
                    ));
                };
                self.writer.write_char(')')?;
                self.value_completed()?;
            }
            Event::Scalar(scalar) => {
                self.before_value_start()?;
                self.write_scalar(scalar)?;
                self.value_completed()?;
            }
        }

        Ok(())
    }

    /// Finish writing and return the wrapped formatter.
    pub fn finish(self) -> Result<W, SerializeError> {
        if self.depth != 0 {
            return Err(SerializeError::IncompleteDocument);
        }

        match self.root {
            Frame::Root { has_value: true } => Ok(self.writer),
            Frame::Root { has_value: false } => Err(SerializeError::IncompleteDocument),
            _ => Err(SerializeError::IncompleteDocument),
        }
    }

    fn before_value_start(&mut self) -> Result<(), SerializeError> {
        match self.top() {
            Frame::Root { has_value: false } => Ok(()),
            Frame::Root { has_value: true } => Err(SerializeError::UnexpectedEvent(
                "multiple root values are not allowed",
            )),
            Frame::List { first } => {
                if !first {
                    self.writer.write_str(", ")?;
                }
                let Frame::List { first } = self.top_mut() else {
                    unreachable!();
                };
                *first = false;
                Ok(())
            }
            Frame::Variant { first } => {
                if !first {
                    self.writer.write_str(", ")?;
                }
                let Frame::Variant { first } = self.top_mut() else {
                    unreachable!();
                };
                *first = false;
                Ok(())
            }
            Frame::Map { phase, .. } => match phase {
                MapPhase::WritingKey | MapPhase::WritingValue => Ok(()),
                MapPhase::ExpectKeyMarker => Err(SerializeError::UnexpectedEvent(
                    "map value started without a preceding map-key marker",
                )),
                MapPhase::ExpectValueMarker => Err(SerializeError::UnexpectedEvent(
                    "map value started without a preceding map-value marker",
                )),
            },
        }
    }

    fn value_completed(&mut self) -> Result<(), SerializeError> {
        match self.top_mut() {
            Frame::Root { has_value } => {
                *has_value = true;
                Ok(())
            }
            Frame::List { .. } | Frame::Variant { .. } => Ok(()),
            Frame::Map { entries, phase, .. } => match phase {
                MapPhase::WritingKey => {
                    *phase = MapPhase::ExpectValueMarker;
                    Ok(())
                }
                MapPhase::WritingValue => {
                    *phase = MapPhase::ExpectKeyMarker;
                    *entries += 1;
                    Ok(())
                }
                MapPhase::ExpectKeyMarker => Err(SerializeError::UnexpectedEvent(
                    "map completed a value without being inside a key or value",
                )),
                MapPhase::ExpectValueMarker => Err(SerializeError::UnexpectedEvent(
                    "map key/value separators are out of order",
                )),
            },
        }
    }

    fn push(&mut self, frame: Frame) -> Result<(), SerializeError> {
        if self.depth + 1 >= N {
            return Err(SerializeError::DepthLimitExceeded);
        }
        self.frames[self.depth] = frame;
        self.depth += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<Frame, SerializeError> {
        if self.depth == 0 {
            return Err(SerializeError::UnexpectedEvent(
                "attempted to close the root frame",
            ));
        }
        self.depth -= 1;
        let frame = self.frames[self.depth];
        self.frames[self.depth] = Frame::Root { has_value: false };
        Ok(frame)
    }

    fn top(&self) -> Frame {
        if self.depth == 0 {
            self.root
        } else {
            self.frames[self.depth - 1]
        }
    }

    fn top_mut(&mut self) -> &mut Frame {
        if self.depth == 0 {
            &mut self.root
        } else {
            &mut self.frames[self.depth - 1]
        }
    }

    fn write_scalar(&mut self, scalar: Scalar<'_>) -> Result<(), SerializeError> {
        write_scalar(&mut self.writer, scalar).map_err(SerializeError::from)
    }

    fn write_variant_name(&mut self, name: VariantName<'_>) -> Result<(), SerializeError> {
        write_variant_name(&mut self.writer, name).map_err(SerializeError::from)
    }
}

impl<'a, W, const N: usize> EventSink<'a> for EventWriter<W, N>
where
    W: fmt::Write,
{
    type Error = SerializeError;

    #[inline]
    fn event(&mut self, event: SpannedEvent<'a>) -> Result<(), Self::Error> {
        self.write(event.event)
    }
}

#[derive(Clone, Copy)]
enum Frame {
    Root {
        has_value: bool,
    },
    Map {
        implicit: bool,
        entries: usize,
        phase: MapPhase,
    },
    List {
        first: bool,
    },
    Variant {
        first: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapPhase {
    ExpectKeyMarker,
    WritingKey,
    ExpectValueMarker,
    WritingValue,
}

#[cfg(test)]
mod tests {
    use std::string::String;

    use crate::{Event, EventWriter, Scalar, SerializeError};

    #[test]
    fn writes_compact_map_and_variant() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 16>::new(&mut out);
        writer.write(Event::BeginMap { implicit: true }).unwrap();
        writer.write(Event::MapKey).unwrap();
        writer
            .write(Event::Scalar(Scalar::Identifier("some_enum")))
            .unwrap();
        writer.write(Event::MapValue).unwrap();
        writer
            .write(Event::BeginVariant {
                name: crate::VariantName::Identifier("EnumValue"),
            })
            .unwrap();
        writer.write(Event::EndVariant).unwrap();
        writer.write(Event::EndMap).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "some_enum: EnumValue()");
    }

    #[test]
    fn zero_capacity_writer_supports_scalar_roots() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 0>::new(&mut out);
        writer.write(Event::Scalar(Scalar::Null)).unwrap();
        writer.finish().unwrap();
        assert_eq!(out, "null");
    }

    #[test]
    fn zero_capacity_writer_rejects_containers_without_panicking() {
        let mut out = String::new();
        let mut writer = EventWriter::<_, 0>::new(&mut out);
        assert_eq!(
            writer.write(Event::BeginList),
            Err(SerializeError::DepthLimitExceeded)
        );
        assert_eq!(out, "");
    }
}
