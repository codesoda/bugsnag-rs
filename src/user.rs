#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")] name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] email: Option<String>
}

impl User {
    pub fn new(id: &str, name: Option<&str>, email: Option<&str>) -> User {
        User {
            id: id.to_owned(),
            name: name.map_or_else(|| None, |v| Some(v.to_owned())),
            email: email.map_or_else(|| None, |v| Some(v.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    use serde_test::{assert_ser_tokens, Token};

    #[test]
    fn test_user_to_json() {
        let user = User::new("19", Some("Chris Raethke"), Some("chris@example.com"));

        assert_ser_tokens(
            &user,
            &[
                Token::Struct {
                    name: "User",
                    len: 3,
                },
                Token::Str("id"),
                Token::Str("19"),
                Token::Str("name"),
                Token::Some,
                Token::Str("Chris Raethke"),
                Token::Str("email"),
                Token::Some,
                Token::Str("chris@example.com"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_user_with_id_to_json() {
        let user = User::new("19", None, None);

        assert_ser_tokens(
            &user,
            &[
                Token::Struct {
                    name: "User",
                    len: 1,
                },
                Token::Str("id"),
                Token::Str("19"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_user_with_name_to_json() {
        let user = User::new("19", Some("Chris Raethke"), None);

        assert_ser_tokens(
            &user,
            &[
                Token::Struct {
                    name: "User",
                    len: 2,
                },
                Token::Str("id"),
                Token::Str("19"),
                Token::Str("name"),
                Token::Some,
                Token::Str("Chris Raethke"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_user_with_email_to_json() {
        let user = User::new("19", None, Some("chris@example.com"));

        assert_ser_tokens(
            &user,
            &[
                Token::Struct {
                    name: "User",
                    len: 2,
                },
                Token::Str("id"),
                Token::Str("19"),
                Token::Str("email"),
                Token::Some,
                Token::Str("chris@example.com"),
                Token::StructEnd,
            ],
        );
    }
}
