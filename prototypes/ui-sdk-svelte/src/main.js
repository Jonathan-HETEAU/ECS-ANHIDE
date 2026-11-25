// Le pont pour agents doit exister avant le premier rendu (voir agent.js).
import "./agent.js"
import "@ui-sdk/core/web.css"
import "./theme.css"
import { mount } from "svelte"
import App from "./App.svelte"

mount(App, { target: document.getElementById("app") })
