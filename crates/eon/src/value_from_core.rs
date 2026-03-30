use std::{borrow::Cow, str::FromStr as _};

use eon_core::{
    Event, EventSink, ParseError, Scalar, Span as CoreSpan, SpannedEvent, VariantName, parse,
};
use eon_syntax::Span;

use crate::{
    Error, Map, Number, Result, Value, Variant,
    core_string::{decode_string_token, decode_variant_name},
};

/// Parse an Eon document into an owned [`Value`] using the experimental
/// `eon_core` event parser instead of the existing `eon_syntax` parser.
pub fn from_str_with_core(eon_source: &str) -> Result<Value> {
    let mut collector = ValueCollector::default();
    match parse(eon_source, &mut collector) {
        Ok(()) => collector.finish().map_err(|err| err.into_error(eon_source)),
        Err(ParseError::Parse(err)) => Err(core_error_to_eon_error(eon_source, err)),
        Err(ParseError::Sink(err)) => Err(err.into_error(eon_source)),
    }
}

#[derive(Default)]
struct ValueCollector {
    stack: Vec<Frame>,
    root: Option<Value>,
}

#[derive(Debug)]
enum Frame {
    Map {
        map: Map,
        pending_key: Option<Value>,
        phase: MapPhase,
    },
    List {
        values: Vec<Value>,
    },
    Variant {
        name: String,
        values: Vec<Value>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapPhase {
    ExpectKeyMarker,
    WritingKey,
    ExpectValueMarker,
    WritingValue,
}

#[derive(Debug)]
enum CollectErrorKind {
    InvalidState(&'static str),
    InvalidString(String),
    InvalidNumber(String),
    DuplicateKey,
    MissingRoot,
}

#[derive(Debug)]
struct CollectError {
    span: Option<CoreSpan>,
    kind: CollectErrorKind,
}

impl CollectError {
    fn at(span: CoreSpan, kind: CollectErrorKind) -> Self {
        Self {
            span: Some(span),
            kind,
        }
    }

    fn custom(kind: CollectErrorKind) -> Self {
        Self { span: None, kind }
    }

    fn into_error(self, eon_source: &str) -> Error {
        let message = match self.kind {
            CollectErrorKind::InvalidState(msg) => msg.to_owned(),
            CollectErrorKind::InvalidString(msg) => msg,
            CollectErrorKind::InvalidNumber(msg) => msg,
            CollectErrorKind::DuplicateKey => "Duplicate key in map".to_owned(),
            CollectErrorKind::MissingRoot => {
                "Expected the parser to produce a root value".to_owned()
            }
        };

        Error::new(eon_source, self.span.map(core_span_to_syntax_span), message)
    }
}

impl ValueCollector {
    fn finish(self) -> core::result::Result<Value, CollectError> {
        if !self.stack.is_empty() {
            return Err(CollectError::custom(CollectErrorKind::InvalidState(
                "Parser finished with unterminated containers",
            )));
        }

        self.root
            .ok_or_else(|| CollectError::custom(CollectErrorKind::MissingRoot))
    }

    fn push_value(
        &mut self,
        value: Value,
        span: CoreSpan,
    ) -> core::result::Result<(), CollectError> {
        let Some(frame) = self.stack.last_mut() else {
            if self.root.replace(value).is_some() {
                return Err(CollectError::at(
                    span,
                    CollectErrorKind::InvalidState("Parser emitted multiple root values"),
                ));
            }
            return Ok(());
        };

        match frame {
            Frame::List { values } => {
                values.push(value);
                Ok(())
            }
            Frame::Variant { values, .. } => {
                values.push(value);
                Ok(())
            }
            Frame::Map {
                map,
                pending_key,
                phase,
            } => match phase {
                MapPhase::WritingKey => {
                    *pending_key = Some(value);
                    *phase = MapPhase::ExpectValueMarker;
                    Ok(())
                }
                MapPhase::WritingValue => {
                    let Some(key) = pending_key.take() else {
                        return Err(CollectError::at(
                            span,
                            CollectErrorKind::InvalidState("Missing map key before map value"),
                        ));
                    };
                    if map.insert(key, value).is_some() {
                        return Err(CollectError::at(span, CollectErrorKind::DuplicateKey));
                    }
                    *phase = MapPhase::ExpectKeyMarker;
                    Ok(())
                }
                MapPhase::ExpectKeyMarker => Err(CollectError::at(
                    span,
                    CollectErrorKind::InvalidState(
                        "Map received a value without a preceding key marker",
                    ),
                )),
                MapPhase::ExpectValueMarker => Err(CollectError::at(
                    span,
                    CollectErrorKind::InvalidState(
                        "Map received a value without a preceding value marker",
                    ),
                )),
            },
        }
    }

    fn handle_scalar(
        &mut self,
        scalar: Scalar<'_>,
        span: CoreSpan,
    ) -> core::result::Result<(), CollectError> {
        let value = if self.map_key_context() {
            scalar_to_key_value(scalar, span)?
        } else {
            scalar_to_value(scalar, span)?
        };
        self.push_value(value, span)
    }

    fn map_key_context(&self) -> bool {
        matches!(
            self.stack.last(),
            Some(Frame::Map {
                phase: MapPhase::WritingKey,
                ..
            })
        )
    }

    fn begin_variant(
        &mut self,
        name: VariantName<'_>,
        span: CoreSpan,
    ) -> core::result::Result<(), CollectError> {
        let name = decode_variant_name(name)
            .map(Cow::into_owned)
            .map_err(|err| CollectError::at(span, CollectErrorKind::InvalidString(err)))?;

        self.stack.push(Frame::Variant {
            name,
            values: Vec::new(),
        });
        Ok(())
    }
}

impl<'a> EventSink<'a> for ValueCollector {
    type Error = CollectError;

    fn event(&mut self, event: SpannedEvent<'a>) -> core::result::Result<(), Self::Error> {
        let SpannedEvent { span, event } = event;

        match event {
            Event::BeginMap { .. } => {
                self.stack.push(Frame::Map {
                    map: Map::new(),
                    pending_key: None,
                    phase: MapPhase::ExpectKeyMarker,
                });
                Ok(())
            }
            Event::EndMap => {
                let Some(Frame::Map { map, phase, .. }) = self.stack.pop() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndMap without BeginMap"),
                    ));
                };
                if phase != MapPhase::ExpectKeyMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState(
                            "Map ended while a key or value was incomplete",
                        ),
                    ));
                }
                self.push_value(Value::Map(map), span)
            }
            Event::MapKey => {
                let Some(Frame::Map { phase, .. }) = self.stack.last_mut() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("MapKey outside of a map"),
                    ));
                };
                if *phase != MapPhase::ExpectKeyMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("Unexpected MapKey marker"),
                    ));
                }
                *phase = MapPhase::WritingKey;
                Ok(())
            }
            Event::MapValue => {
                let Some(Frame::Map { phase, .. }) = self.stack.last_mut() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("MapValue outside of a map"),
                    ));
                };
                if *phase != MapPhase::ExpectValueMarker {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("Unexpected MapValue marker"),
                    ));
                }
                *phase = MapPhase::WritingValue;
                Ok(())
            }
            Event::BeginList => {
                self.stack.push(Frame::List { values: Vec::new() });
                Ok(())
            }
            Event::EndList => {
                let Some(Frame::List { values }) = self.stack.pop() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndList without BeginList"),
                    ));
                };
                self.push_value(Value::List(values), span)
            }
            Event::BeginVariant { name } => self.begin_variant(name, span),
            Event::EndVariant => {
                let Some(Frame::Variant { name, values }) = self.stack.pop() else {
                    return Err(CollectError::at(
                        span,
                        CollectErrorKind::InvalidState("EndVariant without BeginVariant"),
                    ));
                };
                self.push_value(Value::Variant(Variant { name, values }), span)
            }
            Event::Scalar(scalar) => self.handle_scalar(scalar, span),
        }
    }
}

