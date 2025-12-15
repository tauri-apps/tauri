import { invoke } from './core';

async function addRecentDocument(path: string): Promise<void> {
  return invoke('plugin:recent_doc|add_recent_document', { path });
}

async function getRecentDocuments(): Promise<string[]> {
  return invoke<string[]>('plugin:recent_doc|get_recent_documents');
}

async function clearRecentDocuments(): Promise<void> {
  return invoke('plugin:recent_doc|clear_recent_documents');
}

export {
  addRecentDocument,
  getRecentDocuments,
  clearRecentDocuments
}
