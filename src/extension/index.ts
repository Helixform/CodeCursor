import * as vscode from "vscode";
import * as crypto from "crypto";

import { GenerateSession, getScratchpadManager } from "./generate";
import { getGlobalState } from "./globalState";
import { ChatPanelProvider } from "./chat/chatPanelProvider";
import { sharedChatServiceImpl } from "./chat/chatServiceImpl";
import { setExtensionContext, signIn, signOut } from "@crates/cursor-core";
import { ExtensionContext } from "./context";
import { handleGenerateProjectCommand } from "./project";

function setHasActiveGenerateSessionContext(value: boolean) {
    vscode.commands.executeCommand(
        "setContext",
        "aicursor.hasActiveGenerateSession",
        value
    );
}

async function handleGenerateCodeCommand() {
    const input = await vscode.window.showInputBox({
        placeHolder: "Instructions for code to generate...",
    });
    if (!input) {
        return;
    }

    // Get the current editor and selection.
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    // End the active session first.
    const globalState = getGlobalState();
    const activeSession = globalState.activeSession;
    if (activeSession) {
        activeSession.dispose();
    }

    const session = new GenerateSession(input, editor);
    session.onDidDispose(() => {
        globalState.activeSession = null;
        setHasActiveGenerateSessionContext(false);
    });
    session.start();
    session.showResult();
    globalState.activeSession = session;
    setHasActiveGenerateSessionContext(true);
}

export function activate(context: vscode.ExtensionContext) {
    // To use crypto features in WebAssembly, we need to add this polyfill.
    global.crypto = {
        getRandomValues: (arr: Uint8Array) => {
            crypto.randomFillSync(arr);
        },
    } as any;

    setExtensionContext(new ExtensionContext());
    getGlobalState().storage = context.globalState;

    context.subscriptions.push(
        vscode.commands.registerCommand("aicursor.generateCode", () => {
            handleGenerateCodeCommand();
        }),
        vscode.commands.registerCommand("aicursor.showLastResult", () => {
            getGlobalState().activeSession?.showResult();
        }),
        vscode.commands.registerCommand("aicursor.acceptChanges", () => {
            getGlobalState().activeSession?.applyChanges();
        }),
        vscode.commands.registerCommand("aicursor.rejectChanges", () => {
            const globalState = getGlobalState();
            globalState.activeSession?.dispose();
            globalState.activeSession = null;
        }),
        vscode.commands.registerCommand("aicursor.resetChat", () => {
            sharedChatServiceImpl().clearSession();
        }),
        vscode.commands.registerCommand("aicursor.signInUp", () => {
            signIn();
        }),
        vscode.commands.registerCommand("aicursor.signOut", () => {
            signOut();
        }),
        vscode.commands.registerCommand("aicursor.configureApiKey", () => {
            vscode.commands.executeCommand(
                "workbench.action.openSettings",
                "aicursor.openaiApiKey"
            );
        }),
        vscode.commands.registerCommand("aicursor.generateProject", () => {
            handleGenerateProjectCommand();
        }),
        getScratchpadManager().registerTextDocumentContentProvider(),
        vscode.window.registerWebviewViewProvider(
            ChatPanelProvider.viewType,
            new ChatPanelProvider(context)
        )
    );
}

export function deactivate() {
    const globalState = getGlobalState();
    globalState.activeSession?.dispose();
    globalState.activeSession = null;
    globalState.storage = null;
}
