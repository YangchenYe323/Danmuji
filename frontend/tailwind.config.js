module.exports = {
  content: [
    "index.html",
    "./src/**/*.{html,js,jsx}"
  ],
  theme: {
    extend: {
      keyframes: {
        wave: {
          '0%': { transform: 'rotate(0.0deg)' },
          '10%': { transform: 'rotate(14deg)' },
          '20%': { transform: 'rotate(-8deg)' },
          '30%': { transform: 'rotate(14deg)' },
          '40%': { transform: 'rotate(-4deg)' },
          '50%': { transform: 'rotate(10.0deg)' },
          '60%': { transform: 'rotate(0.0deg)' },
          '100%': { transform: 'rotate(0.0deg)' },
        },

        movein: {
          '0%': {
            transform: 'translateX(30px)',
            opacity: 0,
          },

          '100%': {
            transform: 'translateX(0)',
            opacity: 1
          }
        }
      },

      animation: {
        'waving-hand': 'wave 2s linear infinite',
        'danmaku-movein': 'movein 1s linear',
      }

    },
  },
  plugins: [],
}
