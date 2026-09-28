use serde::{Deserialize, Serialize};
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
