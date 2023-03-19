export interface BotMessage {
    sender: string;
    sendAt: number;
    conversationId: string;
    type: string;
    message: string;
    lastToken: string;
    finished: boolean;
    currentFile: string;
    interrupted: boolean;
    maxOrigLine: number;
    hitTokenLimit: boolean;
}

export enum BotMessageType {
    edit = "edit",
    generate = "generate",
}

export function interruptedBotMessage(
    messageType: BotMessageType,
    conversationId: string,
    message: string,
    lastToken: string,
    currentFile: string
): BotMessage {
    return {
        sender: "bot",
        sendAt: Date.now(),
        conversationId: conversationId,
        type: messageType,
        message: message,
        lastToken: lastToken,
        finished: false,
        currentFile: currentFile,
        interrupted: true,
        maxOrigLine: Math.floor(Math.random() * 1000) + 1,
        hitTokenLimit: true,
    };
}
