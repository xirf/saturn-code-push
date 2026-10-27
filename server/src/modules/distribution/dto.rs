use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Check {
    pub app_id: String,
    pub channel: String,
    pub release_version: String,
    pub platform: String,
    pub arch: String,
    pub current_patch_number: Option<i64>,
    pub patch_number: Option<i64>,
    pub client_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_native_shorebird_identity_and_legacy_requests() {
        // Shape verified against shorebirdtech/updater d9da1d37116106830133fa332da27151478df37f.
        let fixture = include_str!("../../tests/fixtures/shorebird_patch_check.json");
        let mut payload: serde_json::Value = serde_json::from_str(fixture).unwrap();
        let check: Check = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(
            check.client_id.as_deref(),
            Some("11111111-2222-4333-8444-555555555555")
        );
        assert_eq!(check.current_patch_number, None);
        payload["current_patch_number"] = serde_json::json!(3);
        let check: Check = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(check.current_patch_number, Some(3));
        payload.as_object_mut().unwrap().remove("client_id");
        payload
            .as_object_mut()
            .unwrap()
            .remove("current_patch_number");
        payload["patch_number"] = serde_json::json!(2);
        let check: Check = serde_json::from_value(payload).unwrap();
        assert_eq!(check.client_id, None);
        assert_eq!(check.patch_number, Some(2));
    }
}
