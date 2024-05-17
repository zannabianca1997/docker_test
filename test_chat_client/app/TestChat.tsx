'use client'
import { ChangeEvent, FormEvent, useEffect, useState } from 'react';
import styles from './styles.module.css'
import { Board, Message, StoredMessage } from './bindings';

function start_user(): string {
    let result = 'anon_';
    const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    const charactersLength = characters.length;
    for (let i = 0; i < 5; i++) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
}

function is_valid_user(user: string): boolean {
    return user.trim() === "";
}

function find_api_url(): URL | null {
    if (typeof document !== 'undefined') {
        const url = new URL(document.URL);
        url.port = "4000";
        url.pathname = "/";
        return url;
    } else {
        return null;
    }
}

export default function TestChat() {
    // The last valid user
    const [user, setUser] = useState<string>(start_user());
    // Last time got from the server
    const [time, setTime] = useState<Date | null>(null);
    // Last time got from the server
    const [title, setTitle] = useState<string>("");
    // Message received from the server
    const [messages, setMessages] = useState<StoredMessage[]>([]);


    const update_messages = async () => {
        const APIURL = find_api_url();
        if (APIURL) {
            const response = await fetch(APIURL);

            if (!response.ok) {
                console.log(response)
            }

            const { title, time, messages } = (await response.json() as Board);

            setTime(new Date(Date.parse(time)));
            setTitle(title);
            setMessages(messages);
        }
    }

    const sendMessage = async (content: string) => {
        const APIURL = find_api_url();
        if (APIURL) {
            const message: Message = { user, content };

            const response = await fetch(APIURL, {
                mode: 'cors',
                method: 'post',
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(message)
            });

            if (!response.ok) {
                // ups. Logging for now
                console.log(response)
            }

            // update the message board instantly
            await update_messages()
        }
    };

    // update the page every second
    useEffect(() => {
        const interval = setInterval(update_messages, 1000);

        return () => clearInterval(interval);
    }, [])

    return <div className={styles.TestChat}>
        <TopBar title={title} time={time} />
        <Messages user={user} messages={messages} />
        <InputBar user={user} setUser={setUser} sendMessage={sendMessage} />
    </div>;
}

function TopBar({ title, time }: { title: string, time: null | Date }) {
    return <div className={styles.TopBar}>
        <div className={styles.Title}>{title}</div>
        <div className={styles.Time}>{time?.toLocaleTimeString()}</div>
    </div>;
}

function Messages({ user, messages }: { user: string, messages: StoredMessage[] }) {
    return <div className={styles.Messages}>
        {messages.map((message) => (<MessageEl message={message} own={message.user === user} />))}
    </div>
}

function MessageEl({ message: { user, content, time }, own }: { message: StoredMessage, own: boolean }) {
    return <div className={styles.Message + (own ? (" " + styles.OwnMessage) : "")}>
        <div className={styles.User}>{user}</div>
        <div className={styles.Time}>{new Date(Date.parse(time)).toLocaleTimeString()}</div>
        <div className={styles.Content}>{content}</div>
    </div>
}

function InputBar(
    { user, setUser, sendMessage }:
        {
            user: string,
            setUser: (user: string) => void,
            sendMessage: (content: string) => void,
        }
) {
    const [user_valid, setUserValid] = useState(true)

    const submit = async (e: FormEvent) => {
        e.preventDefault();

        if (!user_valid) {
            return;
        }

        // read the message and clear the input box
        const msg_input = document.getElementById('msg_input') as HTMLInputElement;
        const content = msg_input.value;
        msg_input.value = "";

        if (content.trim() === "") {
            // Do not send empty messages
            return;
        }

        sendMessage(content)
    };

    const user_changed = (e: ChangeEvent) => {
        const user = (e.target as HTMLInputElement).value;
        setUser(user.trim())

        if (is_valid_user(user)) {
            setUserValid(false);
        } else {
            setUserValid(true);
        }
    };


    return <form className={styles.InputBar}>
        <input id='user_input' type='text' placeholder='user' className={
            styles.UserInput +
            ((!user_valid) ? (" " + styles.WrongUser) : "")} onChange={user_changed} value={user} />
        <input id='msg_input' type='text' className={styles.MsgInput} />
        <input type='submit' value='Send' disabled={!user_valid} onClick={submit} className={styles.MsgSend} />
    </form>
}
