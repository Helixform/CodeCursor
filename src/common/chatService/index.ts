import { IService } from "../ipc";
import { MessageItemModel } from "./model";

export const CHAT_SERVICE_NAME = "chat";

export interface IChatService extends IService {
    confirmPrompt(prompt: string): Promise<void>;
}

export const CHAT_VIEW_SERVICE_NAME = "chat_view";

export interface IChatViewService extends IService {
    setHasSelection(hasSelection: boolean): Promise<void>;
    addMessage(msg: MessageItemModel): Promise<void>;
    updateMessage(msg: MessageItemModel): Promise<void>;
}
