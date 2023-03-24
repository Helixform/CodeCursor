import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";

export class ChatServiceImpl implements IChatService {
    #currentMessageId = 0;

    get name(): string {
        return CHAT_SERVICE_NAME;
    }

    async confirmPrompt(prompt: string): Promise<number> {
        const id = ++this.#currentMessageId;
        console.log(`Got user prompt: ${prompt}`);
        return id;
    }
}

let shared: ChatServiceImpl | null = null;
export function sharedChatServiceImpl(): ChatServiceImpl {
    if (!shared) {
        shared = new ChatServiceImpl();
    }
    return shared;
}
