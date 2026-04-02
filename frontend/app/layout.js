import "./globals.css";

import { Geist, Geist_Mono } from "next/font/google";
import { AuthProvider } from "./context/AuthContxt";
import Navbar from "@/components/Navbar";

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
        <Navbar />
        {children}
      </AuthProvider>
    </body>
  </html>
);

export default RootLayout;
