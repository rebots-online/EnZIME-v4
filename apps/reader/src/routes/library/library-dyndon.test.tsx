import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { libraryService, type CatalogEntry, type CatalogPage, type InstallPlan, type DownloadJob } from "../../services/libraryService";
import { readerService } from "../../services/readerService";
import { useLibraryStore } from "../../stores/libraryStore";
import { budgetBytesFor, DownloadPlanDialog } from "./DownloadPlanDialog";
import { CatalogBrowser, catalogLanguage } from "./CatalogBrowser";
import { InstallPlanSummary } from "./InstallPlanSummary";
import { DownloadQueue } from "./DownloadQueue";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const entry: CatalogEntry = { catalogEntryId: "wiki", title: "Wikipedia", description: "Knowledge", languageBcp47: "eng", category: "encyclopedia", publisher: "Kiwix", artifactUrl: "https://download.kiwix.org/wiki.zim", artifactSizeBytes: 100 * 1024 ** 3, artifactSha256: null };
const root = { storageRootId: "root", displayName: "Local storage", writable: true };
const plan: InstallPlan = { installPlanId: "plan", catalogEntry: entry, destination: root, transferMode: "ranged", downloadBytes: 500 * 1024 ** 2, temporaryBytes: 32 * 1024 ** 2, finalBytes: 1024 ** 3, freeBytesBefore: 10 * 1024 ** 3, freeBytesAfter: 9 * 1024 ** 3, verification: { expectedSizeBytes: entry.artifactSizeBytes, expectedSha256: null, signatureRequired: false, zimStructureCheck: true } };
const job: DownloadJob = { downloadJobId: "job", state: "completed", bytesVerified: 0, bytesRetained: 500 * 1024 ** 2, expectedSizeBytes: entry.artifactSizeBytes, transferScope: "partial", errorDetail: null };
const initial = useLibraryStore.getState();
const ready = <T,>(data: T) => ({ status: "ready" as const, data, error: null, lastUpdated: null });
beforeEach(() => { vi.clearAllMocks(); useLibraryStore.setState(initial, true); vi.mocked(invoke).mockResolvedValue({ items: [] }); });
afterEach(() => cleanup());

function seedDialog() { useLibraryStore.setState({ online: true, activeCatalogEntryId: "wiki", selectedStorageRootId: "root", catalogEntries: ready([entry]), storageRoots: ready([root]) }); }

describe("DynDon wire contracts", () => {
  it("converts MiB/GiB to safe exact bytes and sends installed pack identity", async () => {
    expect(budgetBytesFor("512", "MiB")).toBe(536870912); expect(budgetBytesFor("1.5", "GiB")).toBe(1610612736);
    for (const value of ["", "0", "-1", "NaN", "Infinity", "9007199254740992", "0.0000001"]) expect(budgetBytesFor(value, "MiB")).toBeUndefined();
    await libraryService.installPlan({ catalogEntryId: "wiki", destinationStorageRootId: "root", budgetBytes: 536870912 });
    expect(invoke).toHaveBeenLastCalledWith("install_plan", { catalogEntryId: "wiki", destinationStorageRootId: "root", budgetBytes: 536870912 });
    await readerService.readerOpen("installed-pack"); expect(invoke).toHaveBeenLastCalledWith("reader_open", { packId: "installed-pack" });
    await libraryService.installCancel({ installPlanId: "plan" }); expect(invoke).toHaveBeenLastCalledWith("install_cancel", { installPlanId: "plan" });
  });
  it("preserves an unknown upstream digest without claiming full verification", () => {
    render(<InstallPlanSummary plan={plan} />);
    expect(screen.getByText("Not provided by source")).toBeTruthy(); expect(screen.getByText(/full archive SHA-256 is not verified/)).toBeTruthy();
  });
  it("renders retained partial bytes without dividing by the full archive size", () => {
    useLibraryStore.setState({ downloads: ready([job]) }); render(<DownloadQueue />);
    expect(screen.getByText(/retained locally/).textContent).not.toContain("100 GiB");
    expect(screen.getByText(/retained content available offline/)).toBeTruthy(); expect(screen.queryByText(/verified \/ /)).toBeNull();
  });
});

