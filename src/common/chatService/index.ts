import { IService } from "../ipc";

export const CHAT_SERVICE_NAME = "chat";

export interface IChatService extends IService {
    confirmPrompt(prompt: string): Promise<number>;
}

export const CHAT_VIEW_SERVICE_NAME = "chat_view";

export interface IChatViewService extends IService {
    setHasSelection(hasSelection: boolean): Promise<void>;
    updateMessage(msgId: number, contents: string): Promise<void>;
}
