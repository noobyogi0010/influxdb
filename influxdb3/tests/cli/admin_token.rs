use std::time::Duration;

use crate::server::{ConfigProvider, TestServer, parse_token};
use observability_deps::tracing::info;
use serde_json::Value;
use test_helpers::{assert_contains, assert_not_contains};

#[test_log::test(tokio::test)]
async fn test_create_admin_token() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    println!("{result:?}");
    assert_contains!(&result, "New token created successfully!");
}

#[test_log::test(tokio::test)]
async fn test_create_admin_token_json_format() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &[
        "--tls-ca",
        "../testing-certs/rootCA.pem",
        "--format",
        "json",
    ];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();

    let value: Value =
        serde_json::from_str(&result).expect("token creation response should be in json format");
    let token = value
        .get("token")
        .expect("token to be present")
        .as_str()
        .expect("token to be a str");
    // check if the token generated works by using it to regenerate
    let result = server
        .run_with_confirmation(
            vec!["create", "token", "--admin"],
            &[
                "--regenerate",
                "--tls-ca",
                "../testing-certs/rootCA.pem",
                "--token",
                token,
            ],
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
}

#[test_log::test(tokio::test)]
async fn test_create_admin_token_allowed_once() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(&result, "New token created successfully!");

    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(
        &result,
        "Failed to create token, error: ApiError { code: 409, message: \"token name already exists, _admin\" }"
    );
}

#[test_log::test(tokio::test)]
async fn test_regenerate_admin_token() {
    // when created with_auth, TestServer spins up server and generates admin token.
    let mut server = TestServer::configure().with_auth().spawn().await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    // already has admin token, so it cannot be created again
    assert_contains!(
        &result,
        "Failed to create token, error: ApiError { code: 409, message: \"token name already exists, _admin\" }"
    );

    // regenerating token is allowed
    let result = server
        .run_with_confirmation(
            vec!["create", "token", "--admin"],
            &["--regenerate", "--tls-ca", "../testing-certs/rootCA.pem"],
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let old_token = server.token().expect("admin token to be present");
    let new_token = parse_token(result);
    assert!(old_token != &new_token);

    // old token cannot access
    let res = server
        .create_database("sample_db")
        .run()
        .err()
        .unwrap()
        .to_string();
    assert_contains!(&res, "401 Unauthorized");

    // new token should allow
    server.set_token(Some(new_token));
    let res = server.create_database("sample_db").run().unwrap();
    assert_contains!(&res, "Database \"sample_db\" created successfully");
}

#[test_log::test(tokio::test)]
async fn test_delete_token() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &[];
    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--tls-ca",
                "../testing-certs/rootCA.pem",
            ],
            args,
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let token = parse_token(result);

    let result = server
        .run(
            vec!["delete", "token"],
            &[
                "--token-name",
                "_admin",
                "--token",
                &token,
                "--tls-ca",
                "../testing-certs/rootCA.pem",
            ],
        )
        .unwrap();
    assert_contains!(
        result,
        "The operator token \"_admin\" is required and cannot be deleted. To regenerate an operator token, use: influxdb3 create token --admin --regenerate --token [TOKEN]"
    );

    // you should be able to create the token again
    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--tls-ca",
                "../testing-certs/rootCA.pem",
            ],
            args,
        )
        .unwrap();
    assert_contains!(
        &result,
        "Failed to create token, error: ApiError { code: 409, message: \"token name already exists, _admin\" }"
    );
}

#[test_log::test(tokio::test)]
async fn test_create_admin_token_endpoint_disabled() {
    let server = TestServer::configure().spawn().await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let expected = "code: 405, message: \"endpoint disabled, started without auth\"";

    let run_args = vec!["create", "token", "--admin"];
    let result = server.run(run_args, args).unwrap();
    assert_contains!(&result, expected);

    let regen_args = vec!["create", "token", "--admin", "--regenerate"];
    let result = server.run_with_confirmation(regen_args, args).unwrap();
    assert_contains!(&result, expected);
}

#[test_log::test(tokio::test)]
async fn test_create_named_admin_token() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let operator_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
}

#[test_log::test(tokio::test)]
async fn test_create_named_admin_token_only_once() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let operator_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap();
    assert_contains!(
        &result,
        "Failed to create token, error: ApiError { code: 409, message: \"token name already exists, foo_admin\" }"
    );
}

