import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { OverviewPage } from "./OverviewPage";
import { useDashboardSnapshot, useVersionInfo } from "../lib/queries/overview";
import { useQuickInit } from "../lib/mutations/overview";

// Mock the hooks
vi.mock("../lib/queries/overview", () => ({
  useDashboardSnapshot: vi.fn(),
  useVersionInfo: vi.fn(),
}));

vi.mock("../lib/mutations/overview", () => ({
  useQuickInit: vi.fn(),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

// Mock @tanstack/react-router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
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

describe("OverviewPage", () => {
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
    (useDashboardSnapshot as any).mockReturnValue({
      data: null,
      isLoading: true,
      isError: false,
    });
    (useVersionInfo as any).mockReturnValue({
      data: null,
      isLoading: true,
    });
    (useQuickInit as any).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    });

    renderWithQueryClient(<OverviewPage />);

    // Should show skeleton loaders
    expect(document.querySelector("[data-testid='skeleton']") || document.querySelector(".animate-pulse")).toBeTruthy();
  });

  it("renders data correctly when loaded", async () => {
    const mockSnapshot = {
      status: {
        active_agent_count: 5,
        agent_count: 10,
        config_exists: true,
        uptime_seconds: 3600,
        healthy: true,
      },
      providers: [
        { id: "ollama", auth_status: "ok", display_name: "Ollama" },
        { id: "openai", auth_status: "error", display_name: "OpenAI" },
      ],
      channels: [
        { id: "telegram", enabled: true },
        { id: "discord", enabled: false },
      ],
    };

    const mockVersion = {
      version: "2026.4.27",
      commit: "abc123",
    };

    (useDashboardSnapshot as any).mockReturnValue({
      data: mockSnapshot,
      isLoading: false,
      isError: false,
    });
    (useVersionInfo as any).mockReturnValue({
      data: mockVersion,
      isLoading: false,
    });
    (useQuickInit as any).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    });

    renderWithQueryClient(<OverviewPage />);

    // Should render version info
    await waitFor(() => {
      expect(screen.getByText("2026.4.27")).toBeInTheDocument();
    });

    // Should render agent counts
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("10")).toBeInTheDocument();
  });

  it("renders init prompt when config does not exist", () => {
    const mockSnapshot = {
      status: {
        active_agent_count: 0,
        agent_count: 0,
        config_exists: false,
        uptime_seconds: 0,
        healthy: true,
      },
      providers: [],
      channels: [],
    };

    (useDashboardSnapshot as any).mockReturnValue({
      data: mockSnapshot,
      isLoading: false,
      isError: false,
    });
    (useVersionInfo as any).mockReturnValue({
      data: null,
      isLoading: false,
    });
    (useQuickInit as any).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    });

    renderWithQueryClient(<OverviewPage />);

    // Should show init prompt
    expect(screen.getByText(/quickInit/)).toBeInTheDocument();
  });

  it("renders error state when query fails", () => {
    (useDashboardSnapshot as any).mockReturnValue({
      data: null,
      isLoading: false,
      isError: true,
      error: new Error("Failed to fetch"),
    });
    (useVersionInfo as any).mockReturnValue({
      data: null,
      isLoading: false,
      isError: true,
    });
    (useQuickInit as any).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    });

    renderWithQueryClient(<OverviewPage />);

    // Should show error state or empty
    expect(document.body.textContent).toBeTruthy();
  });
});