describe("catalog loading", () => {
  it("normalizes supported languages without treating an invalid code as a real filter", () => {
    expect(catalogLanguage(" en ")).toBe("eng"); expect(catalogLanguage("fr")).toBe("fra"); expect(catalogLanguage("jpn")).toBe("jpn"); expect(catalogLanguage("invalid")).toBeUndefined();
  });
  it("keeps cache provenance and paginates using native offsets with ID deduplication", async () => {
    const search = vi.spyOn(libraryService, "catalogSearch").mockResolvedValueOnce({ items: [entry], total: null, nextOffset: 24, source: "cache", notice: "Source unavailable" }).mockResolvedValueOnce({ items: [entry, { ...entry, catalogEntryId: "other", title: "Other" }], total: null, nextOffset: null, source: "cache" });
    await initial.searchCatalog({ query: "wiki", languageBcp47: "eng" });
    expect(useLibraryStore.getState().catalogSource).toBe("cache");
    render(<CatalogBrowser />); expect(screen.getByText(/Cached catalog results/)).toBeTruthy(); expect(screen.getByText("Source unavailable")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Load more" })); await screen.findByText("Other");
    expect(search.mock.calls[1]![0]).toMatchObject({ query: "wiki", offset: 24, languageBcp47: "eng" });
    expect(useLibraryStore.getState().catalogEntries.data).toHaveLength(2); expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
  });
  it("ignores old catalog success and failure after a newer query", async () => {
    let finish!: (page: CatalogPage) => void, fail!: (cause: Error) => void;
    const search = vi.spyOn(libraryService, "catalogSearch").mockImplementationOnce(() => new Promise(resolve => { finish = resolve; })).mockResolvedValueOnce({ items: [{ ...entry, title: "New" }], source: "live" });
    const old = initial.searchCatalog({ query: "old" }); await initial.searchCatalog({ query: "new" }); finish({ items: [entry], source: "cache" }); await old;
    expect(useLibraryStore.getState().catalogEntries.data?.[0]?.title).toBe("New"); expect(useLibraryStore.getState().catalogSource).toBe("live");
    search.mockImplementationOnce(() => new Promise((_resolve, reject) => { fail = reject; })).mockResolvedValueOnce({ items: [entry], source: "live" });
    const obsolete = initial.searchCatalog({ query: "obsolete" }); await initial.searchCatalog({ query: "wiki" }); fail(new Error("Late network failure")); await obsolete;
    expect(useLibraryStore.getState().catalogEntries.status).toBe("ready");
  });
  it("shows a real source failure as an error rather than no matches", async () => {
    vi.spyOn(libraryService, "catalogSearch").mockRejectedValue({ kind: "recoverable", message: "Kiwix unreachable", cause: null });
    await initial.searchCatalog({ query: "wiki" }); render(<CatalogBrowser />);
    expect(screen.getByText("Search failed")).toBeTruthy(); expect(screen.getByText("Kiwix unreachable")).toBeTruthy(); expect(screen.queryByText("No matches")).toBeNull();
  });
});

describe("budgeted install flow", () => {
  it("sends the chosen budget, retains the failed plan and retries the same ID", async () => {
    seedDialog(); const compute = vi.spyOn(libraryService, "installPlan").mockResolvedValue(plan);
    const start = vi.spyOn(libraryService, "installStart").mockRejectedValueOnce({ kind: "recoverable", message: "Network interrupted", cause: null }).mockResolvedValue(job);
    render(<DownloadPlanDialog />); fireEvent.change(screen.getByLabelText("Storage budget"), { target: { value: "512" } }); fireEvent.change(screen.getByLabelText("Storage budget unit"), { target: { value: "MiB" } });
    fireEvent.click(screen.getByRole("button", { name: "Compute plan" })); await screen.findByRole("button", { name: "Start download" });
    expect(compute).toHaveBeenCalledWith({ catalogEntryId: "wiki", destinationStorageRootId: "root", budgetBytes: 536870912 });
    fireEvent.click(screen.getByRole("button", { name: "Start download" })); await screen.findByText("Network interrupted");
    expect(useLibraryStore.getState().installPlan.data?.installPlanId).toBe("plan"); expect(useLibraryStore.getState().activeCatalogEntryId).toBe("wiki");
    fireEvent.click(screen.getByRole("button", { name: "Retry download" })); await waitFor(() => expect(useLibraryStore.getState().activeCatalogEntryId).toBeNull());
    expect(start.mock.calls.map(call => call[0].installPlanId)).toEqual(["plan", "plan"]);
  });
  it("invalidates a plan when destination changes and rejects its stale response", async () => {
    seedDialog(); let resolve!: (plan: InstallPlan) => void;
    vi.spyOn(libraryService, "installPlan").mockImplementation(() => new Promise(done => { resolve = done; }));
    const computing = initial.computePlan("wiki", "root", 1024 ** 3); initial.setSelectedStorageRoot("another"); resolve(plan); await computing;
    expect(useLibraryStore.getState().installPlan.data).toBeNull(); expect(await initial.startInstall("plan")).toBe(false);
  });
  it("disallows starting an old plan after the entered budget changes", async () => {
    seedDialog(); vi.spyOn(libraryService, "installPlan").mockResolvedValue(plan); render(<DownloadPlanDialog />);
    fireEvent.click(screen.getByRole("button", { name: "Compute plan" })); await screen.findByRole("button", { name: "Start download" });
    fireEvent.change(screen.getByLabelText("Storage budget"), { target: { value: "2" } }); expect(screen.queryByRole("button", { name: "Start download" })).toBeNull(); expect(screen.getByRole("button", { name: "Compute plan" })).toBeTruthy();
  });
  it("requests native cancellation while keeping the plan until transfer termination", async () => {
    seedDialog(); vi.spyOn(libraryService, "installPlan").mockResolvedValue(plan);
    let reject!: (cause: unknown) => void; vi.spyOn(libraryService, "installStart").mockImplementation(() => new Promise((_resolve, fail) => { reject = fail; }));
    const cancel = vi.spyOn(libraryService, "installCancel").mockResolvedValue(undefined); render(<DownloadPlanDialog />);
    fireEvent.click(screen.getByRole("button", { name: "Compute plan" })); await screen.findByRole("button", { name: "Start download" }); fireEvent.click(screen.getByRole("button", { name: "Start download" }));
    fireEvent.click(await screen.findByRole("button", { name: "Cancel download" })); expect(cancel).toHaveBeenCalledWith({ installPlanId: "plan" }); expect(useLibraryStore.getState().installStarting).toBe(true);
    await act(async () => reject({ kind: "recoverable", message: "Download cancelled", cause: null })); expect(await screen.findByRole("button", { name: "Retry download" })).toBeTruthy();
  });
});


it("shows current retry retained progress inside the modal until native completion", async () => {
  seedDialog(); vi.spyOn(libraryService, "installPlan").mockResolvedValue(plan);
  let finish!: (job: DownloadJob) => void;
  vi.spyOn(libraryService, "installStart").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  render(<DownloadPlanDialog />); fireEvent.click(screen.getByRole("button", { name: "Compute plan" }));
  fireEvent.click(await screen.findByRole("button", { name: "Start download" }));
  const dialog = screen.getByRole("dialog");
  act(() => useLibraryStore.setState({ downloads: ready([
    { ...job, downloadJobId: "job-plan-other", bytesRetained: 999, state: "fetching" },
    { ...job, downloadJobId: "job-plan-retry-01ARZ3NDEKTSV4RRFFQ69G5FAV", bytesRetained: 250 * 1024 ** 2, state: "fetching" },
    { ...job, downloadJobId: "job-plan", state: "cancelled", bytesRetained: 50 * 1024 ** 2 },
  ]) }));
  const progress = within(dialog).getByRole("region", { name: "Current download progress" });
  expect(within(progress).getByText("Download state: fetching")).toBeTruthy();
  expect(within(progress).getByText(/retained \/ .*planned partial payload/).textContent).toBe("250 MB retained / 500 MB planned partial payload");
  expect(within(progress).getByRole("progressbar").getAttribute("aria-valuenow")).toBe("50");
  expect(progress.textContent).not.toContain("100 GB");
  act(() => useLibraryStore.setState({ downloads: ready([{ ...job, downloadJobId: "job-plan" }]) }));
  expect(within(dialog).getByText("Finalizing reader availability…")).toBeTruthy(); expect(useLibraryStore.getState().installStarting).toBe(true);
  await act(async () => finish(job)); expect(useLibraryStore.getState().activeCatalogEntryId).toBeNull();
});
