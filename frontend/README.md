# Piggyback Learning (frontend)

A [Next.js](https://nextjs.org) project.

---

## Prerequisites

Before getting started, make sure you have the following installed:

- [Node.js](https://nodejs.org) v20.9.0 or later
- A package manager: [npm](https://www.npmjs.com/), [yarn](https://yarnpkg.com/), [pnpm](https://pnpm.io/), or [bun](https://bun.sh/)

---

## Installation

Clone the repository and install dependencies:

```bash
git clone https://github.com/Capstone-Projects-2026-spring/piggyback-learning-2.git
cd piggyback-learning-2/frontend
```

Then install dependencies with your preferred package manager:

```bash
npm install
# or
yarn install
# or
pnpm install
# or
bun install
```

---

## Environment Variables

Create a `.env` file in the `frontend/` directory:

```bash
cp env.example .env
```

---

## Running in Development

Start the development server with hot-reloading:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser to view the app.

---

## Building for Production

Compile and optimize the app for production:

```bash
npm run build
# or
yarn build
# or
pnpm build
# or
bun run build
```

Build output is generated in the `.next/` directory.

---

## Starting the Production Server

After building, start the production server:

```bash
npm run start
# or
yarn start
# or
pnpm start
# or
bun run start
```

The app will be available at [http://localhost:3000](http://localhost:3000).

> **Tip:** You can specify a custom port with the `-p` flag:
> ```bash
> npm run start -- -p 8080
> ```

---

## Project Structure

```
.
├── app/           		# App Router pages and layouts
├── components/    		# Components
├── context/       		# Custom context for authentication and websocket
├── hooks/         		# Custom hooks
├── public/        		# Static assets
├── utils/         		# Utility functions
├── .next/         		# Production build output (generated)
└── next.config.mjs		# Next.js configuration
```

---

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [Next.js GitHub Repository](https://github.com/vercel/next.js)
- [Deploy on Vercel](https://vercel.com/new)
