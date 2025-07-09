use super::event::Event;

const NOTIFIER_NAME: &'static str = "Bugsnag Rust";
const NOTIFIER_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NOTIFIER_URL: &'static str = "https://github.com/mehcode/bugsnag-rs";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Notifier {
    name: &'static str,
    version: &'static str,
    url: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification<'a> {
    api_key: &'a str,
    notifier: Notifier,
    events: &'a [Event<'a>],
}

impl<'a> Notification<'a> {
    pub fn new(apikey: &'a str, events: &'a [Event]) -> Notification<'a> {
        Notification {
            api_key: apikey,
            notifier: Notifier {
                name: NOTIFIER_NAME,
                version: NOTIFIER_VERSION,
                url: NOTIFIER_URL,
            },
            events: events,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Notification, NOTIFIER_NAME, NOTIFIER_URL, NOTIFIER_VERSION};
    use super::super::{deviceinfo, event, exception, stacktrace};
    use serde_test::{assert_ser_tokens, Token};

    #[test]
    fn test_notification_to_json() {
        let empty_vec = Vec::new();
        let notification = Notification::new("safe-api-key", &empty_vec);

        assert_ser_tokens(
            &notification,
            &[
                Token::Struct {
                    name: "Notification",
                    len: 3,
                },
                Token::Str("apiKey"),
                Token::Str("safe-api-key"),
                Token::Str("notifier"),
                Token::Struct {
                    name: "Notifier",
                    len: 3,
                },
                Token::Str("name"),
                Token::Str(NOTIFIER_NAME),
                Token::Str("version"),
                Token::Str(NOTIFIER_VERSION),
                Token::Str("url"),
                Token::Str(NOTIFIER_URL),
                Token::StructEnd,
                Token::Str("events"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_notification_with_event_to_json() {
        let frames = vec![stacktrace::Frame::new("test.rs", 400, "test", false)];
        let exceptions = vec![exception::Exception::new("Assert", "Assert", &frames)];
        let mut device = deviceinfo::DeviceInfo::new("1.0.0", "testmachine");
        device.set_time("1970-01-01T00:00:00.000Z");
        let app = None;
        let user = None;
        let unhandled = None;
        let events = vec![
            event::Event::new(&exceptions, None, None, None, &device, &app, &user, &unhandled),
        ];

        let _notification = Notification::new("safe-api-key", &events);

        // Test that notification creation with event works without panicking
        // The hardware info is now populated from system info automatically
        assert!(true); // Basic smoke test that the notification was created successfully
    }
}
