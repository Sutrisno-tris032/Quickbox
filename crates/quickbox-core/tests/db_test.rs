use quickbox_core::{auth, db};

#[tokio::test]
async fn test_db_inbox_and_password_lifecycle() {
    let pool = db::init_db("sqlite::memory:").await.unwrap();
    let hashed = auth::hash_password("secret123").unwrap();
    assert!(auth::verify_password(&hashed, "secret123").unwrap());
    assert!(!auth::verify_password(&hashed, "wrong").unwrap());

    let inbox = db::get_or_create_inbox(&pool, "test-user").await.unwrap();
    assert_eq!(inbox.name, "test-user");
    assert!(!inbox.is_locked);
}

