//! 示例测试 — 演示如何使用测试基础设施。

use crate::{assert_json_error, assert_json_ok, test_request, MockKernelBuilder, TestAppState};
use axum::http::{Method, StatusCode};
use tower::ServiceExt;

/// 测试 GET /api/health 端点返回 200 且包含 status 字段。
#[tokio::test]
async fn test_health_endpoint() {
    let app = TestAppState::new();
    let router = app.router();

    let req = test_request(Method::GET, "/api/health", None);
    let resp = router.oneshot(req).await.expect("请求失败");
    let json = assert_json_ok(resp).await;

    // health 端点应该返回 status 字段
    assert!(
        json.get("status").is_some(),
        "健康检查应包含 status 字段，实际返回: {json}"
    );
    let status = json["status"].as_str().unwrap();
    assert!(
        status == "ok" || status == "degraded",
        "status 应为 ok 或 degraded，实际: {status}"
    );
}

/// 测试 GET /api/agents 端点 — 返回 items 数组和 total 字段。
#[tokio::test]
async fn test_list_agents() {
    let app = TestAppState::new();
    let router = app.router();

    let req = test_request(Method::GET, "/api/agents", None);
    let resp = router.oneshot(req).await.expect("请求失败");
    let json = assert_json_ok(resp).await;

    // list_agents 返回 {"items": [...], "total": N, "offset": 0}
    assert!(
        json.get("items").is_some(),
        "list_agents 应返回 items 字段，实际: {json}"
    );
    assert!(
        json["items"].is_array(),
        "items 应为数组，实际: {}",
        json["items"]
    );
    assert!(
        json.get("total").is_some(),
        "list_agents 应返回 total 字段，实际: {json}"
    );
    // kernel 启动时会自动创建默认 agent，所以 total >= 0
    assert!(json["total"].as_u64().unwrap() >= 0, "total 应为非负整数");
}

/// 测试 GET /api/agents/{id} — 使用无效 ID 应返回 400。
#[tokio::test]
async fn test_get_agent_invalid_id() {
    let app = TestAppState::new();
    let router = app.router();

    let req = test_request(Method::GET, "/api/agents/not-a-valid-uuid", None);
    let resp = router.oneshot(req).await.expect("请求失败");
    let json = assert_json_error(resp, StatusCode::BAD_REQUEST).await;

    assert!(
        json.get("error").is_some(),
        "错误响应应包含 error 字段，实际: {json}"
    );
}

/// 测试 GET /api/agents/{id} — 使用有效但不存在的 UUID 应返回 404。
#[tokio::test]
async fn test_get_agent_not_found() {
    let app = TestAppState::new();
    let router = app.router();

    // 使用一个有效的 UUID 但不存在于 registry 中
    let fake_id = uuid::Uuid::new_v4();
    let path = format!("/api/agents/{fake_id}");
    let req = test_request(Method::GET, &path, None);
    let resp = router.oneshot(req).await.expect("请求失败");
    let json = assert_json_error(resp, StatusCode::NOT_FOUND).await;

    assert!(
        json.get("error").is_some(),
        "404 响应应包含 error 字段，实际: {json}"
    );
}

/// 测试 MockLlmDriver 的调用记录功能。
#[tokio::test]
async fn test_mock_llm_driver_recording() {
    use crate::MockLlmDriver;
    use librefang_runtime::llm_driver::{CompletionRequest, LlmDriver};

    let driver = MockLlmDriver::new(vec!["回复1".into(), "回复2".into()]);

    let request = CompletionRequest {
        model: "test-model".into(),
        messages: vec![],
        tools: vec![],
        max_tokens: 100,
        temperature: 0.0,
        system: Some("test system prompt".into()),
        thinking: None,
        prompt_caching: false,
    };

    // 第一次调用
    let resp1 = driver.complete(request.clone()).await.unwrap();
    assert_eq!(resp1.text(), "回复1");

    // 第二次调用
    let resp2 = driver.complete(request).await.unwrap();
    assert_eq!(resp2.text(), "回复2");

    // 验证调用记录
    assert_eq!(driver.call_count(), 2);
    let calls = driver.recorded_calls();
    assert_eq!(calls[0].model, "test-model");
    assert_eq!(calls[0].system, Some("test system prompt".into()));
}

/// 测试使用自定义 config 构建 kernel。
#[tokio::test]
async fn test_custom_config_kernel() {
    let app = TestAppState::with_builder(MockKernelBuilder::new().with_config(|cfg| {
        cfg.language = "zh".into();
    }));

    // 验证自定义配置生效
    assert_eq!(app.state.kernel.config.language, "zh");
}

/// 测试 GET /api/version 端点。
#[tokio::test]
async fn test_version_endpoint() {
    let app = TestAppState::new();
    let router = app.router();

    let req = test_request(Method::GET, "/api/version", None);
    let resp = router.oneshot(req).await.expect("请求失败");
    let json = assert_json_ok(resp).await;

    assert!(
        json.get("version").is_some(),
        "version 端点应包含 version 字段，实际: {json}"
    );
}
