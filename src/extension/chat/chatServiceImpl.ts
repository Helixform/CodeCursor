import * as vscode from "vscode";

import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";
import { MessageItemModel } from "../../common/chatService/model";
import { SelectionRange } from "../generate/core";
import { chat, resetChat } from "./core";

export interface ChatServiceClient {
    handleReadyStateChange?: (isReady: boolean) => void;
    handleNewMessage?: (msg: MessageItemModel) => void;
    handleMessageChange?: (msg: MessageItemModel) => void;
    handleClearMessage?: () => void;
}

export class ChatServiceImpl implements IChatService {
    #currentMessageId = 0;
    #messages = new Array<MessageItemModel>();
    #messageIndex = new Map<string, MessageItemModel>();
    #clients = new Set<ChatServiceClient>();
    #currentAbortController: AbortController | null = null;
    #clearSessionScheduled = false;

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

    clearSession() {
        const abortController = this.#currentAbortController;
        if (abortController) {
            this.#clearSessionScheduled = true;
            abortController.abort();
            return;
        }

        resetChat();
        this.#messageIndex.clear();
        this.#messages.splice(0, this.#messages.length);
        for (const client of this.#clients) {
            client.handleClearMessage?.call(client);
        }
        this.#clearSessionScheduled = false;

        vscode.window.showInformationMessage("Chat session has been reset!");
    }

    #updateReadyState(isReady: boolean) {
        for (const client of this.#clients) {
            client.handleReadyStateChange?.call(client, isReady);
        }
    }

    #addMessage(msg: MessageItemModel): string {
        const id = ++this.#currentMessageId;
        msg.id = msg.isReply ? `bot:${id}` : `user:${id}`;
        this.#messages.push(msg);
        this.#messageIndex.set(msg.id, msg);

        for (const client of this.#clients) {
            client.handleNewMessage?.call(client, msg);
        }

        return msg.id;
    }

    #updateMessage(msgId: string, newContents: string, finished?: boolean) {
        const msg = this.#messageIndex.get(msgId);
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
        if (this.#currentAbortController) {
            // TODO: optimize the UX.
            console.warn("A chat session is in-flight");
            return;
        }

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
                this.#updateReadyState(false);

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
                    // TODO: optimize the display of error message.
                    this.#updateMessage(
                        replyMsgId,
                        "\n(Response interrupted)",
                        true
                    );
                } finally {
                    this.#currentAbortController = null;
                    this.#updateReadyState(true);
                    if (this.#clearSessionScheduled) {
                        this.clearSession();
                    }
                }
            }
        );
    }

    async syncState(): Promise<void> {
        for (const msg of this.#messages) {
            for (const client of this.#clients) {
                client.handleNewMessage?.call(client, msg);
            }
        }

        const isReady = this.#currentAbortController === null;
        this.#updateReadyState(isReady);
    }

    async insertCodeSnippet(contents: string): Promise<void> {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            return;
        }

        await activeEditor.insertSnippet(
            new vscode.SnippetString(contents),
            activeEditor.selection
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
