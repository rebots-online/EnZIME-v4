import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { MemoryRouter, Routes, Route, useLocation, useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { PackGrid } from "./PackGrid";
import { useLibraryStore } from "../../stores/libraryStore";
import { useReaderStore } from "../../stores/readerStore";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const initial = useLibraryStore.getState();
beforeEach(() => {
  vi.clearAllMocks();
  useLibraryStore.setState(initial, true);
  useLibraryStore.setState({ online: true, packs: { status: "ready", error: null, lastUpdated: null, data: [
    { packId: "real-pack", displayName: "Football", description: "", languageBcp47: "eng", articleCount: 549, sizeBytes: 7700000 },
  ] } });
});
afterEach(cleanup);
function Destination() { const params = useParams(); return <pre data-testid="destination">{JSON.stringify(params)}</pre>; }
function setup() {
  let pathname = "/";
  let navigate!: ReturnType<typeof useNavigate>;
  function Observe() { pathname = useLocation().pathname; navigate = useNavigate(); return null; }
  render(<MemoryRouter><Observe /><Routes>
    <Route path="/" element={<PackGrid />} />
    <Route path="/other" element={<p>Other screen</p>} />
    <Route path="/reader/:packId/:articleKey" element={<Destination />} />
  </Routes></MemoryRouter>);
  return { get state() { return { location: { pathname } }; }, navigate: (path: string) => navigate(path) };
}

it("routes the actual native main page without inventing a namespace landing page", async () => {
  const key = "C/football database/100%25.html";
  vi.mocked(invoke).mockResolvedValue({ packId: "real-pack", title: "Football", mainPageKey: key });
  const router = setup();
  fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  const result = await screen.findByTestId("destination");
  expect(result.textContent).toBe(JSON.stringify({ packId: "real-pack", articleKey: key }));
  expect(router.state.location.pathname).toBe(`/reader/real-pack/${encodeURIComponent(key)}`);
  expect(invoke).toHaveBeenCalledExactlyOnceWith("reader_open", { packId: "real-pack" });
});
it("prevents duplicate opens and ignores a completion after leaving Library", async () => {
  let finish!: (value: unknown) => void;
  vi.mocked(invoke).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const router = setup(); const tile = screen.getByRole("button", { name: "Open pack Football" });
  fireEvent.click(tile); fireEvent.click(tile);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect((tile as HTMLButtonElement).disabled).toBe(true);
  await act(() => router.navigate("/other"));
  await act(async () => finish({ packId: "real-pack", title: "Football", mainPageKey: "C/index.html" }));
  expect(router.state.location.pathname).toBe("/other");
});
it.each([null, "", "   "])("keeps missing main page %s retryable in Library", async mainPageKey => {
  vi.mocked(invoke).mockResolvedValueOnce({ packId: "real-pack", mainPageKey }).mockResolvedValueOnce({ packId: "real-pack", mainPageKey: "C/index.html" });
  const router = setup();
  fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  await screen.findByText(/no readable main page/);
  expect(router.state.location.pathname).toBe("/");
  fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  await screen.findByTestId("destination");
  expect(invoke).toHaveBeenCalledTimes(2);
});
it("shows the native error and permits retry without fabricated content", async () => {
  vi.mocked(invoke).mockRejectedValueOnce("offline catalog unavailable").mockResolvedValueOnce({ packId: "real-pack", mainPageKey: "C/main" });
  setup(); fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  await screen.findByText(/offline catalog unavailable/);
  fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  await screen.findByTestId("destination");
});
it("rejects a response for a different pack", async () => {
  vi.mocked(invoke).mockResolvedValue({ packId: "different", mainPageKey: "C/main" });
  const router = setup(); fireEvent.click(screen.getByRole("button", { name: "Open pack Football" }));
  await screen.findByText(/different pack/); expect(router.state.location.pathname).toBe("/");
});
it("never requests a null article through the existing store open API", async () => {
  vi.mocked(invoke).mockResolvedValue({ packId: "real-pack", title: "Football", mainPageKey: null });
  expect(await useReaderStore.getState().open("real-pack")).toBeNull();
  expect(invoke).toHaveBeenCalledExactlyOnceWith("reader_open", { packId: "real-pack" });
  expect(useReaderStore.getState().currentArticle.error?.message).toContain("no readable main page");
});
