import * as vscode from "vscode";
import { GenerateSession, getScratchpadManager } from "./generate";

const globalState = {
    activeSession: null as GenerateSession | null,
};

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
    const selection = editor.selection;

    // End the active session first.
    const activeSession = globalState.activeSession;
    if (activeSession) {
        activeSession.dispose();
    }

    const session = new GenerateSession(input, selection, editor);
    session.start();
    session.showResult();
    globalState.activeSession = session;
}

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
    // The command has been defined in the package.json file
    // Now provide the implementation of the command with registerCommand
    // The commandId parameter must match the command field in package.json
    context.subscriptions.push(
        vscode.commands.registerCommand("aicursor.generateCode", () => {
            handleGenerateCodeCommand();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand("aicursor.showLastResult", () => {
            globalState.activeSession?.showResult();
        })
    );

    context.subscriptions.push(
        getScratchpadManager().registerTextDocumentContentProvider()
    );
}

// This method is called when your extension is deactivated
export function deactivate() {}
