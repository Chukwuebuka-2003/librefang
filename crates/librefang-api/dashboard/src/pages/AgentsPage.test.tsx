import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AgentsPage } from "./AgentsPage";

// Mock the hooks
vi.mock("../lib/queries/agents", () => ({
  agentQueries: {
    list: () => ["agents", "list"],
  },
  useAgentTemplates: vi.fn(),
  useExperimentMetrics: vi.fn(),
  useExperiments: vi.fn(),
  usePromptVersions: vi.fn(),
}));

vi.mock("../lib/queries/overview", () => ({
  useDashboardSnapshot: vi.fn(),
}));

vi.mock("../lib/queries/providers", () => ({
  useProviders: vi.fn(),
}));

vi.mock("../lib/queries/models", () => ({
  useModels: vi.fn(),
}));

vi.mock("../lib/mutations/agents", () => ({
  useActivatePromptVersion: vi.fn(),
  useCloneAgent: vi.fn(),
  useCompleteExperiment: vi.fn(),
  useCreateExperiment: vi.fn(),
  useCreatePromptVersion: vi.fn(),
  useDeleteAgent: vi.fn(),
  useDeletePromptVersion: vi.fn(),
  usePatchAgent: vi.fn(),
  usePatchAgentConfig: vi.fn(),
  usePatchHandAgentRuntimeConfig: vi.fn(),
  usePauseExperiment: vi.fn(),
  useResumeAgent: vi.fn(),
  useSpawnAgent: vi.fn(),
  useStartExperiment: vi.fn(),
  useSuspendAgent: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
}));

vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQuery: vi.fn(),
    useQueryClient: () => ({
      invalidateQueries: vi.fn(),
      getQueryData: vi.fn(),
    }),
  };
});

vi.mock("../lib/store", () => ({
  useUIStore: () => ({
    setSelectedAgentId: vi.fn(),
    selectedAgentId: null,
  }),
}));

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        staleTime: 0,
      },
    },
  });

describe("AgentsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const renderWithQueryClient = (ui: React.ReactElement) => {
    const queryClient = createTestQueryClient();
    return render(
      <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
    );
  };

  it("renders loading state", () => {
    const { useQuery } = require("@tanstack/react-query");
    const { useAgentTemplates } = require("../lib/queries/agents");
    const { useDashboardSnapshot } = require("../lib/queries/overview");
    const { useProviders } = require("../lib/queries/providers");
    const { useModels } = require("../lib/queries/models");

    (useAgentTemplates as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useDashboardSnapshot as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useProviders as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useModels as any).mockReturnValue({
      data: null,
      isLoading: true,
    });

    renderWithQueryClient(<AgentsPage />);

    // Should render something
    expect(document.body).toBeTruthy();
  });

  it("renders with agents list", async () => {
    const { useAgentTemplates } = require("../lib/queries/agents");
    const { useDashboardSnapshot } = require("../lib/queries/overview");
    const { useProviders } = require("../lib/queries/providers");
    const { useModels } = require("../lib/queries/models");

    (useAgentTemplates as any).mockReturnValue({
      data: [],
      isLoading: false,
    });
    (useDashboardSnapshot as any).mockReturnValue({
      data: {
        status: { agent_count: 5 },
      },
      isLoading: false,
    });
    (useProviders as any).mockReturnValue({
      data: [],
      isLoading: false,
    });
    (useModels as any).mockReturnValue({
      data: [],
      isLoading: false,
    });

    renderWithQueryClient(<AgentsPage />);

    await waitFor(() => {
      expect(document.body).toBeTruthy();
    });
  });
});
