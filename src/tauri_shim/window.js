import { windowMinimize, windowClose } from '../bridge';
export const appWindow = {
  minimize: async () => windowMinimize(),
  close: async () => windowClose(),
};
