import {
    ServiceManager,
    IMessageReplyPort,
    MESSAGE_TYPE_REQUEST,
    MESSAGE_TYPE_RESPONSE,
} from "./base";

class WebviewServiceManager extends ServiceManager {
    #currentMessageId = 0;
    #pendingReplies = new Map<number, IMessageReplyPort>();
    #vscode: ReturnType<typeof acquireVsCodeApi>;

    constructor(vscode: ReturnType<typeof acquireVsCodeApi>) {
        super();
        this.#vscode = vscode;
    }

    protected sendOutgoingMessage(
        msg: unknown,
        replyPort: IMessageReplyPort
    ): void {
        const id = ++this.#currentMessageId;
        this.#pendingReplies.set(id, replyPort);
        this.#vscode.postMessage({
            id,
            type: MESSAGE_TYPE_REQUEST,
            payload: msg,
        });
    }

    async _sendRawIncomingMessage(rawMsg: any) {
        const { id, type, payload } = rawMsg;

        if (type === MESSAGE_TYPE_RESPONSE) {
            const replyPort = this.#pendingReplies.get(id);
            if (!replyPort) {
                console.warn(`Unexpected reply with id: ${id}`);
                return;
            }
            this.#pendingReplies.delete(id);
            replyPort.sendReply(payload);
        } else {
            const reply = await this.handleIncomingMessage(payload);
            this.#vscode.postMessage({
                id,
                type: MESSAGE_TYPE_RESPONSE,
                payload: reply,
            });
        }
    }
}

let serviceManager: WebviewServiceManager | null;

export function getServiceManager(): ServiceManager {
    if (serviceManager) {
        return serviceManager;
    }

    // Setup the message port.
    window.addEventListener("message", (event) => {
        const message = event.data;
        serviceManager!._sendRawIncomingMessage(message);
    });

    serviceManager = new WebviewServiceManager(acquireVsCodeApi());
    return serviceManager;
}
