"use client";

import { createContext, useContext, useEffect, useState } from "react";
import { connectSocket, disconnectSocket, sendMessage } from "@/utils/socket";
import { toast } from "react-hot-toast";
import { AuthContext } from "./AuthContext";

const SocketContext = createContext();

export const SocketProvider = ({ children }) => {
  const { account } = useContext(AuthContext);
  const username = account?.username;

  const [messages, setMessages] = useState([]);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    if (!username) return;

    connectSocket(
      username,
      (data) => {
        setMessages((prev) => [...prev, data]);

        // Toast notification
        if (data?.msg) {
          toast(`${data.sender}: ${data.msg}`, {
            style: {
              background: "#fef3c7",
              color: "#7c3aed",
              borderRadius: "12px",
            },
          });
        }
      },
      () => {
        setConnected(true);
        toast.success("Connected 🚀");
      },
      () => {
        setConnected(false);
        toast.error("Disconnected ❌");
      },
    );

    return () => {
      disconnectSocket();
    };
  }, [username]);

  const send = (payload) => sendMessage(payload);

  return (
    <SocketContext.Provider value={{ messages, connected, send, username }}>
      {children}
    </SocketContext.Provider>
  );
};

export const useSocket = () => useContext(SocketContext);
