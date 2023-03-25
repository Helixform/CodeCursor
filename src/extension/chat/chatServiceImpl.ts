import * as vscode from "vscode";

import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";
import { SelectionRange } from "../generate/core";
import { chat } from "./core";

export interface ChatMessage {
    id: number;
    contents: string;
}

export interface ChatServiceClient {
    handleMessageChange?: (msg: ChatMessage) => void;
}

export class ChatServiceImpl implements IChatService {
    #currentMessageId = 0;
    #messages = new Map<number, ChatMessage>();
    #clients = new Set<ChatServiceClient>();

    get name(): string {
        return CHAT_SERVICE_NAME;
    }

    attachClient(client: ChatServiceClient) {
        this.#clients.add(client);
    }

    detachClient(client: ChatServiceClient) {
        this.#clients.delete(client);
    }

    notifyClientMessageChange(msgId: number) {
        const msg = this.#messages.get(msgId);
        if (!msg) {
            return;
        }

        for (const client of this.#clients) {
            client.handleMessageChange?.call(client, msg);
        }
    }

    async confirmPrompt(prompt: string): Promise<number> {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            throw new Error("No active editor");
        }

        const { document, selection } = editor;

        const selectionStartOffset = document.offsetAt(selection.start);
        const selectionEndOffset = document.offsetAt(selection.end);
        const selectionRange = new SelectionRange(
            selectionStartOffset,
            selectionEndOffset - selectionStartOffset
        );

        const id = ++this.#currentMessageId;
        const msg: ChatMessage = {
            id,
            contents: "",
        };
        this.#messages.set(id, msg);

        vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: "Generating reply...",
                cancellable: true,
            },
            async (_progress, token) => {
                try {
                    const that = this;
                    await chat(prompt, document, selectionRange, token, {
                        write(value) {
                            msg.contents += value;
                            that.notifyClientMessageChange(msg.id);
                        },
                        end() {
                            console.log("end");
                        },
                    });
                } catch (e) {
                    console.error(e);
                    return;
                }
            }
        );

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
