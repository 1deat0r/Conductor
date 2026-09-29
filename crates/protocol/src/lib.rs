use jsonschema::{Validator, validator_for};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json_canonicalizer::to_vec as jcs_to_vec;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;

const MAX_JSON_DEPTH: usize = 128;
const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_991.0;
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
const COMMAND_DIGEST_DOMAIN: &[u8] = b"Conductor.CommandV1\0";
const SUBMISSION_FINGERPRINT_DOMAIN: &[u8] = b"Conductor.CommandSubmissionV1\0";

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
    let mut value = serde_json::Value::deserialize(&mut deserializer)
        .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidJson))?;
    deserializer
        .end()
        .map_err(|_| JsonInputError::new(JsonInputErrorKind::InvalidJson))?;
    normalize_integer_numbers(&mut value);
    Ok(value)
}

fn normalize_integer_numbers(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Number(number) => {
            if let Some(value) = number
                .as_f64()
                .filter(|value| value.fract() == 0.0 && value.abs() <= MAX_SAFE_INTEGER_F64)
            {
                *number = serde_json::Number::from(value as i64);
            }
        }
        serde_json::Value::Array(values) => {
            for item in values {
                normalize_integer_numbers(item);
            }
        }
        serde_json::Value::Object(values) => {
            for item in values.values_mut() {
                normalize_integer_numbers(item);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::String(_) => {}
    }
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
        if value.fract() != 0.0 || value.abs() > MAX_SAFE_INTEGER_F64 {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum CommandActionV1 {
    #[serde(rename = "task.create")]
    TaskCreate { title: String, goal: String },
    #[serde(rename = "task.update")]
    TaskUpdate { changes: TaskChangesV1 },
    #[serde(rename = "attempt.start")]
    AttemptStart { launch_digest: String },
    #[serde(rename = "attempt.cancel")]
    AttemptCancel {
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskChangesV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTargetV1 {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionTargetV1 {
    pub aggregate_kind: AggregateKindV1,
    pub aggregate_id: String,
    pub expected_revision: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateKindV1 {
    Task,
    Attempt,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandSubmissionV1 {
    pub schema_version: u8,
    pub command_id: String,
    pub deadline_unix_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<CommandTargetV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision_target: Option<RevisionTargetV1>,
    pub action: CommandActionV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandV1 {
    schema_version: u8,
    command_id: String,
    payload_digest: String,
    submission_fingerprint: String,
    tenant_id: String,
    actor_id: String,
    device_id: String,
    deadline_unix_ms: i64,
    target: CommandTargetV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision_target: Option<RevisionTargetV1>,
    action: CommandActionV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandV1Wire {
    schema_version: u8,
    command_id: String,
    payload_digest: String,
    submission_fingerprint: String,
    tenant_id: String,
    actor_id: String,
    device_id: String,
    deadline_unix_ms: i64,
    target: CommandTargetV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision_target: Option<RevisionTargetV1>,
    action: CommandActionV1,
}
impl From<CommandV1Wire> for CommandV1 {
    fn from(wire: CommandV1Wire) -> Self {
        Self {
            schema_version: wire.schema_version,
            command_id: wire.command_id,
            payload_digest: wire.payload_digest,
            submission_fingerprint: wire.submission_fingerprint,
            tenant_id: wire.tenant_id,
            actor_id: wire.actor_id,
            device_id: wire.device_id,
            deadline_unix_ms: wire.deadline_unix_ms,
            target: wire.target,
            revision_target: wire.revision_target,
            action: wire.action,
        }
    }
}
impl CommandV1 {
    pub fn schema_version(&self) -> u8 {
        self.schema_version
    }
    pub fn command_id(&self) -> &str {
        &self.command_id
    }
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
    pub fn submission_fingerprint(&self) -> &str {
        &self.submission_fingerprint
    }
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    pub fn deadline_unix_ms(&self) -> i64 {
        self.deadline_unix_ms
    }
    pub fn target(&self) -> &CommandTargetV1 {
        &self.target
    }
    pub fn revision_target(&self) -> Option<&RevisionTargetV1> {
        self.revision_target.as_ref()
    }
    pub fn action(&self) -> &CommandActionV1 {
        &self.action
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticatedPrincipalV1 {
    pub tenant_id: String,
    pub actor_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinatorAuthorityV1 {
    pub kind: String,
    pub tenant_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinatorReceiptV1 {
    pub schema_version: u8,
    pub authority: CoordinatorAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub submission_fingerprint: String,
    pub receipt_state: CoordinatorReceiptStateV1,
    pub stored_at_unix_ms: i64,
    pub assigned_target: CommandTargetV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinatorReceiptStateV1 {
    #[serde(rename = "coordinator_stored")]
    CoordinatorStored,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostAuthorityV1 {
    pub kind: String,
    pub host_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingHostReceiptV1 {
    pub schema_version: u8,
    pub authority: HostAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub receipt_state: PendingHostReceiptStateV1,
    pub updated_at_unix_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingHostReceiptStateV1 {
    #[serde(rename = "pending_host_admission")]
    PendingHostAdmission,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostAcceptedReceiptV1 {
    pub schema_version: u8,
    pub authority: HostAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub receipt_state: HostAcceptedReceiptStateV1,
    pub updated_at_unix_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostAcceptedReceiptStateV1 {
    #[serde(rename = "host_accepted")]
    HostAccepted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostRejectedReceiptV1 {
    pub schema_version: u8,
    pub authority: HostAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub receipt_state: HostRejectedReceiptStateV1,
    pub updated_at_unix_ms: i64,
    pub rejection: HostRejectionV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostRejectedReceiptStateV1 {
    #[serde(rename = "host_rejected")]
    HostRejected,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostRejectionV1 {
    pub code: HostRejectionCodeV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRejectionCodeV1 {
    StaleRevision,
    Expired,
    PermissionChanged,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectResolvedReceiptV1 {
    pub schema_version: u8,
    pub authority: HostAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub receipt_state: EffectResolvedReceiptStateV1,
    pub updated_at_unix_ms: i64,
    pub resolution: HostResolutionV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectResolvedReceiptStateV1 {
    #[serde(rename = "effect_resolved")]
    EffectResolved,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HostResolutionV1 {
    WithDigest {
        resolution_id: String,
        outcome: EffectOutcomeV1,
        result_digest: String,
    },
    NoResult {
        resolution_id: String,
        outcome: EffectOutcomeV1,
        no_result: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectOutcomeV1 {
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeUnknownReceiptV1 {
    pub schema_version: u8,
    pub authority: HostAuthorityV1,
    pub command_id: String,
    pub payload_digest: String,
    pub receipt_state: OutcomeUnknownReceiptStateV1,
    pub updated_at_unix_ms: i64,
    pub unknown_reason: UnknownReasonV1,
    pub reconciliation_required: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeUnknownReceiptStateV1 {
    #[serde(rename = "outcome_unknown")]
    OutcomeUnknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownReasonV1 {
    #[serde(rename = "ambiguous_history")]
    AmbiguousHistory,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HostReceiptV1 {
    Pending(PendingHostReceiptV1),
    Accepted(HostAcceptedReceiptV1),
    Rejected(HostRejectedReceiptV1),
    Resolved(EffectResolvedReceiptV1),
    Unknown(OutcomeUnknownReceiptV1),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventAggregateV1 {
    pub kind: EventAggregateKindV1,
    pub id: String,
    pub sequence: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventAggregateKindV1 {
    Task,
    Attempt,
    Session,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEventTypeV1 {
    #[serde(rename = "task.created")]
    TaskCreated,
    #[serde(rename = "task.updated")]
    TaskUpdated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttemptStateChangedEventTypeV1 {
    #[serde(rename = "attempt.state_changed")]
    AttemptStateChanged,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequiredEventClassV1 {
    #[serde(rename = "required")]
    Required,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InformationalEventClassV1 {
    #[serde(rename = "informational")]
    Informational,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStateV1 {
    Queued,
    Provisioning,
    Running,
    WaitingForInput,
    WaitingForApproval,
    Validating,
    ReviewReady,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    OutcomeUnknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEventAggregateV1 {
    pub kind: TaskAggregateKindV1,
    pub id: String,
    pub sequence: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAggregateKindV1 {
    Task,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptEventAggregateV1 {
    pub kind: AttemptAggregateKindV1,
    pub id: String,
    pub sequence: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptAggregateKindV1 {
    Attempt,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRevisionPayloadV1 {
    pub task_revision: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptStateChangedPayloadV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<AttemptStateV1>,
    pub to: AttemptStateV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRevisionEventV1 {
    pub schema_version: u8,
    pub event_id: String,
    pub event_type: TaskEventTypeV1,
    pub event_class: RequiredEventClassV1,
    pub aggregate: TaskEventAggregateV1,
    pub command_id: String,
    pub causation_id: Option<String>,
    pub actor_id: String,
    pub ownership_generation: i64,
    pub occurred_at_unix_ms: i64,
    pub payload: TaskRevisionPayloadV1,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptStateChangedEventV1 {
    pub schema_version: u8,
    pub event_id: String,
    pub event_type: AttemptStateChangedEventTypeV1,
    pub event_class: RequiredEventClassV1,
    pub aggregate: AttemptEventAggregateV1,
    pub command_id: String,
    pub causation_id: Option<String>,
    pub actor_id: String,
    pub ownership_generation: i64,
    pub occurred_at_unix_ms: i64,
    pub payload: AttemptStateChangedPayloadV1,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownCommandEventV1 {
    TaskRevision(TaskRevisionEventV1),
    AttemptStateChanged(AttemptStateChangedEventV1),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownInformationalCommandEventV1 {
    pub schema_version: u8,
    pub event_id: String,
    pub event_type: String,
    pub event_class: InformationalEventClassV1,
    pub aggregate: EventAggregateV1,
    pub command_id: String,
    pub causation_id: Option<String>,
    pub actor_id: String,
    pub ownership_generation: i64,
    pub occurred_at_unix_ms: i64,
    pub snapshot_cursor: String,
    pub payload: Value,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCommandEventV1 {
    Known(KnownCommandEventV1),
    UnknownInformational {
        event: UnknownInformationalCommandEventV1,
        refresh_snapshot_before_advance: bool,
    },
}

fn parse_strict_json(input: &str, limits: JsonInputLimits) -> Result<Value, String> {
    parse_json_input(input, limits).map_err(|error| error.to_string())
}
type CachedValidator = OnceLock<Result<Validator, String>>;
static COMMAND_ACTION_VALIDATOR: CachedValidator = OnceLock::new();
static COMMAND_SUBMISSION_VALIDATOR: CachedValidator = OnceLock::new();
static COMMAND_VALIDATOR: CachedValidator = OnceLock::new();
static COORDINATOR_RECEIPT_VALIDATOR: CachedValidator = OnceLock::new();
static HOST_RECEIPT_VALIDATOR: CachedValidator = OnceLock::new();
static COMMAND_EVENT_VALIDATOR: CachedValidator = OnceLock::new();

fn validate_schema(schema_text: &str, value: &Value, name: &str) -> Result<(), String> {
    let cache = match name {
        "CommandActionV1" => &COMMAND_ACTION_VALIDATOR,
        "CommandSubmissionV1" => &COMMAND_SUBMISSION_VALIDATOR,
        "CommandV1" => &COMMAND_VALIDATOR,
        "CoordinatorReceiptV1" => &COORDINATOR_RECEIPT_VALIDATOR,
        "HostReceiptV1" => &HOST_RECEIPT_VALIDATOR,
        "CommandEventV1" => &COMMAND_EVENT_VALIDATOR,
        _ => return Err(format!("No validator cache configured for {name}")),
    };
    let validator = cache.get_or_init(|| {
        let schema: Value = serde_json::from_str(schema_text).map_err(|e| e.to_string())?;
        validator_for(&schema).map_err(|e| e.to_string())
    });
    let validator = validator.as_ref().map_err(Clone::clone)?;
    validator
        .validate(value)
        .map_err(|e| format!("Invalid or unsupported {name}: {e}"))
}
fn parse_schema_json<T: DeserializeOwned>(
    input: &str,
    limits: JsonInputLimits,
    schema: &str,
    name: &str,
) -> Result<(Value, T), String> {
    let value = parse_strict_json(input, limits)?;
    validate_schema(schema, &value, name)?;
    let typed = serde_json::from_value(value.clone())
        .map_err(|e| format!("Invalid or unsupported {name}: {e}"))?;
    Ok((value, typed))
}

fn id_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}
fn validate_target_semantics(
    action: &CommandActionV1,
    target: Option<&CommandTargetV1>,
    revision: Option<&RevisionTargetV1>,
    canonical: bool,
) -> Result<(), String> {
    let require_revision = |kind: AggregateKindV1, id: &str| -> Result<(), String> {
        match revision {
            Some(value) if value.aggregate_kind == kind && value.aggregate_id == id => Ok(()),
            _ => Err("Revision target does not match the command target".into()),
        }
    };
    match action {
        CommandActionV1::TaskCreate { .. } => {
            if revision.is_some() || (!canonical && target.is_some()) {
                return Err("Task creation cannot target a pre-existing aggregate".into());
            }
            if canonical && target.is_none_or(|item| item.attempt_id.is_some()) {
                return Err("Canonical task creation requires its assigned task ID".into());
            }
        }
        CommandActionV1::TaskUpdate { .. } => {
            let target = target.ok_or("Task update requires a task target")?;
            if target.attempt_id.is_some() {
                return Err("Task update cannot target an attempt".into());
            }
            require_revision(AggregateKindV1::Task, &target.task_id)?;
        }
        CommandActionV1::AttemptStart { .. } => {
            let target = target.ok_or("Attempt start requires a task target")?;
            if canonical && target.attempt_id.is_none() {
                return Err("Canonical attempt start requires its assigned attempt ID".into());
            }
            if !canonical && target.attempt_id.is_some() {
                return Err("Attempt start submission cannot claim its assigned attempt ID".into());
            }
            require_revision(AggregateKindV1::Task, &target.task_id)?;
        }
        CommandActionV1::AttemptCancel { .. } => {
            let target = target.ok_or("Attempt cancellation requires an attempt target")?;
            let attempt_id = target
                .attempt_id
                .as_deref()
                .ok_or("Attempt cancellation requires an attempt ID")?;
            require_revision(AggregateKindV1::Attempt, attempt_id)?;
        }
    }
    Ok(())
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn parse_command_action_v1_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<CommandActionV1, String> {
    let value = parse_strict_json(input, limits)?;
    validate_schema(
        include_str!("../../../contracts/command-action-v1.schema.json"),
        &value,
        "CommandActionV1",
    )?;
    serde_json::from_value(value).map_err(|e| e.to_string())
}
pub fn parse_command_submission_v1_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<CommandSubmissionV1, String> {
    let (_, submission): (_, CommandSubmissionV1) = parse_schema_json(
        input,
        limits,
        include_str!("../../../contracts/command-submission-v1.schema.json"),
        "CommandSubmissionV1",
    )?;
    if submission.schema_version != 1 {
        return Err("Unsupported CommandSubmissionV1 schema version".into());
    }
    validate_target_semantics(
        &submission.action,
        submission.target.as_ref(),
        submission.revision_target.as_ref(),
        false,
    )?;
    Ok(submission)
}
fn parse_command_v1_value(
    input: &str,
    limits: JsonInputLimits,
) -> Result<(Value, CommandV1), String> {
    let value = parse_strict_json(input, limits)?;
    validate_schema(
        include_str!("../../../contracts/command-v1.schema.json"),
        &value,
        "CommandV1",
    )?;
    let wire: CommandV1Wire = serde_json::from_value(value.clone())
        .map_err(|e| format!("Invalid or unsupported CommandV1: {e}"))?;
    if wire.schema_version != 1 {
        return Err("Unsupported CommandV1 schema version".into());
    }
    validate_target_semantics(
        &wire.action,
        Some(&wire.target),
        wire.revision_target.as_ref(),
        true,
    )?;
    if !valid_digest(&wire.payload_digest) {
        return Err("Invalid CommandV1 payload digest".into());
    }
    Ok((value, wire.into()))
}
pub fn canonicalize_json_text(input: &str, limits: JsonInputLimits) -> Result<String, String> {
    let value = parse_strict_json(input, limits)?;
    let canonical = jcs_to_vec(&value).map_err(|e| e.to_string())?;
    String::from_utf8(canonical).map_err(|e| e.to_string())
}
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn digest_command_value(mut value: Value) -> Result<String, String> {
    let object = value.as_object_mut().ok_or("CommandV1 must be an object")?;
    object.remove("payload_digest");
    let canonical = jcs_to_vec(&value).map_err(|e| e.to_string())?;
    let mut preimage = Vec::with_capacity(COMMAND_DIGEST_DOMAIN.len() + canonical.len());
    preimage.extend_from_slice(COMMAND_DIGEST_DOMAIN);
    preimage.extend_from_slice(&canonical);
    Ok(sha256_hex(&preimage))
}
pub fn compute_command_v1_digest_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<String, String> {
    let (value, _) = parse_command_v1_value(input, limits)?;
    digest_command_value(value)
}
pub fn parse_command_v1_json(input: &str, limits: JsonInputLimits) -> Result<CommandV1, String> {
    let (value, command) = parse_command_v1_value(input, limits)?;
    let digest = digest_command_value(value)?;
    if digest != command.payload_digest {
        return Err("CommandV1 digest mismatch".into());
    }
    Ok(command)
}
pub fn submission_fingerprint_v1_json(
    input: &str,
    principal: &AuthenticatedPrincipalV1,
    limits: JsonInputLimits,
) -> Result<String, String> {
    let (submission_value, submission): (Value, CommandSubmissionV1) = parse_schema_json(
        input,
        limits,
        include_str!("../../../contracts/command-submission-v1.schema.json"),
        "CommandSubmissionV1",
    )?;
    if submission.schema_version != 1 {
        return Err("Unsupported CommandSubmissionV1 schema version".into());
    }
    validate_target_semantics(
        &submission.action,
        submission.target.as_ref(),
        submission.revision_target.as_ref(),
        false,
    )?;
    for id in [
        &principal.tenant_id,
        &principal.actor_id,
        &principal.device_id,
    ] {
        if !id_is_valid(id) {
            return Err("Invalid authenticated principal context".into());
        }
    }
    let context = serde_json::json!({
        "tenant_id": principal.tenant_id,
        "actor_id": principal.actor_id,
        "device_id": principal.device_id,
        "submission": submission_value,
    });
    let canonical = jcs_to_vec(&context).map_err(|e| e.to_string())?;
    let mut preimage = Vec::with_capacity(SUBMISSION_FINGERPRINT_DOMAIN.len() + canonical.len());
    preimage.extend_from_slice(SUBMISSION_FINGERPRINT_DOMAIN);
    preimage.extend_from_slice(&canonical);
    Ok(sha256_hex(&preimage))
}

pub fn deadline_rejection_at(
    deadline_unix_ms: i64,
    now_unix_ms: i64,
) -> Result<Option<HostRejectionCodeV1>, String> {
    if deadline_unix_ms < 0
        || now_unix_ms < 0
        || deadline_unix_ms > MAX_SAFE_INTEGER
        || now_unix_ms > MAX_SAFE_INTEGER
    {
        return Err("Deadline values must be nonnegative safe integers".into());
    }
    Ok((deadline_unix_ms <= now_unix_ms).then_some(HostRejectionCodeV1::Expired))
}

pub fn parse_coordinator_receipt_v1_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<CoordinatorReceiptV1, String> {
    let (_, receipt): (_, CoordinatorReceiptV1) = parse_schema_json(
        input,
        limits,
        include_str!("../../../contracts/coordinator-receipt-v1.schema.json"),
        "CoordinatorReceiptV1",
    )?;
    Ok(receipt)
}
pub fn parse_host_receipt_v1_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<HostReceiptV1, String> {
    let (_, receipt): (_, HostReceiptV1) = parse_schema_json(
        input,
        limits,
        include_str!("../../../contracts/host-receipt-v1.schema.json"),
        "HostReceiptV1",
    )?;
    Ok(receipt)
}
pub fn parse_command_event_v1_json(
    input: &str,
    limits: JsonInputLimits,
) -> Result<ParsedCommandEventV1, String> {
    let value = parse_strict_json(input, limits)?;
    validate_schema(
        include_str!("../../../contracts/command-event-v1.schema.json"),
        &value,
        "CommandEventV1",
    )?;
    let event_type = value
        .get("event_type")
        .and_then(Value::as_str)
        .ok_or("CommandEventV1 is missing event_type")?;
    match event_type {
        "task.created" | "task.updated" => {
            let event: TaskRevisionEventV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            if event.schema_version != 1 {
                return Err("Unsupported CommandEventV1 schema version".into());
            }
            Ok(ParsedCommandEventV1::Known(
                KnownCommandEventV1::TaskRevision(event),
            ))
        }
        "attempt.state_changed" => {
            let event: AttemptStateChangedEventV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            if event.schema_version != 1 {
                return Err("Unsupported CommandEventV1 schema version".into());
            }
            Ok(ParsedCommandEventV1::Known(
                KnownCommandEventV1::AttemptStateChanged(event),
            ))
        }
        _ => {
            let event: UnknownInformationalCommandEventV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            if event.schema_version != 1 {
                return Err("Unsupported CommandEventV1 schema version".into());
            }
            if event.snapshot_cursor.is_empty() {
                return Err(
                    "Unknown events require informational classification and snapshot recovery"
                        .into(),
                );
            }
            Ok(ParsedCommandEventV1::UnknownInformational {
                event,
                refresh_snapshot_before_advance: true,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const TEST_JSON_LIMITS: JsonInputLimits = JsonInputLimits {
        max_bytes: 16_384,
        max_depth: 128,
    };

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
    fn normalizes_integer_valued_decimal_tokens_like_typescript_numbers() {
        let value = parse_json_input(r#"{"integer":1.0,"zero":0.0}"#, TEST_JSON_LIMITS).unwrap();
        assert_eq!(value["integer"].as_i64(), Some(1));
        assert_eq!(value["zero"].as_i64(), Some(0));
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
    fn accepts_shared_health_fixture() {
        let value = HealthV1::parse(include_str!(
            "../../../contracts/fixtures/health.valid.json"
        ))
        .unwrap();
        assert_eq!(value, HealthV1::scaffold(Service::Host));
    }
    #[test]
    fn rejects_health_execution_claim() {
        assert!(
            HealthV1::parse(include_str!(
                "../../../contracts/fixtures/health.invalid.json"
            ))
            .is_err()
        );
    }
    #[test]
    fn rejects_unknown_health_version_and_fields() {
        assert!(HealthV1::parse(r#"{"schema_version":2,"service":"host","status":"scaffold","execution_available":false}"#).is_err());
        assert!(HealthV1::parse(r#"{"schema_version":1,"service":"host","status":"scaffold","execution_available":false,"token":"x"}"#).is_err());
    }

    #[test]
    fn validates_all_operation_actions_and_submissions() {
        for input in [
            include_str!("../../../contracts/fixtures/commands/submission.valid.json"),
            include_str!("../../../contracts/fixtures/commands/submission.task-update.valid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/submission.attempt-start.valid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/submission.attempt-cancel.valid.json"
            ),
        ] {
            let submission = parse_command_submission_v1_json(input, TEST_JSON_LIMITS).unwrap();
            let serialized = serde_json::to_string(&submission).unwrap();
            assert_eq!(
                parse_command_submission_v1_json(&serialized, TEST_JSON_LIMITS).unwrap(),
                submission
            );
            let action = serde_json::to_string(&submission.action).unwrap();
            assert_eq!(
                parse_command_action_v1_json(&action, TEST_JSON_LIMITS).unwrap(),
                submission.action
            );
        }
        for input in [
            include_str!("../../../contracts/fixtures/commands/submission.invalid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/submission.task-create-targeted.invalid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/submission.attempt-start-claimed-attempt.invalid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/submission.bad-revision-target.invalid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/submission.attempt-cancel-bad-revision.invalid.json"
            ),
        ] {
            assert!(parse_command_submission_v1_json(input, TEST_JSON_LIMITS).is_err());
        }
        assert!(
            parse_command_action_v1_json(
                include_str!("../../../contracts/fixtures/commands/action.invalid.json"),
                TEST_JSON_LIMITS
            )
            .is_err()
        );
    }

    #[test]
    fn validates_canonical_command_digests_and_revision_bindings() {
        for input in [
            include_str!("../../../contracts/fixtures/commands/command.valid.json"),
            include_str!("../../../contracts/fixtures/commands/command.task-create.valid.json"),
            include_str!("../../../contracts/fixtures/commands/command.task-update.valid.json"),
            include_str!("../../../contracts/fixtures/commands/command.attempt-cancel.valid.json"),
            include_str!("../../../contracts/fixtures/commands/command.integer-float.valid.json"),
        ] {
            let command = parse_command_v1_json(input, TEST_JSON_LIMITS).unwrap();
            assert_eq!(
                compute_command_v1_digest_json(input, TEST_JSON_LIMITS).unwrap(),
                command.payload_digest()
            );
            let serialized = serde_json::to_string(&command).unwrap();
            assert_eq!(
                parse_command_v1_json(&serialized, TEST_JSON_LIMITS).unwrap(),
                command
            );
        }
        let valid_command = include_str!("../../../contracts/fixtures/commands/command.valid.json");
        let mut valid_value = parse_strict_json(valid_command, TEST_JSON_LIMITS).unwrap();
        let original_digest = valid_value["payload_digest"].as_str().unwrap().to_owned();
        valid_value["payload_digest"] = Value::String("f".repeat(64));
        let changed_digest = serde_json::to_string(&valid_value).unwrap();
        assert_eq!(
            compute_command_v1_digest_json(&changed_digest, TEST_JSON_LIMITS).unwrap(),
            original_digest
        );
        assert!(parse_command_v1_json(&changed_digest, TEST_JSON_LIMITS).is_err());
        assert!(
            parse_command_v1_json(
                include_str!("../../../contracts/fixtures/commands/command.invalid.json"),
                TEST_JSON_LIMITS
            )
            .is_err()
        );

        let wrong_revision = include_str!(
            "../../../contracts/fixtures/commands/command.bad-revision-target.invalid.json"
        );
        let value = parse_strict_json(wrong_revision, TEST_JSON_LIMITS).unwrap();
        let digest = digest_command_value(value.clone()).unwrap();
        assert_eq!(
            value["payload_digest"].as_str().unwrap(),
            digest,
            "the invalid fixture has a valid digest and isolates target binding"
        );
        assert!(
            parse_command_v1_json(wrong_revision, TEST_JSON_LIMITS)
                .unwrap_err()
                .contains("Revision target does not match")
        );
    }

    #[test]
    fn rejects_duplicate_properties_and_matches_jcs_vectors() {
        for input in [
            include_str!("../../../contracts/fixtures/commands/duplicate-key.raw.json"),
            include_str!("../../../contracts/fixtures/commands/jcs-duplicate-escaped-key.raw.json"),
        ] {
            assert!(
                canonicalize_json_text(input, TEST_JSON_LIMITS)
                    .unwrap_err()
                    .contains("duplicate_property")
            );
        }
        let vectors: Vec<Value> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/commands/jcs-vectors.json"
        ))
        .unwrap();
        for vector in vectors {
            let name = vector["name"].as_str().unwrap();
            let actual =
                canonicalize_json_text(vector["input_json"].as_str().unwrap(), TEST_JSON_LIMITS)
                    .unwrap();
            assert_eq!(actual, vector["canonical_json"].as_str().unwrap(), "{name}");
        }
        for input in [
            include_str!("../../../contracts/fixtures/commands/jcs-fractional-number.raw.json"),
            include_str!("../../../contracts/fixtures/commands/jcs-unsafe-integer.raw.json"),
            include_str!("../../../contracts/fixtures/commands/jcs-i64-min.raw.json"),
        ] {
            assert!(canonicalize_json_text(input, TEST_JSON_LIMITS).is_err());
        }

        let inputs: Vec<JsonInputVector> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/command-json-inputs.v1.json"
        ))
        .unwrap();
        for vector in inputs
            .iter()
            .filter(|vector| vector.expected == "negative_zero")
        {
            assert_eq!(
                parse_json_input(&vector.json, TEST_JSON_LIMITS)
                    .unwrap_err()
                    .kind(),
                "negative_zero",
                "{}",
                vector.id
            );
            assert_eq!(
                canonicalize_json_text(&vector.json, TEST_JSON_LIMITS).unwrap_err(),
                "negative_zero",
                "{}",
                vector.id
            );
        }
    }

    #[test]
    fn fingerprints_same_submission_and_rejects_changed_content_or_principal() {
        let replay: Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/commands/create-replay.json"
        ))
        .unwrap();
        let principal: AuthenticatedPrincipalV1 =
            serde_json::from_value(replay["principal"].clone()).unwrap();
        let submission = serde_json::to_string(&replay["submission"]).unwrap();
        let same_retry = serde_json::to_string(&replay["same_retry"]).unwrap();
        let changed_retry = serde_json::to_string(&replay["changed_retry"]).unwrap();
        let fingerprint =
            submission_fingerprint_v1_json(&submission, &principal, TEST_JSON_LIMITS).unwrap();
        assert_eq!(
            fingerprint,
            replay["expected_submission_fingerprint"].as_str().unwrap()
        );
        assert_eq!(
            submission_fingerprint_v1_json(&same_retry, &principal, TEST_JSON_LIMITS).unwrap(),
            fingerprint
        );
        let action = &replay["submission"]["action"];
        let reordered_submission = format!(
            "{{\"action\":{{\"goal\":{},\"title\":{},\"kind\":{}}},\"deadline_unix_ms\":{},\"command_id\":{},\"schema_version\":{}}}",
            serde_json::to_string(&action["goal"]).unwrap(),
            serde_json::to_string(&action["title"]).unwrap(),
            serde_json::to_string(&action["kind"]).unwrap(),
            replay["submission"]["deadline_unix_ms"],
            serde_json::to_string(&replay["submission"]["command_id"]).unwrap(),
            replay["submission"]["schema_version"],
        );
        assert_eq!(
            submission_fingerprint_v1_json(&reordered_submission, &principal, TEST_JSON_LIMITS)
                .unwrap(),
            fingerprint
        );
        assert_eq!(
            submission_fingerprint_v1_json(&changed_retry, &principal, TEST_JSON_LIMITS).unwrap(),
            replay["expected_changed_content_fingerprint"]
                .as_str()
                .unwrap()
        );
        let changed_principal: AuthenticatedPrincipalV1 =
            serde_json::from_value(replay["changed_principal"].clone()).unwrap();
        assert_eq!(
            submission_fingerprint_v1_json(&submission, &changed_principal, TEST_JSON_LIMITS)
                .unwrap(),
            replay["expected_changed_principal_fingerprint"]
                .as_str()
                .unwrap()
        );
        assert_ne!(
            fingerprint,
            replay["expected_changed_content_fingerprint"]
                .as_str()
                .unwrap()
        );
        assert_ne!(
            fingerprint,
            replay["expected_changed_principal_fingerprint"]
                .as_str()
                .unwrap()
        );
        let receipt = parse_coordinator_receipt_v1_json(
            &serde_json::to_string(&replay["original_receipt"]).unwrap(),
            TEST_JSON_LIMITS,
        )
        .unwrap();
        let canonical_command = parse_command_v1_json(
            include_str!("../../../contracts/fixtures/commands/command.task-create.valid.json"),
            TEST_JSON_LIMITS,
        )
        .unwrap();
        assert_eq!(canonical_command.submission_fingerprint(), fingerprint);
        assert_eq!(receipt.payload_digest, canonical_command.payload_digest());
        assert_eq!(
            receipt.command_id,
            replay["submission"]["command_id"].as_str().unwrap()
        );
        assert_eq!(receipt.submission_fingerprint, fingerprint);
        assert_ne!(
            receipt.submission_fingerprint,
            replay["expected_changed_content_fingerprint"]
                .as_str()
                .unwrap()
        );
        assert_ne!(
            receipt.submission_fingerprint,
            replay["expected_changed_principal_fingerprint"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            receipt.assigned_target.task_id,
            replay["expected_assigned_task_id"].as_str().unwrap()
        );
        assert!(
            parse_coordinator_receipt_v1_json(
                include_str!(
                    "../../../contracts/fixtures/commands/coordinator-receipt.invalid.json"
                ),
                TEST_JSON_LIMITS
            )
            .is_err()
        );
    }

    #[test]
    fn classifies_deadline_equal_to_or_before_now_as_expired() {
        let vectors: Vec<Value> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/commands/deadline-vectors.json"
        ))
        .unwrap();
        for vector in vectors {
            let expected = vector["expected_rejection"].as_str();
            let result = deadline_rejection_at(
                vector["deadline_unix_ms"].as_i64().unwrap(),
                vector["now_unix_ms"].as_i64().unwrap(),
            )
            .unwrap();
            assert_eq!(
                result.as_ref().map(|code| match code {
                    HostRejectionCodeV1::Expired => "expired",
                    _ => "unexpected",
                }),
                expected
            );
        }
        assert!(deadline_rejection_at(-1, 0).is_err());
    }

    #[test]
    fn validates_receipt_states_without_implying_attempt_state() {
        for input in [
            include_str!("../../../contracts/fixtures/commands/host-receipt.pending.valid.json"),
            include_str!("../../../contracts/fixtures/commands/host-receipt.accepted.valid.json"),
            include_str!("../../../contracts/fixtures/commands/host-receipt.valid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/host-receipt.stale-revision.valid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/host-receipt.permission-changed.valid.json"
            ),
            include_str!("../../../contracts/fixtures/commands/host-receipt.resolved.valid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/host-receipt.resolved-no-result.valid.json"
            ),
            include_str!("../../../contracts/fixtures/commands/host-receipt.unknown.valid.json"),
        ] {
            let receipt = parse_host_receipt_v1_json(input, TEST_JSON_LIMITS).unwrap();
            let serialized = serde_json::to_string(&receipt).unwrap();
            assert!(parse_host_receipt_v1_json(&serialized, TEST_JSON_LIMITS).is_ok());
        }
        assert!(
            parse_host_receipt_v1_json(
                include_str!("../../../contracts/fixtures/commands/host-receipt.invalid.json"),
                TEST_JSON_LIMITS
            )
            .is_err()
        );
        let mut host_receipt = parse_strict_json(
            include_str!("../../../contracts/fixtures/commands/host-receipt.accepted.valid.json"),
            TEST_JSON_LIMITS,
        )
        .unwrap();
        host_receipt["attempt_state"] = Value::String("running".into());
        assert!(
            parse_host_receipt_v1_json(
                &serde_json::to_string(&host_receipt).unwrap(),
                TEST_JSON_LIMITS
            )
            .is_err()
        );
        let mut coordinator_receipt = parse_strict_json(
            include_str!("../../../contracts/fixtures/commands/coordinator-receipt.valid.json"),
            TEST_JSON_LIMITS,
        )
        .unwrap();
        coordinator_receipt["attempt_state"] = Value::String("completed".into());
        assert!(
            parse_coordinator_receipt_v1_json(
                &serde_json::to_string(&coordinator_receipt).unwrap(),
                TEST_JSON_LIMITS,
            )
            .is_err()
        );
    }

    #[test]
    fn parses_typed_known_events_and_fail_closes_unknown_required_events() {
        match parse_command_event_v1_json(
            include_str!("../../../contracts/fixtures/commands/event.valid.json"),
            TEST_JSON_LIMITS,
        )
        .unwrap()
        {
            ParsedCommandEventV1::Known(KnownCommandEventV1::AttemptStateChanged(event)) => {
                assert_eq!(event.payload.to, AttemptStateV1::Running);
            }
            other => panic!("expected typed attempt event, got {other:?}"),
        }
        for input in [
            include_str!("../../../contracts/fixtures/commands/event.task-created.valid.json"),
            include_str!("../../../contracts/fixtures/commands/event.task-updated.valid.json"),
            include_str!("../../../contracts/fixtures/commands/event.integer-float.valid.json"),
        ] {
            assert!(matches!(
                parse_command_event_v1_json(input, TEST_JSON_LIMITS),
                Ok(ParsedCommandEventV1::Known(_))
            ));
        }
        for input in [
            include_str!(
                "../../../contracts/fixtures/commands/event.unknown-informational.valid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/event.constructor-informational.valid.json"
            ),
        ] {
            assert!(matches!(
                parse_command_event_v1_json(input, TEST_JSON_LIMITS),
                Ok(ParsedCommandEventV1::UnknownInformational {
                    refresh_snapshot_before_advance: true,
                    ..
                })
            ));
        }
        for input in [
            include_str!("../../../contracts/fixtures/commands/event.invalid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/event.unknown-no-cursor.invalid.json"
            ),
            include_str!(
                "../../../contracts/fixtures/commands/event.unknown-required.invalid.json"
            ),
            include_str!("../../../contracts/fixtures/commands/event.unsafe-integer.invalid.json"),
            include_str!(
                "../../../contracts/fixtures/commands/event.fractional-number.invalid.json"
            ),
        ] {
            assert!(parse_command_event_v1_json(input, TEST_JSON_LIMITS).is_err());
        }
    }
}
