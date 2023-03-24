import * as React from "react";
import { useState, useEffect } from "react";
import { VSCodeButton, VSCodeTextArea } from "@vscode/webview-ui-toolkit/react";

import "./style.css";
import { MessageItem, MessageItemModel } from "./MessageItem";

export function ChatPage() {
    const [messages, setMessages] = useState([] as MessageItemModel[]);
    const [prompt, setPrompt] = useState("");
    useEffect(() => {
        setMessages(
            Array.from({ length: 20 }).map((_, index) => {
                const isReply = index % 2 !== 0;
                return {
                    id: "" + index,
                    contents: isReply
                        ? "It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like)."
                        : "Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
                    isReply: index % 2 !== 0,
                };
            })
        );
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
                    placeholder="Talk about the whole document..."
                    value={prompt}
                    onInput={(e: React.ChangeEvent<HTMLTextAreaElement>) => {
                        setPrompt(e.target.value);
                    }}
                />
                <VSCodeButton disabled={prompt.length === 0}>Ask</VSCodeButton>
            </div>
        </div>
    );
}
