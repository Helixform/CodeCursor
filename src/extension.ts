import * as vscode from "vscode";

import { GenerateSession, getScratchpadManager } from "./generate";
import { getGlobalState } from "./globalState";

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
    session.start();
    session.showResult();
    globalState.activeSession = session;
}

export function activate(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.commands.registerCommand("aicursor.generateCode", () => {
            handleGenerateCodeCommand();
        }),
        vscode.commands.registerCommand("aicursor.showLastResult", () => {
            getGlobalState().activeSession?.showResult();
        }),
        getScratchpadManager().registerTextDocumentContentProvider()
    );
}

export function deactivate() {
    const globalState = getGlobalState();
    globalState.activeSession?.dispose();
    globalState.activeSession = null;
}
