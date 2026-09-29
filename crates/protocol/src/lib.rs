use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

const MAX_JSON_DEPTH: usize = 128;
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonInputLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonInputErrorKind {
    DuplicateProperty,
    InputTooLarge,
    InvalidJson,
    InvalidLimits,
    InvalidNumber,
    NegativeZero,
    NestingLimit,
    UnsafeNumber,
}

impl JsonInputErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateProperty => "duplicate_property",
            Self::InputTooLarge => "input_too_large",
            Self::InvalidJson => "invalid_json",
            Self::InvalidLimits => "invalid_limits",
            Self::InvalidNumber => "invalid_number",
            Self::NegativeZero => "negative_zero",
            Self::NestingLimit => "nesting_limit",
            Self::UnsafeNumber => "unsafe_number",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonInputError {
    kind: JsonInputErrorKind,
}

impl JsonInputError {
    const fn new(kind: JsonInputErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> &'static str {
        self.kind.as_str()
    }
}

impl fmt::Display for JsonInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind())
    }
}

impl std::error::Error for JsonInputError {}

/// Parse bounded protocol JSON before schema validation or durable identity work.
///
/// Callers must provide the byte and nesting limits from their trusted
/// transport or operation policy. No unbounded default is used.
pub fn parse_json_input(
    input: &str,
    limits: JsonInputLimits,
) -> Result<serde_json::Value, JsonInputError> {
    if limits.max_depth > MAX_JSON_DEPTH {
        return Err(JsonInputError::new(JsonInputErrorKind::InvalidLimits));
    }
    if input.len() > limits.max_bytes {
        return Err(JsonInputError::new(JsonInputErrorKind::InputTooLarge));
    }

    JsonInputScanner::new(input, limits.max_depth).scan()?;
    let mut deserializer = serde_json::Deserializer::from_str(input);
    // The preflight scanner already enforced the shared 128-container ceiling.
    deserializer.disable_recursion_limit();
    let value = serde_json::Value::deserialize(&mut deserializer)
        .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidJson))?;
    deserializer
        .end()
        .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidJson))?;
    Ok(value)
}

struct JsonInputScanner<'a> {
    input: &'a str,
    bytes: &'a [u8],
    position: usize,
    max_depth: usize,
}

