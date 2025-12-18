/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'method-vi': {
          primary: '#2563eb',
          secondary: '#1e40af',
          success: '#10b981',
          warning: '#f59e0b',
          danger: '#ef4444',
          neutral: '#6b7280',
        }
      }
    },
  },
  plugins: [],
}
