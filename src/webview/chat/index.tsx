import * as React from "react";
import { useState, useEffect, useCallback, useRef } from "react";
import { VSCodeButton, VSCodeTextArea } from "@vscode/webview-ui-toolkit/react";

import "./style.css";
import { MessageItem, MessageItemModel } from "./MessageItem";
import { ChatViewServiceImpl } from "./chatViewServiceImpl";
import { getServiceManager } from "../../common/ipc/webview";
import { IChatService, CHAT_SERVICE_NAME } from "../../common/chatService";

function messagesWithUpdatedBotMessage(
    msgs: MessageItemModel[],
    updatedMsgId: number,
    contents: string
): MessageItemModel[] {
    return msgs.map((msg) => {
        if ("bot" + updatedMsgId === msg.id) {
            return { id: msg.id, contents, isReply: msg.isReply };
        }
        return msg;
    });
}

export function ChatPage() {
    const [messages, setMessages] = useState([] as MessageItemModel[]);
    const [hasSelection, setHasSelection] = useState(false);
    const [prompt, setPrompt] = useState("");

    // Dependent on `setMessages`, which will never change.
    const updateMessageAction = useCallback(
        (msgId: number, contents: string) => {
            setMessages((prev) => {
                return messagesWithUpdatedBotMessage(prev, msgId, contents);
            });
        },
        []
    );

    const handleAskAction = useCallback(async () => {
        const chatService = await getServiceManager().getService<IChatService>(
            CHAT_SERVICE_NAME
        );
        const msgId = await chatService.confirmPrompt(prompt);
        setMessages((prev) => {
            return [
                ...prev,
                { id: "user" + (prev.length + 1), contents: prompt },
                {
                    id: "bot" + msgId,
                    contents: "",
                    isReply: true,
                },
            ];
        });
        setPrompt("");
    }, [prompt, setPrompt, setMessages]);

    useEffect(() => {
        const viewServiceImpl = new ChatViewServiceImpl();
        viewServiceImpl.setHasSelectionAction = setHasSelection;
        viewServiceImpl.updateMessageAction = updateMessageAction;
        getServiceManager().registerService(viewServiceImpl);
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
                    value={prompt}
                    onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                        setPrompt(e.target.value);
                    }}
                />
                <VSCodeButton
                    disabled={prompt.length === 0}
                    onClick={handleAskAction}
                >
                    Ask
                </VSCodeButton>
            </div>
        </div>
    );
}
