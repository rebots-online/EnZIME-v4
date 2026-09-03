import type { JSX } from "react";
import {Fragment, type CSSProperties, useId, useState} from "react";
import type {NotebookNode, NotebookUpdateRequest, Ulid} from "../../services/annotationService";
import {EzButton, EzIconButton} from "./sharedUi";

export interface NotebookTreeProps {
  nodes: NotebookNode[];
  selectedNotebookIds: Ulid[];
  onSelectionChange: (notebookIds: Ulid[]) => void;
  onCreate: (parentNotebookId?: Ulid) => void;
  onUpdateNotebook?: (request: NotebookUpdateRequest) => void;
}

export function NotebookTree(props: NotebookTreeProps): JSX.Element {
  const headingId = useId();
  return (
    <section aria-labelledby={headingId} style={sectionStyle}>
      <header style={headerStyle}>
        <h3 id={headingId} style={headingStyle}>
          Notebooks
        </h3>
        <EzIconButton
          ariaLabel="Create top-level notebook"
          onClick={() => props.onCreate(undefined)}
          title="New notebook"
        >
          +
        </EzIconButton>
      </header>
      {props.nodes.length === 0 ? (
        <p style={{margin: 0, color: "var(--ez-color-text-muted)"}}>
          No notebooks yet. Group annotations by topic — create a notebook to
          start organising highlights, comments, ink, and voice notes.
        </p>
      ) : (
        <ul role="tree" style={listStyle}>
          {props.nodes.map((node) => (
            <NotebookRow
              key={node.notebookId}
              node={node}
              depth={0}
              selectedNotebookIds={props.selectedNotebookIds}
              onSelectionChange={props.onSelectionChange}
              onCreate={props.onCreate}
              onUpdateNotebook={props.onUpdateNotebook}
            />
          ))}
        </ul>
      )}
    </section>
  );
}

interface NotebookRowProps {
  node: NotebookNode;
  depth: number;
  selectedNotebookIds: Ulid[];
  onSelectionChange: (ids: Ulid[]) => void;
  onCreate: (parentNotebookId?: Ulid) => void;
  onUpdateNotebook?: (request: NotebookUpdateRequest) => void;
}

