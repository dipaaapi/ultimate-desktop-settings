import { useEffect, useRef, useState } from 'react';
import { Add20Regular, Delete20Regular, FolderOpen20Regular, Pin20Filled, Pin20Regular, Rename20Regular } from '@fluentui/react-icons';
import { api } from '../lib/api';
import { timeAgo } from '../lib/format';
import { useAction } from '../lib/useAction';
import { useAsync } from '../lib/useAsync';
import { Button, ConfirmButton, ErrorBox, Loading } from '../components/ui';

const SAVE_DELAY = 600;

export default function Notes() {
  const notes = useAsync(api.listNotes);
  const { run } = useAction();
  const [selected, setSelected] = useState<string>();
  const [content, setContent] = useState('');
  const [status, setStatus] = useState<'saved' | 'saving' | 'unsaved'>('saved');
  const [renaming, setRenaming] = useState<string>();
  const pending = useRef<{ title: string; content: string }>(undefined);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const flush = async () => {
    clearTimeout(timer.current);
    const p = pending.current;
    if (!p) return;
    pending.current = undefined;
    setStatus('saving');
    await run('save', () => api.saveNote(p.title, p.content));
    setStatus('saved');
  };

  // Fall back to the first note when nothing (or a deleted note) is selected.
  const open = notes.data?.some((n) => n.title === selected) ? selected : notes.data?.[0]?.title;

  // Load the open note.
  useEffect(() => {
    if (!open) return;
    let active = true;
    api
      .readNote(open)
      .then((text) => {
        if (!active) return;
        setContent(text);
        setStatus('saved');
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, [open]);

  // Save whatever is pending when the board closes.
  useEffect(() => () => void flush(), []); // eslint-disable-line react-hooks/exhaustive-deps

  const select = async (title: string) => {
    await flush();
    setSelected(title);
  };

  if (notes.error) return <ErrorBox message={notes.error} onRetry={notes.reload} />;
  if (!notes.data) return <Loading rows={4} />;
  const current = notes.data.find((n) => n.title === open);

  return (
    <div className="notes">
      <aside className="notes-list">
        <div className="section-actions">
          <Button
            size="sm"
            variant="primary"
            icon={<Add20Regular />}
            onClick={async () => {
              await flush();
              let n = 1;
              while (notes.data!.some((x) => x.title === `New note ${n}`)) n++;
              const title = await run('new', () => api.createNote(`New note ${n}`));
              if (title) {
                notes.reload();
                setSelected(title);
              }
            }}
          >
            New
          </Button>
          <Button size="sm" variant="subtle" aria-label="Open notes folder" icon={<FolderOpen20Regular />} onClick={() => run('folder', api.openNotesFolder)} />
        </div>
        <div className="list notes-items">
          {notes.data.map((n) => (
            <button key={n.title} type="button" className="note-item" aria-current={n.title === open} onClick={() => select(n.title)}>
              <span className="row-title">
                {n.pinned && <Pin20Filled aria-label="Pinned" style={{ width: 14, height: 14 }} />}
                {n.title}
              </span>
              <span className="row-desc">{n.preview || 'Empty'}</span>
            </button>
          ))}
        </div>
      </aside>

      {current && (
        <section className="notes-editor">
          <header className="notes-editor-head">
            {renaming !== undefined ? (
              <form
                className="form-row"
                style={{ flex: 1 }}
                onSubmit={async (e) => {
                  e.preventDefault();
                  await flush();
                  const title = await run('rename', () => api.renameNote(current.title, renaming));
                  if (title) {
                    setRenaming(undefined);
                    setSelected(title);
                    notes.reload();
                  }
                }}
              >
                <input className="input" style={{ flex: 1 }} aria-label="Note title" autoFocus value={renaming} onChange={(e) => setRenaming(e.target.value)} />
                <button type="submit" className="btn btn-primary btn-sm">Save</button>
                <Button size="sm" variant="subtle" onClick={() => setRenaming(undefined)}>Cancel</Button>
              </form>
            ) : (
              <>
                <div className="row-text">
                  <div className="row-title">{current.title}</div>
                  <div className="row-desc">
                    {status === 'saving' ? 'Saving…' : status === 'unsaved' ? 'Editing…' : `Saved · ${timeAgo(current.modifiedMs)}`}
                  </div>
                </div>
                <Button size="sm" variant="subtle" aria-label="Rename" icon={<Rename20Regular />} onClick={() => setRenaming(current.title)} />
                <Button
                  size="sm"
                  variant="subtle"
                  aria-label={current.pinned ? 'Unpin' : 'Pin'}
                  aria-pressed={current.pinned}
                  icon={current.pinned ? <Pin20Filled /> : <Pin20Regular />}
                  onClick={async () => {
                    await run('pin', () => api.pinNote(current.title, !current.pinned));
                    notes.reload();
                  }}
                />
                <ConfirmButton
                  size="sm"
                  variant="subtle"
                  aria-label="Delete note"
                  icon={<Delete20Regular />}
                  disabled={current.pinned}
                  title={current.pinned ? 'Unpin the note to delete it' : undefined}
                  confirmLabel="Delete?"
                  onConfirm={async () => {
                    pending.current = undefined;
                    const ok = await run('delete', () => api.deleteNote(current.title), `"${current.title}" deleted.`);
                    if (ok !== undefined) {
                      setSelected(undefined);
                      notes.reload();
                    }
                  }}
                />
              </>
            )}
          </header>
          <textarea
            className="textarea notes-text"
            aria-label={`${current.title} text`}
            value={content}
            placeholder="Start writing…"
            onChange={(e) => {
              setContent(e.target.value);
              setStatus('unsaved');
              pending.current = { title: current.title, content: e.target.value };
              clearTimeout(timer.current);
              timer.current = setTimeout(() => void flush().then(notes.reload), SAVE_DELAY);
            }}
            onBlur={() => void flush()}
          />
        </section>
      )}
    </div>
  );
}
