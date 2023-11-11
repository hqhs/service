# Template for quick prototyping using Rust 

I prototype often, I wanted ready-to-go github repo to start new projects faster. This is pretty opinionated, but basically what I perceive as most productive, simplest and generally the best set of tools and decisions for current state of web development.

I know that Rust could not seem like a good language to do this, but best thing to choose for quick development is something you already know, and I've been doing Rust for multiple years now both professionally and personally. 

Another argument for Rust is that you'll gain time lost fighting the compiler in time you would spend debugging, deploying and testing otherwise.

## stack

Rust:
- axum
- sqlx 
- tera (template engine)

Storage:
- sqlite3

Frontend:
- htmx 
- alpine.js
- tailwind

## Targets 

Then I first started this project I was thinking about doing something crazy along the lines of "set the prototype flexible enough to build it for any deployment options," e.g.: 

- Build as classic standalone backend server
- Build as Service Worker to use in browser
- Deploy to Cloudflare 
- Build as standalone application (like electron, but simpler)

But the thing about prototypes is that they are SIMPLE. Modern web standards already allow you to basically do "whatever" in the browser, most of the time you really don't need native apps. So, if you want to deploy it, just run it somewhere you can access it and you're fine -- any 5$ dedicated linux box would do. 

I would still buy domain and put service behind Cloudflare (use tunnels), but that's me.

## testing
