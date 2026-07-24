/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ApprovalDialog } from './ApprovalDialog';

describe('ApprovalDialog', () => {
  it('requires a click before approving', () => {
    const onRespond = vi.fn();
    render(<ApprovalDialog request={{ id: 'a1', description: '读取项目文件' }} onRespond={onRespond} />);

    expect(onRespond).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: '允许' }));

    expect(onRespond).toHaveBeenCalledWith('a1', true);
  });
});
