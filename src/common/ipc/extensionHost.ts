import * as vscode from "vscode";

import {
    ServiceManager,
    IMessageReplyPort,
    MESSAGE_TYPE_REQUEST,
    MESSAGE_TYPE_RESPONSE,
} from "./base";

export class ExtensionHostServiceManager extends ServiceManager {
    #webview: vscode.Webview;
    #currentMessageId = 0;
    #pendingReplies = new Map<number, IMessageReplyPort>();
    #disposable: vscode.Disposable;

    constructor(webview: vscode.Webview) {
        super();
        this.#webview = webview;

        this.#disposable = webview.onDidReceiveMessage(async (rawMsg) => {
            const { id, type, payload } = rawMsg;
            if (type === MESSAGE_TYPE_REQUEST) {
                const reply = await this.handleIncomingMessage(payload);
                webview.postMessage({
                    id,
                    type: MESSAGE_TYPE_RESPONSE,
                    payload: reply,
                });
            } else {
                const replyPort = this.#pendingReplies.get(id);
                if (!replyPort) {
                    console.warn(`Unexpected reply with id: ${id}`);
                    return;
                }
                this.#pendingReplies.delete(id);
                replyPort.sendReply(payload);
            }
        });
    }

    dispose() {
        this.#disposable.dispose();
    }

    protected sendOutgoingMessage(
        msg: unknown,
        replyPort: IMessageReplyPort
    ): void {
        const id = ++this.#currentMessageId;
        this.#pendingReplies.set(id, replyPort);
        this.#webview.postMessage({
            id,
            type: MESSAGE_TYPE_REQUEST,
            payload: msg,
        });
    }
}