#[test_log::test(tokio::test)]
async fn test_named_admin_token_works() {
    let mut server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    // create operator token manually so that server doesn't set it implicitly
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    let operator_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap();

    let foo_admin_token = parse_token(result);
    server.set_token(Some(foo_admin_token));
    let res = server.create_database("sample_db").run().unwrap();
    assert_contains!(&res, "Database \"sample_db\" created successfully");
}

#[test_log::test(tokio::test)]
async fn test_named_admin_token_cannot_be_regenerated() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    // create operator token manually so that server doesn't set it implicitly
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    let operator_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap();
    let foo_admin_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--regenerate",
                "--token",
                &operator_token,
            ],
            args,
        )
        .unwrap_err()
        .to_string();
    assert_contains!(
        result,
        "--regenerate cannot be used with --name, --regenerate only applies for operator token"
    );

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--regenerate",
                "--token",
                &foo_admin_token,
            ],
            args,
        )
        .unwrap_err()
        .to_string();
    assert_contains!(
        result,
        "--regenerate cannot be used with --name, --regenerate only applies for operator token"
    );
}

#[test_log::test(tokio::test)]
async fn test_create_named_admin_token_endpoint_disabled() {
    let server = TestServer::configure().spawn().await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let expected = "code: 405, message: \"endpoint disabled, started without auth\"";

    let named_admin_args = vec!["create", "token", "--admin", "--name", "foo_admin"];
    let result = server
        .run_with_confirmation(named_admin_args, args)
        .unwrap();
    assert_contains!(&result, expected);
}

#[test_log::test(tokio::test)]
async fn test_delete_named_admin_token() {
    let server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let operator_token = parse_token(result);

    struct TestCase<'a> {
        delete_with: &'a str,
    }

    for case in [
        TestCase {
            delete_with: "operator",
        },
        TestCase {
            // yes use foo_admin_token to delete foo_admin_token :)
            delete_with: "admin",
        },
    ] {
        let result = server
            .run(
                vec![
                    "create",
                    "token",
                    "--admin",
                    "--name",
                    "foo_admin",
                    "--token",
                    &operator_token,
                ],
                args,
            )
            .unwrap();
        let foo_admin_token = parse_token(result);

        let result = server
            .run(vec!["show", "tokens", "--token", &operator_token], args)
            .unwrap();
        assert_contains!(result, "foo_admin");

        let delete_with_token = if case.delete_with == "operator" {
            &operator_token
        } else {
            &foo_admin_token
        };

        let result = server
            .run_with_confirmation(
                vec!["delete", "token"],
                &[
                    "--token-name",
                    "foo_admin",
                    "--token",
                    delete_with_token,
                    "--tls-ca",
                    "../testing-certs/rootCA.pem",
                ],
            )
            .unwrap();
        info!(result, "test: deleted token using token name");

        let result = server
            .run(vec!["show", "tokens", "--token", &operator_token], args)
            .unwrap();
        assert_not_contains!(result, "foo_admin");
    }
}

#[test_log::test(tokio::test)]
async fn test_check_named_admin_token_expiry_works() {
    let mut server = TestServer::configure()
        .with_auth()
        .with_no_admin_token()
        .spawn()
        .await;
    let args = &["--tls-ca", "../testing-certs/rootCA.pem"];
    let result = server
        .run(vec!["create", "token", "--admin"], args)
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let operator_token = parse_token(result);

    let result = server
        .run(
            vec![
                "create",
                "token",
                "--admin",
                "--name",
                "foo_admin",
                "--token",
                &operator_token,
                "--expiry",
                "2s",
            ],
            args,
        )
        .unwrap();
    assert_contains!(&result, "New token created successfully!");
    let foo_admin_token = parse_token(result);

    // from this point on server will use operator token for all ops
    server.set_token(Some(foo_admin_token.clone()));
    // make sure token works
    let res = server.create_database("sample_db").run().unwrap();
    assert_contains!(&res, "Database \"sample_db\" created successfully");

    // Not sure if we can pass in an external clock to be consumed by process
    // it'd be good to remove this 2s delay
    tokio::time::sleep(Duration::from_secs(2)).await;

    // the token shouldn't be useful
    server.set_token(Some(foo_admin_token));
    let res = server
        .create_database("sample_2_db")
        .run()
        .unwrap_err()
        .to_string();
    assert_contains!(&res, "[401 Unauthorized]");
}
