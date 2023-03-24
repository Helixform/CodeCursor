import { IService } from "../ipc";

export const CHAT_SERVICE_NAME = "chat";

export interface IChatService extends IService {
    confirmPrompt(prompt: string): Promise<number>;
}
