import { describe, expect, it } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from '../App';
import { PreferencesProvider } from '../lib/theme';
import { ToastProvider } from '../components/Toasts';
import { ALL_BEADS } from '../beads';

function renderApp() {
  return render(
    <PreferencesProvider>
      <ToastProvider>
        <App />
      </ToastProvider>
    </PreferencesProvider>,
  );
}

describe('App', () => {
  it('shows the title badge, every bead and the preview banner', () => {
    renderApp();
    expect(screen.getByRole('button', { name: /Ultimate Windows Desktop Settings/ })).toBeInTheDocument();
    for (const bead of ALL_BEADS) {
      expect(screen.getByRole('button', { name: `${bead.title}: ${bead.subtitle}` })).toBeInTheDocument();
    }
    expect(screen.getByText(/Preview mode/)).toBeInTheDocument();
  });

  it.each(ALL_BEADS.map((b) => [b.title, b] as const))('opens and closes the %s board', async (_title, bead) => {
    const user = userEvent.setup();
    renderApp();
    await user.click(screen.getByRole('button', { name: `${bead.title}: ${bead.subtitle}` }));
    const dialog = await screen.findByRole('dialog', { name: bead.title });
    // Each board loads its sample data without showing an error.
    await within(dialog).findAllByText(/./, { selector: '.row-title, .tile-title, .stat-label, .big-clock, .tab' }, { timeout: 3000 });
    expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
    // jsdom cannot finish the shared-layout exit animation, so check the bead instead of the dialog.
    const beadButton = screen.getByRole('button', { name: `${bead.title}: ${bead.subtitle}` });
    expect(beadButton).toHaveAttribute('aria-expanded', 'true');
    await user.keyboard('{Escape}');
    await waitFor(() => expect(beadButton).toHaveAttribute('aria-expanded', 'false'));
  });

  it('lists the tech stack in About & Credits', async () => {
    const user = userEvent.setup();
    renderApp();
    await user.click(screen.getByRole('button', { name: /^About & Credits:/ }));
    const dialog = await screen.findByRole('dialog', { name: 'About & Credits' });
    expect(await within(dialog).findByText('The tech stack is listed here.')).toBeInTheDocument();
    for (const name of ['Tauri', 'Rust', 'React', 'TypeScript']) expect(within(dialog).getByText(name)).toBeInTheDocument();
    expect(within(dialog).getByText('Built with Claude')).toBeInTheDocument();
  });

  it('shows minimize and shut down when the title is clicked', async () => {
    const user = userEvent.setup();
    renderApp();
    expect(screen.queryByRole('button', { name: 'Shut down' })).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /Show minimize and shut down/ }));
    expect(screen.getByRole('button', { name: 'Minimize to tray' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Shut down' }));
    // The preview cannot quit; it says so instead.
    expect(await screen.findByText(/works in the Windows app/)).toBeInTheDocument();
  });
});
