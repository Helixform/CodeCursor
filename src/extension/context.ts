import * as vscode from "vscode";
import {
    IExtensionContext,
    IGlobalStorage,
    RustProgressOptions,
} from "@crates/cursor-core";
import { getGlobalState } from "./globalState";

const storageAdapter = {
    get(key: string) {
        return getGlobalState().storage?.get(key) ?? null;
    },
    update(key, value) {
        getGlobalState().storage?.update(key, value);
    },
} satisfies IGlobalStorage;

export class ExtensionContext implements IExtensionContext {
    get storage(): IGlobalStorage {
        return storageAdapter;
    }

    executeCommand(command: string, ...args: any[]): Thenable<any> {
        return vscode.commands.executeCommand(command, ...args);
    }

    executeCommand0(command: string): Thenable<any> {
        return vscode.commands.executeCommand(command);
    }

    withProgress(
        options: RustProgressOptions,
        callback: (signal: AbortSignal) => Thenable<any>
    ): Thenable<any> {
        return vscode.window.withProgress(options, async (_, token) => {
            const abortController = new AbortController();
            token.onCancellationRequested(() => {
                abortController.abort();
            });
            await callback(abortController.signal);
        });
    }

    showInformationMessage(
        message: string,
        items: string[]
    ): Thenable<string | undefined> {
        return vscode.window.showInformationMessage(message, ...items);
    }
}