impl<'a> JsonInputScanner<'a> {
    fn new(input: &'a str, max_depth: usize) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
            max_depth,
        }
    }

    fn scan(mut self) -> Result<(), JsonInputError> {
        self.skip_whitespace();
        self.scan_value(0)?;
        self.skip_whitespace();
        if self.position != self.bytes.len() {
            return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
        }
        Ok(())
    }

    fn scan_value(&mut self, depth: usize) -> Result<(), JsonInputError> {
        self.skip_whitespace();
        match self.bytes.get(self.position).copied() {
            Some(b'"') => self.scan_string().map(|_| ()),
            Some(b'{') => {
                let nested_depth = depth + 1;
                if nested_depth > self.max_depth {
                    return Err(JsonInputError::new(JsonInputErrorKind::NestingLimit));
                }
                self.scan_object(nested_depth)
            }
            Some(b'[') => {
                let nested_depth = depth + 1;
                if nested_depth > self.max_depth {
                    return Err(JsonInputError::new(JsonInputErrorKind::NestingLimit));
                }
                self.scan_array(nested_depth)
            }
            Some(b't') => self.scan_literal(b"true"),
            Some(b'f') => self.scan_literal(b"false"),
            Some(b'n') => self.scan_literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.scan_number(),
            _ => Err(JsonInputError::new(JsonInputErrorKind::InvalidJson)),
        }
    }

    fn scan_object(&mut self, depth: usize) -> Result<(), JsonInputError> {
        self.position += 1;
        self.skip_whitespace();
        if self.consume(b'}') {
            return Ok(());
        }

        let mut property_names = HashSet::new();
        loop {
            if self.bytes.get(self.position) != Some(&b'"') {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
            let property_name = self.scan_string()?;
            if !property_names.insert(property_name) {
                return Err(JsonInputError::new(JsonInputErrorKind::DuplicateProperty));
            }

            self.skip_whitespace();
            if !self.consume(b':') {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
            self.scan_value(depth)?;
            self.skip_whitespace();

            if self.consume(b'}') {
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
            self.skip_whitespace();
        }
    }

    fn scan_array(&mut self, depth: usize) -> Result<(), JsonInputError> {
        self.position += 1;
        self.skip_whitespace();
        if self.consume(b']') {
            return Ok(());
        }

        loop {
            self.scan_value(depth)?;
            self.skip_whitespace();
            if self.consume(b']') {
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
            self.skip_whitespace();
        }
    }

    fn scan_string(&mut self) -> Result<String, JsonInputError> {
        let start = self.position;
        self.position += 1;
        while let Some(byte) = self.bytes.get(self.position).copied() {
            match byte {
                b'"' => {
                    self.position += 1;
                    return serde_json::from_str(&self.input[start..self.position])
                        .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidJson));
                }
                b'\\' => {
                    self.position += 1;
                    match self.bytes.get(self.position).copied() {
                        Some(b'u') => {
                            self.position += 1;
                            for _ in 0..4 {
                                if !self
                                    .bytes
                                    .get(self.position)
                                    .is_some_and(u8::is_ascii_hexdigit)
                                {
                                    return Err(JsonInputError::new(
                                        JsonInputErrorKind::InvalidJson,
                                    ));
                                }
                                self.position += 1;
                            }
                        }
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.position += 1;
                        }
                        _ => {
                            return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
                        }
                    }
                }
                0x00..=0x1f => {
                    return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
                }
                _ => self.position += 1,
            }
        }
        Err(JsonInputError::new(JsonInputErrorKind::InvalidJson))
    }

    fn scan_number(&mut self) -> Result<(), JsonInputError> {
        let start = self.position;
        self.consume(b'-');

        match self.bytes.get(self.position).copied() {
            Some(b'0') => {
                self.position += 1;
                if self
                    .bytes
                    .get(self.position)
                    .is_some_and(u8::is_ascii_digit)
                {
                    return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
                }
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while self
                    .bytes
                    .get(self.position)
                    .is_some_and(u8::is_ascii_digit)
                {
                    self.position += 1;
                }
            }
            _ => return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson)),
        }

        if self.consume(b'.') {
            let fraction_start = self.position;
            while self
                .bytes
                .get(self.position)
                .is_some_and(u8::is_ascii_digit)
            {
                self.position += 1;
            }
            if self.position == fraction_start {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
        }

        if matches!(self.bytes.get(self.position), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.bytes.get(self.position), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let exponent_start = self.position;
            while self
                .bytes
                .get(self.position)
                .is_some_and(u8::is_ascii_digit)
            {
                self.position += 1;
            }
            if self.position == exponent_start {
                return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
            }
        }

        let token = &self.input[start..self.position];
        let value = token
            .parse::<f64>()
            .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidNumber))?;
        if !value.is_finite() {
            return Err(JsonInputError::new(JsonInputErrorKind::InvalidNumber));
        }
        if value == 0.0 && value.is_sign_negative() {
            return Err(JsonInputError::new(JsonInputErrorKind::NegativeZero));
        }
        if value.fract() != 0.0 || value.abs() > MAX_SAFE_INTEGER {
            return Err(JsonInputError::new(JsonInputErrorKind::UnsafeNumber));
        }
        Ok(())
    }

    fn scan_literal(&mut self, literal: &[u8]) -> Result<(), JsonInputError> {
        if self.bytes.get(self.position..self.position + literal.len()) != Some(literal) {
            return Err(JsonInputError::new(JsonInputErrorKind::InvalidJson));
        }
        self.position += literal.len();
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(
            self.bytes.get(self.position),
            Some(b' ' | b'\t' | b'\n' | b'\r')
        ) {
            self.position += 1;
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.bytes.get(self.position) != Some(&byte) {
            return false;
        }
        self.position += 1;
        true
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Service {
    ControlApi,
    Host,
    Supervisor,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthV1 {
    pub schema_version: u8,
    pub service: Service,
    pub status: String,
    pub execution_available: bool,
}
impl HealthV1 {
    pub fn scaffold(service: Service) -> Self {
        Self {
            schema_version: 1,
            service,
            status: "scaffold".into(),
            execution_available: false,
        }
    }
    pub fn parse(input: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(input).map_err(|e| e.to_string())?;
        if value.schema_version != 1 || value.status != "scaffold" || value.execution_available {
            return Err("Invalid or unsupported health envelope".into());
        }
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct JsonInputVector {
        id: String,
        json: String,
        expected: String,
    }

    #[test]
    fn matches_shared_command_json_input_vectors() {
        let vectors: Vec<JsonInputVector> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/command-json-inputs.v1.json"
        ))
        .unwrap();
        let limits = JsonInputLimits {
            max_bytes: 16_384,
            max_depth: 128,
        };

        for vector in vectors {
            match parse_json_input(&vector.json, limits) {
                Ok(_) => assert_eq!(vector.expected, "valid", "{}", vector.id),
                Err(error) => assert_eq!(error.kind(), vector.expected, "{}", vector.id),
            }
        }
    }

    #[test]
    fn enforces_the_caller_provided_input_byte_and_nesting_limits() {
        assert_eq!(
            parse_json_input(
                "[0,0,0]",
                JsonInputLimits {
                    max_bytes: 6,
                    max_depth: 128,
                }
            )
            .unwrap_err()
            .kind(),
            "input_too_large"
        );
        assert_eq!(
            parse_json_input(
                "[[[0]]]",
                JsonInputLimits {
                    max_bytes: 16_384,
                    max_depth: 2,
                }
            )
            .unwrap_err()
            .kind(),
            "nesting_limit"
        );

        let nested_at_limit = format!("{}0{}", "[".repeat(128), "]".repeat(128));
        assert!(
            parse_json_input(
                &nested_at_limit,
                JsonInputLimits {
                    max_bytes: 1024,
                    max_depth: 128,
                }
            )
            .is_ok()
        );
        assert_eq!(
            parse_json_input(
                "0",
                JsonInputLimits {
                    max_bytes: 1,
                    max_depth: 129,
                }
            )
            .unwrap_err()
            .kind(),
            "invalid_limits"
        );

        let unicode = "{\"é\":\"😀\"}";
        assert_eq!(
            parse_json_input(
                unicode,
                JsonInputLimits {
                    max_bytes: unicode.len() - 1,
                    max_depth: 128,
                }
            )
            .unwrap_err()
            .kind(),
            "input_too_large"
        );
    }

    #[test]
    fn accepts_shared_fixture() {
        let value = HealthV1::parse(include_str!(
            "../../../contracts/fixtures/health.valid.json"
        ))
        .unwrap();
        assert_eq!(value, HealthV1::scaffold(Service::Host));
    }
    #[test]
    fn rejects_execution_claim() {
        assert!(
            HealthV1::parse(include_str!(
                "../../../contracts/fixtures/health.invalid.json"
            ))
            .is_err()
        );
    }
    #[test]
    fn rejects_unknown_version_and_fields() {
        assert!(HealthV1::parse(r#"{"schema_version":2,"service":"host","status":"scaffold","execution_available":false}"#).is_err());
        assert!(HealthV1::parse(r#"{"schema_version":1,"service":"host","status":"scaffold","execution_available":false,"token":"x"}"#).is_err());
    }
}
