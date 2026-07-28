import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useHomeStore } from "../../stores/homeStore";
import type { TaskKind, TaskSummary } from "../../services/homeService";
import {
  LinkButton,
  Progress,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
  resolveState,
} from "./shared";

function routeForTask(task: TaskSummary): string {
  switch (task.kind) {
    case "download":
    case "index":
      return "/library";
    case "inference":
      return task.target
        ? `/ask?pack=${encodeURIComponent(task.target.packId)}`
        : "/ask";
    case "lore":
      return "/memory";
    case "backup":
      return "/settings";
    default:
      return "/";
  }
}

function phaseLabel(task: TaskSummary): string {
  switch (task.phase) {
    case "queued":
      return "queued";
    case "running":
      return task.progress < 0
        ? "in progress"
        : `${Math.round(task.progress * 100)}%`;
    case "paused":
      return "paused";
    case "awaiting-input":
      return "awaiting input";
    default:
      return "";
  }
}

function kindLabel(kind: TaskKind): string {
  switch (kind) {
    case "download":
      return "Download";
    case "index":
      return "Index";
    case "inference":
      return "Inference";
    case "lore":
      return "Memory";
    case "backup":
      return "Backup";
    default:
      return "Task";
  }
}

function TaskRow({
  task,
  navigate,
}: {
  task: TaskSummary;
  navigate: (path: string) => void;
}): ReactNode {
  const needsInput = task.phase === "awaiting-input";
  const indeterminate = task.phase === "running" && task.progress < 0;
  const showProgress =
    task.phase === "running" || task.phase === "queued" || task.phase === "paused";
  return (
    <li className="home-list__item">
      <button
        type="button"
        className="home-row"
        onClick={() => navigate(routeForTask(task))}
        aria-label={`${kindLabel(task.kind)}: ${task.label} — ${phaseLabel(task)}${
          needsInput ? ", needs your input" : ""
        }`}
      >
        <span className="home-row__main">
          <span className="home-row__label">
            {task.label}
            {needsInput ? (
              <span
                className="home-pill home-pill--warning"
                style={{ marginLeft: "0.5rem" }}
              >
                needs input
              </span>
            ) : null}
          </span>
          {showProgress ? (
            <Progress
              value={task.progress < 0 ? 0 : task.progress}
              indeterminate={indeterminate}
            />
          ) : (
            <span className="home-row__meta">{phaseLabel(task)}</span>
          )}
        </span>
        <span className="home-row__aside">{phaseLabel(task)}</span>
      </button>
    </li>
  );
}

export function TaskInbox(): ReactNode {
  const navigate = useNavigate();
  const online = useHomeStore((s) => s.online);
  const slice = useHomeStore((s) => s.taskInbox);
  const reload = useHomeStore((s) => s.loadTaskInbox);

  const state = resolveState(online, slice, (data) => data.length > 0);
  const tasks = slice.data ?? [];
  const awaiting = tasks.filter((t) => t.phase === "awaiting-input").length;

  let body: ReactNode;
  if (state === "offline-empty") {
    body = (
      <StatusBanner
        tone="offline"
        icon="!"
        title="Offline"
        detail="Reconnect to load active tasks."
      />
    );
  } else if (state === "loading") {
    body = <SkeletonBlock lines={2} />;
  } else if (state === "populated") {
    body = (
      <ul className="home-list" aria-label="Active tasks">
        {tasks.map((task) => (
          <TaskRow key={task.taskId} task={task} navigate={navigate} />
        ))}
      </ul>
    );
  } else if (state === "empty") {
    body = (
      <StatusBanner
        tone="info"
        icon="✓"
        title="No active tasks"
        detail="Start a download, ask a question, or run a backup to see progress here."
        actions={
          <LinkButton to="/library" navigate={navigate} variant="ghost">
            Open library
          </LinkButton>
        }
      />
    );
  } else if (state === "recoverable") {
    body = (
      <StatusBanner
        tone="warning"
        icon="!"
        title="Could not load tasks"
        detail={slice.error?.message ?? "Unexpected error."}
        actions={<RetryButton onClick={reload} label="Try again" />}
      />
    );
  } else if (state === "non-recoverable") {
    body = (
      <StatusBanner
        tone="error"
        icon="×"
        title="Task queue unavailable"
        detail={slice.error?.message ?? "The task queue could not be read."}
      />
    );
  } else {
    body = (
      <StatusBanner
        tone="locked"
        icon="∅"
        title="Task queue locked"
        detail={
          slice.error?.message ?? "This profile cannot view background tasks."
        }
        actions={
          <LinkButton to="/settings" navigate={navigate} variant="ghost">
            Open settings
          </LinkButton>
        }
      />
    );
  }

  const headingHint =
    state === "populated"
      ? awaiting > 0
        ? `${awaiting} need${awaiting === 1 ? "s" : ""} input`
        : `${tasks.length} active`
      : null;

  return (
    <section
      className="home-card home-card--tasks"
      aria-labelledby="home-tasks-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-tasks-heading">
          Task inbox
        </h2>
        {headingHint ? (
          <span
            className={
              awaiting > 0
                ? "home-pill home-pill--warning"
                : "home-card__hint"
            }
          >
            {headingHint}
          </span>
        ) : null}
      </header>
      {body}
    </section>
  );
}
