use crate::auth::UserId;

pub(super) fn scoped_cv_key(user_id: &UserId, service: &str) -> String {
    format!("{}:{}", user_id.0, service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_cv_key_uses_user_id_colon_service_format() {
        assert_eq!(scoped_cv_key(&UserId(42), "binance"), "42:binance");
    }
}
