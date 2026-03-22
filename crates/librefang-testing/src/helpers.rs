//! 测试辅助函数 — HTTP 请求构建和响应断言。

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};

/// 构建一个测试用的 HTTP 请求。
///
/// # 参数
/// - `method` — HTTP 方法（GET、POST 等）
/// - `path` — 请求路径（例如 "/api/health"）
/// - `body` — 请求体（None 表示空 body）
///
/// # 示例
///
/// ```rust
/// use librefang_testing::test_request;
/// use axum::http::Method;
///
/// let req = test_request(Method::GET, "/api/health", None);
/// let req_with_body = test_request(
///     Method::POST,
///     "/api/agents",
///     Some(r#"{"name": "test"}"#),
/// );
/// ```
pub fn test_request(method: Method, path: &str, body: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(path);

    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }

    let body = match body {
        Some(b) => Body::from(b.to_string()),
        None => Body::empty(),
    };

    builder.body(body).expect("构建测试请求失败")
}

/// 断言响应状态码为 200 且 body 是有效的 JSON。
///
/// 返回解析后的 `serde_json::Value`。
///
/// # Panics
///
/// 如果状态码不是 200 或 body 不是有效 JSON，则 panic。
pub async fn assert_json_ok(response: axum::http::Response<Body>) -> serde_json::Value {
    let status = response.status();
    let body = read_body(response).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "期望状态码 200，实际为 {status}。响应体: {body}"
    );

    serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!("响应体不是有效的 JSON: {e}。原始内容: {body}");
    })
}

/// 断言响应状态码为指定的错误码且 body 是有效的 JSON。
///
/// 返回解析后的 `serde_json::Value`。
///
/// # Panics
///
/// 如果状态码不匹配或 body 不是有效 JSON，则 panic。
pub async fn assert_json_error(
    response: axum::http::Response<Body>,
    expected_status: StatusCode,
) -> serde_json::Value {
    let status = response.status();
    let body = read_body(response).await;

    assert_eq!(
        status, expected_status,
        "期望状态码 {expected_status}，实际为 {status}。响应体: {body}"
    );

    serde_json::from_str(&body).unwrap_or_else(|e| {
        panic!("响应体不是有效的 JSON: {e}。原始内容: {body}");
    })
}

/// 读取 response body 为字符串。
async fn read_body(response: axum::http::Response<Body>) -> String {
    use http_body_util::BodyExt;
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("读取响应体失败")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("响应体不是有效的 UTF-8")
}
