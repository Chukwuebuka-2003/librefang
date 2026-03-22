//! TestAppState — 构建适用于 axum 路由测试的 `AppState` 和 `Router`。
//!
//! 封装了 `MockKernelBuilder` 的输出，提供快速构建测试路由器的方法。

use crate::mock_kernel::MockKernelBuilder;
use axum::Router;
use librefang_api::routes::AppState;
use librefang_kernel::LibreFangKernel;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

/// 测试用 AppState 构建器。
///
/// # 示例
///
/// ```no_run
/// use librefang_testing::TestAppState;
///
/// let test = TestAppState::new();
/// let router = test.router();
/// // 现在可以使用 tower::ServiceExt 发送测试请求
/// ```
pub struct TestAppState {
    /// 共享的 AppState（和生产环境相同的类型）。
    pub state: Arc<AppState>,
    /// 临时目录 — 必须持有引用，否则目录会被删除。
    _tmp: TempDir,
}

impl TestAppState {
    /// 使用默认 mock kernel 创建 TestAppState。
    pub fn new() -> Self {
        Self::with_builder(MockKernelBuilder::new())
    }

    /// 使用自定义 MockKernelBuilder 创建 TestAppState。
    pub fn with_builder(builder: MockKernelBuilder) -> Self {
        let (kernel, tmp) = builder.build();
        let state = Self::build_state(kernel, &tmp);
        Self { state, _tmp: tmp }
    }

    /// 从已有的 kernel 构建（调用方负责持有 TempDir）。
    pub fn from_kernel(kernel: LibreFangKernel, tmp: TempDir) -> Self {
        let state = Self::build_state(kernel, &tmp);
        Self { state, _tmp: tmp }
    }

    /// 构建一个包含所有 API 路由的 axum Router（适合测试）。
    ///
    /// 返回的 Router 已嵌套在 `/api` 路径下，和生产环境一致。
    pub fn router(&self) -> Router {
        // 构建与 server.rs 中 api_v1_routes() 相同的路由树
        // 这里只包含常用的测试端点，避免引入太多依赖
        let api = Router::new()
            .route("/health", axum::routing::get(librefang_api::routes::health))
            .route(
                "/agents",
                axum::routing::get(librefang_api::routes::list_agents),
            )
            .route(
                "/agents/{id}",
                axum::routing::get(librefang_api::routes::get_agent),
            )
            .route("/status", axum::routing::get(librefang_api::routes::status))
            .route(
                "/version",
                axum::routing::get(librefang_api::routes::version),
            )
            .route(
                "/profiles",
                axum::routing::get(librefang_api::routes::list_profiles),
            );

        Router::new()
            .nest("/api", api)
            .with_state(self.state.clone())
    }

    /// 获取 AppState 的 Arc 引用。
    pub fn app_state(&self) -> Arc<AppState> {
        self.state.clone()
    }

    /// 内部：从 kernel 构建 AppState。
    fn build_state(kernel: LibreFangKernel, tmp: &TempDir) -> Arc<AppState> {
        let kernel = Arc::new(kernel);
        let channels_config = kernel.config.channels.clone();

        Arc::new(AppState {
            kernel,
            started_at: Instant::now(),
            peer_registry: None,
            bridge_manager: tokio::sync::Mutex::new(None),
            channels_config: tokio::sync::RwLock::new(channels_config),
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
            clawhub_cache: dashmap::DashMap::new(),
            provider_probe_cache: librefang_runtime::provider_health::ProbeCache::new(),
            webhook_store: librefang_api::webhook_store::WebhookStore::load(
                tmp.path().join("test_webhooks.json"),
            ),
        })
    }
}

impl Default for TestAppState {
    fn default() -> Self {
        Self::new()
    }
}
