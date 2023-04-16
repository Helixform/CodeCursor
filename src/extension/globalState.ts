import { Memento } from "vscode";
import { GenerateSession } from "./generate";
// import { AuthSession } from "@crates/cursor-core";

interface GlobalState {
    activeSession: GenerateSession | null;
    // authSession: AuthSession | null;
    storage: Memento | null;
}

const globalState: GlobalState = {
    activeSession: null,
    // authSession: null,
    storage: null,
};

export function getGlobalState() {
    return globalState;
}
