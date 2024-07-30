/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class', // <html class="dark"> to toggle, refer to https://tailwindcss.com/docs/dark-mode
  content: ['./templates/**/*.jinja2'],
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/typography'),
    require('@tailwindcss/forms'),
  ],
}
