import * as vscode from "vscode";
import { signIn } from "@crates/cursor-core";
import { getGlobalState } from "../globalState";

export async function handleSignInCommand() {
    await signIn();
    // const globalState = getGlobalState();
    // let { authSession } = globalState;
    // if (authSession) {
    //     return;
    // }
    // authSession = new AuthSession();
    // globalState.authSession = authSession;
    // await vscode.commands.executeCommand(
    //     "vscode.open",
    //     vscode.Uri.parse(authSession.loginUrl)
    // );

    // await vscode.window.withProgress(
    //     {
    //         location: vscode.ProgressLocation.Notification,
    //         title: "Waiting...",
    //         cancellable: true,
    //     },
    //     async (_progress, cancellationToken) => {
    //         const abortController = new AbortController();
    //         cancellationToken.onCancellationRequested(() => {
    //             abortController.abort();
    //         });
    //         const { signal: abortSignal } = abortController;
    //         const token = await authSession?.polling(abortSignal);
    //         console.log(`AuthToken: ${token}`);
    //         globalState.authSession = null;
    //     }
    // );
}
