import { openDialog } from '../bridge';
export async function open(opts){
  // tauri open() returns string | string[] | null; we return string|null
  // opts like { directory: true }
  const path = await openDialog({ directory: !!opts?.directory, file: !!opts?.multiple === false });
  return path;
}

import { confirmDialog } from '../bridge';
export async function confirm(message, options){
  return !!(await confirmDialog(message, options));
}
