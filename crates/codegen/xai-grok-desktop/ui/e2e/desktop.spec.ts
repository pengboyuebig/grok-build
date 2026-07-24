import { test, expect } from '@playwright/test';

test.describe('Grok Desktop', () => {
  test('sends a prompt, approves a tool, and launches the terminal session', async ({ page }) => {
    // The desktop binary must be built and served before this test runs.
    await page.goto('http://localhost:1420');

    // Compose and send a message.
    const composer = page.getByLabelText('消息输入');
    await composer.fill('Fix the failing tests');
    await page.getByRole('button', { name: '发送' }).click();

    // Wait for an approval request to appear.
    const approval = page.getByRole('button', { name: '允许' });
    await approval.waitFor({ timeout: 30_000 });
    await approval.click();

    // Launch the terminal session.
    await page.getByRole('button', { name: '打开终端会话' }).click();

    // Expect a terminal-handoff activity row.
    await expect(page.getByText('终端会话已启动')).toBeVisible();
  });
});
