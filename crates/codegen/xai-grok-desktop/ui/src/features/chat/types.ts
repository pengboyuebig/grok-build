/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

export type ChatMessage = {
  id: string;
  role: 'assistant' | 'user';
  text: string;
};
