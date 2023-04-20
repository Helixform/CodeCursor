import * as vscode from "vscode";
import * as fs from "node:fs";
import { generateProject } from "@crates/cursor-core";

export async function handleGenerateProjectCommand() {
    const workspace = getCurrentWorkspace();
    if (!workspace) {
        return;
    }

    const input = await vscode.window.showInputBox({
        title: "Generate A New Project",
        placeHolder: "Instructions for project to generate...",
    });
    if (!input) {
        return;
    }

    // Check if the workspace is empty.
    const files = await vscode.workspace.fs.readDirectory(workspace.uri);
    // Exclude hidden files.
    files.filter((file) => !file[0].startsWith("."));
    if (files.length > 0) {
        const confirmMessage = "Yes, I am sure";
        const cancelMessage = "No";
        const result = await vscode.window.showWarningMessage(
            "The current workspace is not empty. Your existing files may be accidentally modified. Are you sure you want to continue?",
            confirmMessage,
            cancelMessage
        );
        if (result !== confirmMessage) {
            return;
        }
    }

    await generateProject(input, {
        async createFileRecursive(path) {
            // `path` is the relative path to the current workspace.
            const absolutePath = getAbsolutePath(path);
            if (!absolutePath) {
                return;
            }
            // Create a file and recursively create intermediate folders if they do not exist.
            const uri = vscode.Uri.file(absolutePath);
            const parent = uri.with({
                path: uri.path.substring(0, uri.path.lastIndexOf("/")),
            });
            await vscode.workspace.fs.createDirectory(parent);
            await vscode.workspace.fs.writeFile(uri, new Uint8Array());
        },
        makeFileWriter(path) {
            const absolutePath = getAbsolutePath(path);
            if (!absolutePath) {
                return;
            }
            // Using the ability of stream writing files provided by NodeJS.
            const writeStream = fs.createWriteStream(absolutePath);
            return {
                write(data) {
                    writeStream.write(data);
                },
                end() {
                    writeStream.end();
                },
            };
        },
    });
}

function getAbsolutePath(path: string) {
    const workspace = getCurrentWorkspace();
    if (!workspace) {
        return;
    }
    return workspace.uri.path + "/" + path;
}

function getCurrentWorkspace() {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        return;
    }
    return workspaceFolders[0];
}
