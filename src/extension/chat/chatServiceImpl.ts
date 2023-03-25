import * as vscode from "vscode";

import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";
import { MessageItemModel } from "../../common/chatService/model";
import { SelectionRange } from "../generate/core";
import { chat } from "./core";

export interface ChatServiceClient {
    handleNewMessage?: (msg: MessageItemModel) => void;
    handleMessageChange?: (msg: MessageItemModel) => void;
}

export class ChatServiceImpl implements IChatService {
    #currentMessageId = 0;
    #messages = new Map<string, MessageItemModel>();
    #clients = new Set<ChatServiceClient>();
    #currentAbortController: AbortController | null = null;

    get name(): string {
        return CHAT_SERVICE_NAME;
    }

    attachClient(client: ChatServiceClient) {
        this.#clients.add(client);
    }

    detachClient(client: ChatServiceClient) {
        this.#clients.delete(client);

        if (this.#clients.size === 0) {
            // Abort the session when no clients connected.
            this.#currentAbortController?.abort();
            this.#currentAbortController = null;
        }
    }

    #addMessage(msg: MessageItemModel): string {
        const id = ++this.#currentMessageId;
        msg.id = msg.isReply ? `bot:${id}` : `user:${id}`;
        this.#messages.set(msg.id, msg);

        for (const client of this.#clients) {
            client.handleNewMessage?.call(client, msg);
        }

        return msg.id;
    }

    #updateMessage(msgId: string, newContents: string, finished?: boolean) {
        const msg = this.#messages.get(msgId);
        if (!msg) {
            return;
        }

        msg.contents += newContents;
        msg.isFinished = finished;

        for (const client of this.#clients) {
            client.handleMessageChange?.call(client, msg);
        }
    }

    async confirmPrompt(prompt: string): Promise<void> {
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

        this.#addMessage({
            id: "",
            contents: prompt,
        });
        const replyMsgId = this.#addMessage({
            id: "",
            contents: "",
            isReply: true,
        });

        const that = this;
        const resultStream = {
            write(value: string) {
                that.#updateMessage(replyMsgId, value as string);
            },
            end() {
                that.#updateMessage(replyMsgId, "", true);
            },
        };

        vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: "Generating reply...",
                cancellable: true,
            },
            async (_progress, token) => {
                const abortController = new AbortController();
                token.onCancellationRequested(() => {
                    abortController.abort();
                });
                this.#currentAbortController = abortController;

                try {
                    await chat(
                        prompt,
                        document,
                        selectionRange,
                        abortController.signal,
                        resultStream
                    );
                } catch (e) {
                    console.error(e);
                } finally {
                    this.#currentAbortController = null;
                }
            }
        );
    }
}

let shared: ChatServiceImpl | null = null;
export function sharedChatServiceImpl(): ChatServiceImpl {
    if (!shared) {
        shared = new ChatServiceImpl();
    }
    return shared;
}
