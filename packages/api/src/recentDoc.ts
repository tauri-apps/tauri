import { invoke } from './core';

async function addRecentDocument(path: string): Promise<void> {
  return invoke('plugin:recent_doc|add', { path });
}

async function getRecentDocuments(): Promise<string[]> {
  return invoke<string[]>('plugin:recent_doc|list');
}

async function clearRecentDocument(path: string): Promise<void> {
  return invoke('plugin:recent_doc|clear', { path });
}

export {
  addRecentDocument,
  getRecentDocuments,
  clearRecentDocument
}
