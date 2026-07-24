/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { CommandMenu } from './CommandMenu';

describe('CommandMenu', () => {
  it('opens a parameter form for rename', () => {
    render(
      <CommandMenu
        commands={[
          {
            slash: '/rename',
            kind: 'form',
            requires_confirmation: false,
            can_spawn_process: false,
            arguments: [{ name: 'session_name', required: true }],
          },
        ]}
        onDispatch={() => undefined}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: '/rename' }));

    expect(screen.getByLabelText('会话名称')).toBeTruthy();
  });
});