fn scalar_to_value(
    scalar: Scalar<'_>,
    span: CoreSpan,
) -> core::result::Result<Value, CollectError> {
    match scalar {
        Scalar::Null => Ok(Value::Null),
        Scalar::Bool(value) => Ok(Value::Bool(value)),
        Scalar::Number(raw) => Number::from_str(raw)
            .map(Value::Number)
            .map_err(|err| CollectError::at(span, CollectErrorKind::InvalidNumber(err))),
        Scalar::Identifier(identifier) => Ok(Value::new_variant(identifier.to_owned(), vec![])),
        Scalar::String(token) => decode_string_token(token)
            .map(Cow::into_owned)
            .map(Value::String)
            .map_err(|err| CollectError::at(span, CollectErrorKind::InvalidString(err))),
    }
}

fn scalar_to_key_value(
    scalar: Scalar<'_>,
    span: CoreSpan,
) -> core::result::Result<Value, CollectError> {
    match scalar {
        Scalar::Null => Ok(Value::String("null".to_owned())),
        Scalar::Bool(true) => Ok(Value::String("true".to_owned())),
        Scalar::Bool(false) => Ok(Value::String("false".to_owned())),
        Scalar::Identifier(identifier) => Ok(Value::String(identifier.to_owned())),
        Scalar::Number(raw) => Number::from_str(raw)
            .map(Value::Number)
            .map_err(|err| CollectError::at(span, CollectErrorKind::InvalidNumber(err))),
        Scalar::String(token) => decode_string_token(token)
            .map(Cow::into_owned)
            .map(Value::String)
            .map_err(|err| CollectError::at(span, CollectErrorKind::InvalidString(err))),
    }
}

fn core_error_to_eon_error(eon_source: &str, error: eon_core::Error) -> Error {
    Error::new_at(
        eon_source,
        core_span_to_syntax_span(error.span),
        error.kind.to_string(),
    )
}

fn core_span_to_syntax_span(span: CoreSpan) -> Span {
    Span {
        start: span.start,
        end: span.end,
    }
}
