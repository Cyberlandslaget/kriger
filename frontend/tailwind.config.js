/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        "primary-bg": "#0f192e",
        "red": {
          light: "#e8d7d7",
          DEFAULT: "#e78284",
          dark: "#7b2021"
        },
        "green": {
          light: "#dfe7db",
          DEFAULT: "#a6d189",
          dark: "#2d550f"
        },
        "yellow": {
          light: "#eae6df",
          DEFAULT: "#edda9b",
          dark: "#83611d"
        }
      },
    },
  },
  plugins: [],
};
