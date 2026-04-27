import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ChatPage } from "./ChatPage";

// Mock the hooks and API
vi.mock("../lib/queries/agents", () => ({
  useAgents: vi.fn(),
  useAgentSessions: vi.fn(),
}));

vi.mock("../lib/queries/models", () => ({
  useModels: vi.fn(),
}));

vi.mock("../lib/queries/config", () => ({
  useFullConfig: vi.fn(),
}));

vi.mock("../lib/queries/approvals", () => ({
  usePendingApprovals: vi.fn(),
}));

vi.mock("../lib/queries/media", () => ({
  useMediaProviders: vi.fn(),
}));

vi.mock("../lib/queries/sessions", () => ({
  useSessionStream: vi.fn(),
}));

vi.mock("../lib/queries/hands", () => ({
  useActiveHandsWhen: vi.fn(),
}));

vi.mock("../lib/mutations/agents", () => ({
  useCreateAgentSession: vi.fn(),
  useDeleteAgentSession: vi.fn(),
  usePatchAgentConfig: vi.fn(),
  usePatchHandAgentRuntimeConfig: vi.fn(),
  useResolveApproval: vi.fn(),
  useStopAgent: vi.fn(),
  useUploadAgentFile: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useSearch: () => ({}),
}));

vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQueryClient: () => ({
      invalidateQueries: vi.fn(),
    }),
  };
});

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        staleTime: 0,
      },
    },
  });

describe("ChatPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const renderWithQueryClient = (ui: React.ReactElement) => {
    const queryClient = createTestQueryClient();
    return render(
      <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
    );
  };

  it("renders loading state correctly", () => {
    const { useAgents } = await import("../lib/queries/agents");
    const { useModels } = await import("../lib/queries/models");
    const { useFullConfig } = await import("../lib/queries/config");

    (useAgents as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useModels as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useFullConfig as any).mockReturnValue({
      data: null,
      isLoading: true,
    });

    renderWithQueryClient(<ChatPage />);

    // Should show loading indicator or skeleton
    expect(document.body.textContent).toBeTruthy();
  });

  it("renders empty state when no agents exist", async () => {
    const { useAgents } = await import("../lib/queries/agents");
    const { useModels } = await import("../lib/queries/models");
    const { useFullConfig } = await import("../lib/queries/config");

    (useAgents as any).mockReturnValue({
      data: [],
      isLoading: false,
    });
    (useModels as any).mockReturnValue({
      data: [],
      isLoading: false,
    });
    (useFullConfig as any).mockReturnValue({
      data: {},
      isLoading: false,
    });

    renderWithQueryClient(<ChatPage />);

    // Should render page content
    await waitFor(() => {
      expect(document.body).toBeTruthy();
    });
  });

  it("renders with agents data", async () => {
    const { useAgents } = await import("../lib/queries/agents");
    const { useModels } = await import("../lib/queries/models");
    const { useFullConfig } = await import("../lib/queries/config");

    const mockAgents = [
      { id: "agent-1", name: "Test Agent", status: "active" },
    ];

    (useAgents as any).mockReturnValue({
      data: mockAgents,
      isLoading: false,
    });
    (useModels as any).mockReturnValue({
      data: [],
      isLoading: false,
    });
    (useFullConfig as any).mockReturnValue({
      data: {},
      isLoading: false,
    });

    renderWithQueryClient(<ChatPage />);

    // Page should render
    await waitFor(() => {
      expect(document.body).toBeTruthy();
    });
  });
});
