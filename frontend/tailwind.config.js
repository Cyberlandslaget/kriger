/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        "primary-bg": "#0f192e",
        background: "#0f192e",
        border: "hsl(var(--border))",
        input: "rgb(148 163 184 / 0.4)", // bg-gray-200/70
        primary: "rgb(229 231 235 / 0.7)", // bg-slate-400/40
        red: {
          light: "#e8d7d7",
          DEFAULT: "#e78284",
          dark: "#7b2021",
        },
        green: {
          light: "#dfe7db",
          DEFAULT: "#a6d189",
          dark: "#2d550f",
        },
        yellow: {
          light: "#eae6df",
          DEFAULT: "#edda9b",
          dark: "#83611d",
        },
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
