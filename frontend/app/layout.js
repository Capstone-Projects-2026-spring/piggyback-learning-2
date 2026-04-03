import "./globals.css";

import { Geist, Geist_Mono } from "next/font/google";
import Navbar from "@/components/Navbar";
import { AuthProvider } from "@/context/AuthContext";
import { SocketProvider } from "@/context/SocketContext";
import { Toaster } from "react-hot-toast";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata = {
  title: "Piggyback learning",
  description: "A learning app for kids",
};

const RootLayout = ({ children }) => (
  <html
    lang="en"
    className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    suppressHydrationWarning
  >
    <body suppressHydrationWarning className="min-h-full flex flex-col">
      <AuthProvider>
        <SocketProvider>
          <Navbar />
          {children}
          <Toaster position="top-right" />
        </SocketProvider>
      </AuthProvider>
    </body>
  </html>
);

export default RootLayout;
