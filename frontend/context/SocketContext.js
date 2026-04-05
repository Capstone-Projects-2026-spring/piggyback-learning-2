"use client";

import { createContext, useContext, useEffect, useState } from "react";
import { connectSocket, disconnectSocket, sendMessage } from "@/utils/socket";
import { AuthContext } from "./AuthContext";
import ToastModal from "@/components/ToastModal";

const SocketContext = createContext();

export const SocketProvider = ({ children }) => {
  const { account } = useContext(AuthContext);
  const username = account?.username;

  const [messages, setMessages] = useState([]);
  const [connected, setConnected] = useState(false);
  const [modal, setModal] = useState(null);

  const getModalStyle = (data) => {
    switch (data?.action) {
      case "bored":
        return {
          bg: "bg-yellow-100",
          border: "border-yellow-400",
          text: "text-yellow-700",
          emoji: "😐",
          title: `${data?.sender} seems bored!`,
        };
      case "distracted":
        return {
          bg: "bg-red-100",
          border: "border-red-400",
          text: "text-red-700",
          emoji: "😵",
          title: `${data?.sender} seems distracted!`,
        };
      case "focused":
        return {
          bg: "bg-green-100",
          border: "border-green-400",
          text: "text-green-700",
          emoji: "🎯",
          title: `${data?.sender} is focused again!`,
        };
      default:
        return {
          bg: "bg-purple-100",
          border: "border-purple-400",
          text: "text-purple-700",
          emoji: "💬",
          title: "New Message",
        };
    }
  };

  useEffect(() => {
    if (!username) return;

    connectSocket(
      username,
      (data) => {
        setMessages((prev) => [...prev, data]);
        setModal(data);
      },
      () => setConnected(true),
      () => setConnected(false),
    );

    return () => disconnectSocket();
  }, [username]);

  const send = (payload) => sendMessage(payload);

  const closeModal = () => setModal(null);

  return (
    <SocketContext.Provider value={{ messages, connected, send, username }}>
      {children}

      {modal && (
        <ToastModal
          data={modal}
          onClose={closeModal}
          getStyle={getModalStyle}
        />
      )}
    </SocketContext.Provider>
  );
};

export const useSocket = () => useContext(SocketContext);
