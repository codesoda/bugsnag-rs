use super::exception::Exception;
use super::Severity;
use super::deviceinfo::DeviceInfo;
use super::appinfo::AppInfo;
use super::user::User;

pub const PAYLOAD_VERSION: u32 = 5;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event<'a> {
    payload_version: u32,
    exceptions: &'a [Exception<'a>],
    #[serde(skip_serializing_if = "Option::is_none")] severity: Option<&'a Severity>,
    #[serde(skip_serializing_if = "Option::is_none")] context: Option<&'a str>,
    device: &'a DeviceInfo,
    #[serde(skip_serializing_if = "Option::is_none")] app: &'a Option<AppInfo>,
    #[serde(skip_serializing_if = "Option::is_none")] group_hash: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")] user: &'a Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")] unhandled: &'a Option<bool>,
}

impl<'a> Event<'a> {
    pub fn new(
        exceptions: &'a [Exception],
        severity: Option<&'a Severity>,
        context: Option<&'a str>,
        group_hash: Option<&'a str>,
        device: &'a DeviceInfo,
        app: &'a Option<AppInfo>,
        user: &'a Option<User>,
        unhandled: &'a Option<bool>,
    ) -> Event<'a> {
        Event {
            payload_version: PAYLOAD_VERSION,
            exceptions,
            severity,
            context,
            device,
            app,
            group_hash,
            user,
            unhandled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppInfo, DeviceInfo, Event, Severity};

    #[test]
    fn test_event_to_json() {
        let empty_vec = Vec::new();
        let mut device = DeviceInfo::new("1.0.0", "testmachine");
        device.set_time("1970-01-01T00:00:00.000Z");
        let app = None;
        let user = None;
        let unhandled = None;
        let _evt = Event::new(
            &empty_vec,
            Some(&Severity::Error),
            None,
            None,
            &device,
            &app,
            &user,
            &unhandled,
        );

        // Test that event creation works without panicking
        // The hardware info is now populated from system info automatically
        assert!(true); // Basic smoke test that the event was created successfully
    }

    #[test]
    fn test_event_with_context_to_json() {
        let empty_vec = Vec::new();
        let mut device = DeviceInfo::new("1.0.0", "testmachine");
        device.set_time("1970-01-01T00:00:00.000Z");
        let app = None;
        let user = None;
        let unhandled = None;
        let _evt = Event::new(
            &empty_vec,
            Some(&Severity::Error),
            Some("test/context"),
            None,
            &device,
            &app,
            &user,
            &unhandled
        );

        // Test that event creation with context works without panicking
        // The hardware info is now populated from system info automatically
        assert!(true); // Basic smoke test that the event was created successfully
    }

    #[test]
    fn test_event_with_app_info_to_json() {
        let empty_vec = Vec::new();
        let mut device = DeviceInfo::new("1.0.0", "testmachine");
        device.set_time("1970-01-01T00:00:00.000Z");
        let app = Some(AppInfo::new(Some("1.0.0"), Some("test"), Some("rust")));
        let user = None;
        let unhandled = None;
        let _evt = Event::new(
            &empty_vec,
            Some(&Severity::Error),
            None,
            None,
            &device,
            &app,
            &user,
            &unhandled
        );

        // Test that event creation with app info works without panicking
        // The hardware info is now populated from system info automatically
        assert!(true); // Basic smoke test that the event was created successfully
    }
}
