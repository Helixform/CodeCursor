import * as vscode from "vscode";

import { getNonce } from "../utils";
import {
    sharedChatServiceImpl,
    ChatServiceClient,
    ChatMessage,
} from "./chatServiceImpl";
import { ExtensionHostServiceManager } from "../../common/ipc/extensionHost";
import {
    IChatViewService,
    CHAT_VIEW_SERVICE_NAME,
} from "../../common/chatService";

export class ChatPanelProvider
    implements vscode.WebviewViewProvider, ChatServiceClient
{
    static readonly viewType = "chat";

    #view: vscode.WebviewView | null = null;
    #extensionContext: vscode.ExtensionContext;
    #serviceManager: ExtensionHostServiceManager | null = null;

    constructor(extensionContext: vscode.ExtensionContext) {
        this.#extensionContext = extensionContext;
    }

    resolveWebviewView(
        webviewView: vscode.WebviewView,
        _context: vscode.WebviewViewResolveContext<unknown>,
        _token: vscode.CancellationToken
    ): void | Thenable<void> {
        this.#view = webviewView;

        const { extensionUri } = this.#extensionContext;
        const { webview } = webviewView;
        const baseUri = vscode.Uri.joinPath(extensionUri, "dist");
        webview.options = {
            enableScripts: true,
            localResourceRoots: [baseUri],
        };
        webview.html = ChatPanelProvider.#buildWebviewContents(
            webview,
            baseUri
        );

        const chatService = sharedChatServiceImpl();
        chatService.attachClient(this);

        const serviceManager = new ExtensionHostServiceManager(webview);
        serviceManager.registerService(chatService);
        this.#serviceManager = serviceManager;

        const eventDisposable = vscode.window.onDidChangeTextEditorSelection(
            async (e) => {
                const hasSelection = !e.selections[0].isEmpty;
                const chatViewService =
                    await serviceManager.getService<IChatViewService>(
                        CHAT_VIEW_SERVICE_NAME
                    );
                await chatViewService.setHasSelection(hasSelection);
            }
        );

        webviewView.onDidDispose(() => {
            eventDisposable.dispose();
            serviceManager.dispose();
            chatService.detachClient(this);
        });
    }

    handleMessageChange(msg: ChatMessage): void {
        const serviceManager = this.#serviceManager;
        if (!serviceManager) {
            return;
        }

        serviceManager
            .getService<IChatViewService>(CHAT_VIEW_SERVICE_NAME)
            .then((service) => {
                service.updateMessage(msg.id, msg.contents);
            });
    }

    static #buildWebviewContents(
        webview: vscode.Webview,
        baseUri: vscode.Uri
    ): string {
        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(baseUri, "webview.js")
        );
        const nonce = getNonce();

        return `
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width,initial-scale=1,shrink-to-fit=no">
                <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline' ; script-src 'nonce-${nonce}';">
                <title>CodeCursor</title>
                <script nonce="${nonce}">
                    window.__codeCursorPageName = "chat";
                </script>
            </head>
            <body>
                <div id="root"></div>
                <script nonce="${nonce}" src="${scriptUri}"></script>
            </body>
        </html>
        `;
    }
}
