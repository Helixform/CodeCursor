import * as vscode from "vscode";
import {
    IExtensionContext,
    IGlobalStorage,
    IModelConfiguration,
    IProgress,
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
        callback: (progress: IProgress, signal: AbortSignal) => Thenable<any>
    ): Thenable<any> {
        return vscode.window.withProgress(options, async (progress, token) => {
            const abortController = new AbortController();
            token.onCancellationRequested(() => {
                abortController.abort();
            });
            const wrappedProgress = {
                report(message?: string) {
                    progress.report({ message });
                },
            } as IProgress;
            try {
                await callback(wrappedProgress, abortController.signal);
            } catch (e) {
                console.error(e);
            }
        });
    }

    showInformationMessage(
        message: string,
        items: string[]
    ): Thenable<string | undefined> {
        return vscode.window.showInformationMessage(message, ...items);
    }

    getModelConfiguration(): IModelConfiguration {
        const config = vscode.workspace.getConfiguration("aicursor");
        let apiKey: string | null = config.get("openaiApiKey", null);
        if (apiKey === "") {
            apiKey = null;
        }
        const model = config.get("model", "");

        return {
            apiKey: apiKey,
            gptModel: model,
        };
    }
}