function NotebookRow(props: NotebookRowProps): JSX.Element {
  const {node} = props;
  const [expanded, setExpanded] = useState(true);
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(node.name);
  const [description, setDescription] = useState(node.description);
  const selected = props.selectedNotebookIds.includes(node.notebookId);
  const toggleSelected = () => {
    if (selected) {
      props.onSelectionChange(
        props.selectedNotebookIds.filter((id) => id !== node.notebookId),
      );
    } else {
      props.onSelectionChange([...props.selectedNotebookIds, node.notebookId]);
    }
  };

  const commitEdit = () => {
    if (!props.onUpdateNotebook) {
      setEditing(false);
      return;
    }
    const trimmedName = name.trim();
    if (trimmedName.length === 0) {
      return;
    }
    props.onUpdateNotebook({
      notebookId: node.notebookId,
      name: trimmedName,
      description: description.trim(),
      parentNotebookId: node.parentNotebookId,
    });
    setEditing(false);
  };

  return (
    <li
      role="treeitem"
      aria-expanded={expanded}
      aria-selected={selected}
      style={{listStyle: "none", margin: 0, padding: 0}}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--ez-space-1)",
          padding: `var(--ez-space-1) 0`,
          paddingLeft: `calc(var(--ez-space-3) * ${props.depth})`,
        }}
      >
        {node.children.length > 0 ? (
          <EzIconButton
            ariaLabel={expanded ? `Collapse ${node.name}` : `Expand ${node.name}`}
            onClick={() => setExpanded((value) => !value)}
            title={expanded ? "Collapse" : "Expand"}
            style={{width: "1.5rem", height: "1.5rem"}}
          >
            {expanded ? "▾" : "▸"}
          </EzIconButton>
        ) : (
          <span style={{display: "inline-block", width: "1.5rem"}} aria-hidden="true" />
        )}
        <label
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "var(--ez-space-2)",
            flex: 1,
            minWidth: 0,
            cursor: "pointer",
          }}
        >
          <input
            type="checkbox"
            checked={selected}
            onChange={toggleSelected}
            aria-label={`Select notebook ${node.name}`}
          />
          {editing ? (
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              aria-label="Edit notebook name"
              style={inputStyle}
              autoComplete="off"
              spellCheck={false}
            />
          ) : (
            <span style={nameStyle}>{node.name}</span>
          )}
          <span style={countStyle}>
            {node.annotationCount} annotation
            {node.annotationCount === 1 ? "" : "s"}
          </span>
        </label>
        {props.onUpdateNotebook ? (
          <EzIconButton
            ariaLabel={editing ? `Save name for ${node.name}` : `Rename ${node.name}`}
            onClick={() => (editing ? commitEdit() : setEditing(true))}
            title={editing ? "Save" : "Rename"}
          >
            {editing ? "⏎" : "✎"}
          </EzIconButton>
        ) : null}
        <EzIconButton
          ariaLabel={`Create child notebook under ${node.name}`}
          onClick={() => props.onCreate(node.notebookId)}
          title="New child notebook"
        >
          +
        </EzIconButton>
      </div>
      {editing ? (
        <div
          style={{
            paddingLeft: `calc(var(--ez-space-3) * ${props.depth + 1})`,
            paddingTop: "var(--ez-space-1)",
            display: "flex",
            flexDirection: "column",
            gap: "var(--ez-space-2)",
          }}
        >
          <label
            style={{
              fontSize: "0.75rem",
              color: "var(--ez-color-text-secondary)",
            }}
          >
            Description
            <textarea
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              style={{...inputStyle, marginTop: "var(--ez-space-1)"}}
              rows={2}
            />
          </label>
          <div style={{display: "inline-flex", gap: "var(--ez-space-2)"}}>
            <EzButton
              onClick={commitEdit}
              disabled={name.trim().length === 0}
              ariaLabel={`Save edits to ${node.name}`}
            >
              Save
            </EzButton>
            <EzButton
              variant="ghost"
              onClick={() => {
                setEditing(false);
                setName(node.name);
                setDescription(node.description);
              }}
              ariaLabel="Cancel notebook edits"
            >
              Cancel
            </EzButton>
          </div>
        </div>
      ) : null}
      {expanded && node.children.length > 0 ? (
        <ul role="group" style={listStyle}>
          {node.children.map((child) => (
            <Fragment key={child.notebookId}>
              <NotebookRow
                node={child}
                depth={props.depth + 1}
                selectedNotebookIds={props.selectedNotebookIds}
                onSelectionChange={props.onSelectionChange}
                onCreate={props.onCreate}
                onUpdateNotebook={props.onUpdateNotebook}
              />
            </Fragment>
          ))}
        </ul>
      ) : null}
    </li>
  );
}

const sectionStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
};

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "0.95rem",
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--ez-color-text-secondary)",
};

const listStyle: CSSProperties = {
  listStyle: "none",
  margin: 0,
  padding: 0,
  display: "flex",
  flexDirection: "column",
};

const nameStyle: CSSProperties = {
  color: "var(--ez-color-text-primary)",
  fontWeight: 500,
};

const countStyle: CSSProperties = {
  color: "var(--ez-color-text-muted)",
  fontSize: "0.8rem",
  fontFamily: "var(--ez-font-mono)",
};

const inputStyle: CSSProperties = {
  background: "var(--ez-color-bg-elevated)",
  border: "1px solid var(--ez-color-border-subtle)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-1) var(--ez-space-2)",
  color: "var(--ez-color-text-primary)",
  fontFamily: "inherit",
  fontSize: "0.9rem",
  width: "100%",
};
