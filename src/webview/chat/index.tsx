import * as React from "react";
import { useState, useEffect, useCallback, useRef } from "react";
import { VSCodeButton, VSCodeTextArea } from "@vscode/webview-ui-toolkit/react";

import "./style.css";
import { MessageItem } from "./MessageItem";
import { ChatViewServiceImpl } from "./chatViewServiceImpl";
import { getServiceManager } from "../../common/ipc/webview";
import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";
import { MessageItemModel } from "../../common/chatService/model";

function messagesWithUpdatedBotMessage(
    msgs: MessageItemModel[],
    updatedMsg: MessageItemModel
): MessageItemModel[] {
    return msgs.map((msg) => {
        if (updatedMsg.id === msg.id) {
            return updatedMsg;
        }
        return msg;
    });
}

export function ChatPage() {
    const [messages, setMessages] = useState([] as MessageItemModel[]);
    const [hasSelection, setHasSelection] = useState(false);
    const [isReady, setIsReady] = useState(false);
    const [prompt, setPrompt] = useState("");

    // Dependent on `setMessages`, which will never change.
    const addMessageAction = useCallback((msg: MessageItemModel) => {
        setMessages((prev) => {
            return [...prev, msg];
        });
    }, []);
    const updateMessageAction = useCallback((msg: MessageItemModel) => {
        setMessages((prev) => {
            return messagesWithUpdatedBotMessage(prev, msg);
        });
    }, []);
    const clearMessageAction = useCallback(() => {
        setMessages([]);
    }, []);

    const handleAskAction = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );
        await chatService.confirmPrompt(prompt);
        setPrompt("");
    }, [prompt, setPrompt, setMessages]);

    useEffect(() => {
        const serviceManager = getServiceManager();

        const viewServiceImpl = new ChatViewServiceImpl();
        viewServiceImpl.setIsReadyAction = setIsReady;
        viewServiceImpl.setHasSelectionAction = setHasSelection;
        viewServiceImpl.addMessageAction = addMessageAction;
        viewServiceImpl.updateMessageAction = updateMessageAction;
        viewServiceImpl.clearMessageAction = clearMessageAction;
        serviceManager.registerService(viewServiceImpl);

        serviceManager
            .getService<IChatService>(CHAT_SERVICE_NAME)
            .then((chatService) => {
                chatService.syncState();
            });
    }, []);

    return (
        <div className="chat-root">
            <div className="chat-list">
                {messages.map((m) => {
                    return <MessageItem key={m.id} model={m} />;
                })}
            </div>
            <div className="chat-input-area">
                <VSCodeTextArea
                    style={{ width: "100%" }}
                    rows={3}
                    placeholder={`Talk about the ${
                        hasSelection ? "selected contents" : "whole document"
                    }...`}
                    disabled={!isReady}
                    value={prompt}
                    onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                        setPrompt(e.target.value);
                    }}
                />
                <VSCodeButton
                    disabled={!isReady || prompt.length === 0}
                    onClick={handleAskAction}
                >
                    Ask
                </VSCodeButton>
            </div>
        </div>
    );
}
